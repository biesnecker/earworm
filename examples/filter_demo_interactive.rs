//! Interactive filter demonstration using BiquadFilter.
//!
//! Press SPACE to cycle through different filter configurations.
//! Press Q or ESC to quit.

use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, StreamConfig};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use earworm::{AudioSignalExt, BiquadFilter, Signal, SignalExt, SineOscillator, TriangleOscillator};
use std::io::{stdout, Write};
use std::panic;
use std::sync::{Arc, Mutex};

/// Different filter configurations to cycle through
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilterMode {
    Raw,                    // No filter
    LowPass,                // Low-pass filter
    HighPass,               // High-pass filter
    BandPass,               // Band-pass filter
    ResonantLowPass,        // Low-pass with high resonance
    SweptLowPass,           // Low-pass with LFO modulation
    ChainedFilters,         // Multiple filters in series
    NotchFilter,            // Notch filter
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

/// Enum to hold different filter types
enum FilteredSignal {
    Raw(TriangleOscillator),
    LowPass(BiquadFilter<TriangleOscillator>),
    HighPass(BiquadFilter<TriangleOscillator>),
    BandPass(BiquadFilter<TriangleOscillator>),
    ResonantLowPass(BiquadFilter<TriangleOscillator>),
    SweptLowPass(BiquadFilter<TriangleOscillator>),
    ChainedFilters(BiquadFilter<BiquadFilter<TriangleOscillator>>),
    NotchFilter(BiquadFilter<TriangleOscillator>),
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
        // Scale output to avoid clipping
        sample * 0.3
    }
}

/// Audio state that manages the current filter configuration
struct AudioState {
    signal: FilteredSignal,
    mode: FilterMode,
    base_frequency: f64,
    sample_rate: f64,
}

impl AudioState {
    fn new(base_frequency: f64, sample_rate: f64) -> Self {
        let osc = TriangleOscillator::new(base_frequency, sample_rate);
        Self {
            signal: FilteredSignal::Raw(osc),
            mode: FilterMode::Raw,
            base_frequency,
            sample_rate,
        }
    }

    fn set_mode(&mut self, mode: FilterMode) {
        self.mode = mode;

        // Create new signal chain based on mode
        self.signal = match mode {
            FilterMode::Raw => {
                FilteredSignal::Raw(TriangleOscillator::new(self.base_frequency, self.sample_rate))
            }

            FilterMode::LowPass => {
                let osc = TriangleOscillator::new(self.base_frequency, self.sample_rate);
                FilteredSignal::LowPass(osc.lowpass_filter(800.0, 0.707))
            }

            FilterMode::HighPass => {
                let osc = TriangleOscillator::new(self.base_frequency, self.sample_rate);
                FilteredSignal::HighPass(osc.highpass_filter(600.0, 0.707))
            }

            FilterMode::BandPass => {
                let osc = TriangleOscillator::new(self.base_frequency, self.sample_rate);
                FilteredSignal::BandPass(osc.bandpass_filter(self.base_frequency, 5.0))
            }

            FilterMode::ResonantLowPass => {
                let osc = TriangleOscillator::new(self.base_frequency, self.sample_rate);
                FilteredSignal::ResonantLowPass(osc.lowpass_filter(500.0, 10.0))
            }

            FilterMode::SweptLowPass => {
                let osc = TriangleOscillator::new(self.base_frequency, self.sample_rate);
                let lfo = SineOscillator::new(0.5, self.sample_rate);
                // LFO sweeps from 300Hz to 1500Hz
                let modulated_cutoff = lfo.gain(600.0).offset(900.0);
                FilteredSignal::SweptLowPass(osc.lowpass_filter(modulated_cutoff, 2.0))
            }

            FilterMode::ChainedFilters => {
                let osc = TriangleOscillator::new(self.base_frequency, self.sample_rate);
                let chained = osc
                    .highpass_filter(100.0, 0.707)
                    .lowpass_filter(1000.0, 0.707);
                FilteredSignal::ChainedFilters(chained)
            }

            FilterMode::NotchFilter => {
                let osc = TriangleOscillator::new(self.base_frequency, self.sample_rate);
                FilteredSignal::NotchFilter(osc.notch_filter(self.base_frequency, 8.0))
            }
        };
    }

    fn next_sample(&mut self) -> f64 {
        self.signal.next_sample()
    }
}

fn run_audio_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    state: Arc<Mutex<AudioState>>,
) -> Result<cpal::Stream>
where
    T: Sample + FromSample<f64> + cpal::SizedSample,
{
    let channels = config.channels as usize;

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            let mut state = state.lock().unwrap();
            for frame in data.chunks_mut(channels) {
                let sample = state.next_sample();
                let value: T = T::from_sample(sample);
                for s in frame.iter_mut() {
                    *s = value;
                }
            }
        },
        |err| eprintln!("Audio stream error: {}", err),
        None,
    )?;

    stream.play()?;
    Ok(stream)
}

fn draw_ui(mode: FilterMode) -> Result<()> {
    let mut stdout = stdout();

    // Clear and show simple status
    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;
    stdout.execute(crossterm::cursor::MoveTo(0, 0))?;

    write!(
        stdout,
        "Filter: {} | SPACE=switch Q=quit",
        mode.name()
    )?;

    stdout.flush()?;
    Ok(())
}

/// Cleanup function to restore terminal state
fn cleanup_terminal() {
    let _ = stdout().execute(crossterm::cursor::Show);
    let _ = stdout().execute(LeaveAlternateScreen);
    let _ = disable_raw_mode();
}

fn main() -> Result<()> {
    let frequency = 220.0; // A3 note (lower for better filter demonstration)

    // Setup audio
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::anyhow!("No output device available"))?;

    let config = device.default_output_config()?;
    let sample_rate = config.sample_rate().0 as f64;

    let state = Arc::new(Mutex::new(AudioState::new(frequency, sample_rate)));

    // Start audio stream
    let _stream = match config.sample_format() {
        SampleFormat::F32 => run_audio_stream::<f32>(&device, &config.into(), state.clone())?,
        SampleFormat::I16 => run_audio_stream::<i16>(&device, &config.into(), state.clone())?,
        SampleFormat::U16 => run_audio_stream::<u16>(&device, &config.into(), state.clone())?,
        sample_format => {
            return Err(anyhow::anyhow!(
                "Unsupported sample format: {}",
                sample_format
            ))
        }
    };

    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(crossterm::cursor::Hide)?;

    // Set up panic hook to restore terminal on panic
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        cleanup_terminal();
        original_hook(panic_info);
    }));

    // Draw initial UI
    draw_ui(state.lock().unwrap().mode)?;

    // Event loop
    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break,
                    KeyCode::Char(' ') => {
                        let mut s = state.lock().unwrap();
                        let next_mode = s.mode.next();
                        s.set_mode(next_mode);
                        drop(s);
                        draw_ui(next_mode)?;
                    }
                    _ => {}
                }
            }
        }
    }

    // Cleanup terminal
    cleanup_terminal();

    Ok(())
}
