//! Interactive filter demonstration using BiquadFilter.
//!
//! Press SPACE to cycle through different filter configurations.
//! Press Q or ESC to quit.

mod common;

use anyhow::Result;
use common::{ExampleAudioState, KeyAction, KeyboardConfig, is_quit_key, run_interactive_example};
use crossterm::{
    ExecutableCommand,
    event::{KeyCode, KeyEvent},
};
use earworm::{
    AudioSignalExt, BiquadFilter, Signal, SignalExt, SineOscillator, TriangleOscillator,
};
use std::io::{Write, stdout};

const SAMPLE_RATE: u32 = 44100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilterMode {
    Raw,
    LowPass,
    HighPass,
    BandPass,
    ResonantLowPass,
    SweptLowPass,
    ChainedFilters,
    NotchFilter,
}

impl FilterMode {
    fn next(&self) -> Self {
        match self {
            FilterMode::Raw => FilterMode::LowPass,
            FilterMode::LowPass => FilterMode::HighPass,
            FilterMode::HighPass => FilterMode::BandPass,
            FilterMode::BandPass => FilterMode::ResonantLowPass,
            FilterMode::ResonantLowPass => FilterMode::SweptLowPass,
            FilterMode::SweptLowPass => FilterMode::ChainedFilters,
            FilterMode::ChainedFilters => FilterMode::NotchFilter,
            FilterMode::NotchFilter => FilterMode::Raw,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            FilterMode::Raw => "Raw Signal",
            FilterMode::LowPass => "Low-Pass (800Hz)",
            FilterMode::HighPass => "High-Pass (600Hz)",
            FilterMode::BandPass => "Band-Pass (220Hz)",
            FilterMode::ResonantLowPass => "Resonant LP (500Hz)",
            FilterMode::SweptLowPass => "Swept LP (LFO)",
            FilterMode::ChainedFilters => "Chained (HP→LP)",
            FilterMode::NotchFilter => "Notch (220Hz)",
        }
    }
}

enum FilteredSignal {
    Raw(TriangleOscillator<SAMPLE_RATE>),
    LowPass(BiquadFilter<SAMPLE_RATE, TriangleOscillator<SAMPLE_RATE>>),
    HighPass(BiquadFilter<SAMPLE_RATE, TriangleOscillator<SAMPLE_RATE>>),
    BandPass(BiquadFilter<SAMPLE_RATE, TriangleOscillator<SAMPLE_RATE>>),
    ResonantLowPass(BiquadFilter<SAMPLE_RATE, TriangleOscillator<SAMPLE_RATE>>),
    SweptLowPass(BiquadFilter<SAMPLE_RATE, TriangleOscillator<SAMPLE_RATE>>),
    ChainedFilters(
        BiquadFilter<SAMPLE_RATE, BiquadFilter<SAMPLE_RATE, TriangleOscillator<SAMPLE_RATE>>>,
    ),
    NotchFilter(BiquadFilter<SAMPLE_RATE, TriangleOscillator<SAMPLE_RATE>>),
}

impl Signal for FilteredSignal {
    fn next_sample(&mut self) -> f64 {
        let sample = match self {
            FilteredSignal::Raw(osc) => osc.next_sample(),
            FilteredSignal::LowPass(filter) => filter.next_sample(),
            FilteredSignal::HighPass(filter) => filter.next_sample(),
            FilteredSignal::BandPass(filter) => filter.next_sample(),
            FilteredSignal::ResonantLowPass(filter) => filter.next_sample(),
            FilteredSignal::SweptLowPass(filter) => filter.next_sample(),
            FilteredSignal::ChainedFilters(filter) => filter.next_sample(),
            FilteredSignal::NotchFilter(filter) => filter.next_sample(),
        };
        sample * 0.3
    }
}

struct AudioState {
    signal: FilteredSignal,
    mode: FilterMode,
    base_frequency: f64,
}

impl AudioState {
    fn new(base_frequency: f64) -> Self {
        let osc = TriangleOscillator::new(base_frequency);
        Self {
            signal: FilteredSignal::Raw(osc),
            mode: FilterMode::Raw,
            base_frequency,
        }
    }

    fn set_mode(&mut self, mode: FilterMode) {
        self.mode = mode;
        self.signal = match mode {
            FilterMode::Raw => FilteredSignal::Raw(TriangleOscillator::new(self.base_frequency)),
            FilterMode::LowPass => FilteredSignal::LowPass(
                TriangleOscillator::new(self.base_frequency).lowpass_filter(800.0, 0.707),
            ),
            FilterMode::HighPass => FilteredSignal::HighPass(
                TriangleOscillator::new(self.base_frequency).highpass_filter(600.0, 0.707),
            ),
            FilterMode::BandPass => FilteredSignal::BandPass(
                TriangleOscillator::new(self.base_frequency)
                    .bandpass_filter(self.base_frequency, 5.0),
            ),
            FilterMode::ResonantLowPass => FilteredSignal::ResonantLowPass(
                TriangleOscillator::new(self.base_frequency).lowpass_filter(500.0, 10.0),
            ),
            FilterMode::SweptLowPass => {
                let lfo = SineOscillator::<SAMPLE_RATE>::new(0.5)
                    .gain(600.0)
                    .offset(900.0);
                FilteredSignal::SweptLowPass(
                    TriangleOscillator::new(self.base_frequency).lowpass_filter(lfo, 2.0),
                )
            }
            FilterMode::ChainedFilters => FilteredSignal::ChainedFilters(
                TriangleOscillator::new(self.base_frequency)
                    .highpass_filter(100.0, 0.707)
                    .lowpass_filter(1000.0, 0.707),
            ),
            FilterMode::NotchFilter => FilteredSignal::NotchFilter(
                TriangleOscillator::new(self.base_frequency).notch_filter(self.base_frequency, 8.0),
            ),
        };
    }
}

impl ExampleAudioState for AudioState {
    fn next_sample(&mut self) -> f64 {
        self.signal.next_sample()
    }
}

fn draw_ui(mode: FilterMode) -> Result<()> {
    let mut stdout = stdout();
    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;
    stdout.execute(crossterm::cursor::MoveTo(0, 0))?;
    write!(stdout, "Filter: {} | SPACE=switch Q=quit", mode.name())?;
    stdout.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    run_interactive_example(
        AudioState::new(220.0),
        KeyboardConfig::default(),
        |state| draw_ui(state.lock().unwrap().mode),
        |state, key_event: &KeyEvent| match key_event.code {
            KeyCode::Char(' ') => {
                let mut s = state.lock().unwrap();
                let next_mode = s.mode.next();
                s.set_mode(next_mode);
                drop(s);
                draw_ui(next_mode)?;
                Ok(KeyAction::Continue)
            }
            code if is_quit_key(code) => Ok(KeyAction::Exit),
            _ => Ok(KeyAction::Continue),
        },
    )
}
