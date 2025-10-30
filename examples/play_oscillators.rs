//! Interactive example for switching between oscillator types.
//!
//! Press SPACE to cycle through oscillator types.
//! Press Q or ESC to quit.

mod common;

use anyhow::Result;
use common::{ExampleAudioState, KeyAction, KeyboardConfig, is_quit_key, run_interactive_example};
use crossterm::{
    ExecutableCommand,
    event::{KeyCode, KeyEvent},
};
use earworm::{
    PulseOscillator, SawtoothOscillator, Signal, SineOscillator, SquareOscillator,
    TriangleOscillator,
};
use std::io::{Write, stdout};

const SAMPLE_RATE: u32 = 44100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OscillatorType {
    Sine,
    Triangle,
    Sawtooth,
    Square,
    Pulse,
    PulseLFO,
}

impl OscillatorType {
    fn next(self) -> Self {
        match self {
            OscillatorType::Sine => OscillatorType::Triangle,
            OscillatorType::Triangle => OscillatorType::Sawtooth,
            OscillatorType::Sawtooth => OscillatorType::Square,
            OscillatorType::Square => OscillatorType::Pulse,
            OscillatorType::Pulse => OscillatorType::PulseLFO,
            OscillatorType::PulseLFO => OscillatorType::Sine,
        }
    }

    fn name(self) -> &'static str {
        match self {
            OscillatorType::Sine => "Sine",
            OscillatorType::Triangle => "Triangle",
            OscillatorType::Sawtooth => "Sawtooth",
            OscillatorType::Square => "Square",
            OscillatorType::Pulse => "Pulse (25%)",
            OscillatorType::PulseLFO => "Pulse (PWM)",
        }
    }
}

enum OscillatorWrapper {
    Sine(SineOscillator<SAMPLE_RATE>),
    Triangle(TriangleOscillator<SAMPLE_RATE>),
    Sawtooth(SawtoothOscillator<SAMPLE_RATE>),
    Square(SquareOscillator<SAMPLE_RATE>),
    Pulse(PulseOscillator<SAMPLE_RATE>),
    PulseLFO(PulseOscillator<SAMPLE_RATE>),
}

impl OscillatorWrapper {
    fn new(osc_type: OscillatorType, frequency: f64) -> Self {
        match osc_type {
            OscillatorType::Sine => {
                OscillatorWrapper::Sine(SineOscillator::<SAMPLE_RATE>::new(frequency))
            }
            OscillatorType::Triangle => {
                OscillatorWrapper::Triangle(TriangleOscillator::new(frequency))
            }
            OscillatorType::Sawtooth => {
                OscillatorWrapper::Sawtooth(SawtoothOscillator::new(frequency))
            }
            OscillatorType::Square => OscillatorWrapper::Square(SquareOscillator::new(frequency)),
            OscillatorType::Pulse => {
                OscillatorWrapper::Pulse(PulseOscillator::new(frequency, 0.25.into()))
            }
            OscillatorType::PulseLFO => {
                let lfo = SineOscillator::<SAMPLE_RATE>::new(0.5);
                OscillatorWrapper::PulseLFO(PulseOscillator::new(frequency, lfo.into()))
            }
        }
    }
}

impl Signal for OscillatorWrapper {
    fn next_sample(&mut self) -> f64 {
        match self {
            OscillatorWrapper::Sine(osc) => osc.next_sample(),
            OscillatorWrapper::Triangle(osc) => osc.next_sample(),
            OscillatorWrapper::Sawtooth(osc) => osc.next_sample(),
            OscillatorWrapper::Square(osc) => osc.next_sample(),
            OscillatorWrapper::Pulse(osc) => osc.next_sample(),
            OscillatorWrapper::PulseLFO(osc) => osc.next_sample(),
        }
    }
}

struct AudioState {
    oscillator: OscillatorWrapper,
    osc_type: OscillatorType,
    frequency: f64,
    fade_samples: usize,
}

impl AudioState {
    fn new(frequency: f64) -> Self {
        let osc_type = OscillatorType::Sine;
        Self {
            oscillator: OscillatorWrapper::new(osc_type, frequency),
            osc_type,
            frequency,
            fade_samples: 0,
        }
    }

    fn switch_oscillator(&mut self) {
        self.osc_type = self.osc_type.next();
        self.oscillator = OscillatorWrapper::new(self.osc_type, self.frequency);
        self.fade_samples = (SAMPLE_RATE as f64 * 0.002) as usize;
    }
}

impl ExampleAudioState for AudioState {
    fn next_sample(&mut self) -> f64 {
        let sample = self.oscillator.next_sample();

        if self.fade_samples > 0 {
            let fade_start = (SAMPLE_RATE as f64 * 0.002) as usize;
            let fade_progress = 1.0 - (self.fade_samples as f64 / fade_start as f64);
            self.fade_samples -= 1;
            sample * fade_progress
        } else {
            sample
        }
    }
}

fn draw_ui(osc_type: OscillatorType, frequency: f64) -> Result<()> {
    let mut stdout = stdout();
    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;
    stdout.execute(crossterm::cursor::MoveTo(0, 0))?;
    write!(
        stdout,
        "Playing: {} @ {:.0}Hz | SPACE=switch Q=quit",
        osc_type.name(),
        frequency
    )?;
    stdout.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let frequency = 440.0;

    run_interactive_example(
        AudioState::new(frequency),
        KeyboardConfig::default(),
        |state| {
            let state = state.lock().unwrap();
            draw_ui(state.osc_type, state.frequency)
        },
        |state, key_event: &KeyEvent| match key_event.code {
            KeyCode::Char(' ') => {
                let mut state = state.lock().unwrap();
                state.switch_oscillator();
                let osc_type = state.osc_type;
                drop(state);
                draw_ui(osc_type, frequency)?;
                Ok(KeyAction::Continue)
            }
            code if is_quit_key(code) => Ok(KeyAction::Exit),
            _ => Ok(KeyAction::Continue),
        },
    )
}
