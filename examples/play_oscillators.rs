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
    InterpolationMode, PulseOscillator, SawtoothOscillator, Signal, SineOscillator,
    SquareOscillator, TriangleOscillator, WavetableOscillator,
};
use std::f64::consts::PI;
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
    WavetableHarmonics,
    WavetableOrgan,
    WavetableVowel,
}

impl OscillatorType {
    fn next(self) -> Self {
        match self {
            OscillatorType::Sine => OscillatorType::Triangle,
            OscillatorType::Triangle => OscillatorType::Sawtooth,
            OscillatorType::Sawtooth => OscillatorType::Square,
            OscillatorType::Square => OscillatorType::Pulse,
            OscillatorType::Pulse => OscillatorType::PulseLFO,
            OscillatorType::PulseLFO => OscillatorType::WavetableHarmonics,
            OscillatorType::WavetableHarmonics => OscillatorType::WavetableOrgan,
            OscillatorType::WavetableOrgan => OscillatorType::WavetableVowel,
            OscillatorType::WavetableVowel => OscillatorType::Sine,
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
            OscillatorType::WavetableHarmonics => "Wavetable: Additive (harmonics 1,2,3,5)",
            OscillatorType::WavetableOrgan => "Wavetable: Organ (drawbar simulation)",
            OscillatorType::WavetableVowel => "Wavetable: Vowel 'ah' (formant peaks)",
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
    WavetableHarmonics(WavetableOscillator<SAMPLE_RATE>),
    WavetableOrgan(WavetableOscillator<SAMPLE_RATE>),
    WavetableVowel(WavetableOscillator<SAMPLE_RATE>),
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
            OscillatorType::WavetableHarmonics => {
                // Additive synthesis: fundamental + 2nd + 3rd + 5th harmonics
                // Creates a bright, harmonic-rich sound
                //
                // Alternative approach using Signal::iter():
                // let samples: Vec<f64> = SineOscillator::<SAMPLE_RATE>::new(...)
                //     .iter().take(1024).map(|s| /* process */).collect();
                // WavetableOscillator::from_samples(frequency, samples)

                OscillatorWrapper::WavetableHarmonics(
                    WavetableOscillator::<SAMPLE_RATE>::from_function(frequency, 1024, |phase| {
                        let p = phase * 2.0 * PI;
                        (p.sin()
                            + 0.5 * (2.0 * p).sin()
                            + 0.33 * (3.0 * p).sin()
                            + 0.2 * (5.0 * p).sin())
                            / 2.03 // Normalize
                    })
                    .with_interpolation(InterpolationMode::Linear),
                )
            }
            OscillatorType::WavetableOrgan => {
                // Hammond organ-style drawbar settings (888000000)
                // 16', 8', 5⅓' feet pipes
                OscillatorWrapper::WavetableOrgan(
                    WavetableOscillator::<SAMPLE_RATE>::from_function(frequency, 2048, |phase| {
                        let p = phase * 2.0 * PI;
                        (0.8 * (0.5 * p).sin() + // 16' (sub-octave)
                         0.8 * p.sin() +          // 8' (fundamental)
                         0.8 * (1.5 * p).sin())   // 5⅓' (3rd harmonic)
                            / 2.4 // Normalize
                    })
                    .with_interpolation(InterpolationMode::Cubic),
                )
            }
            OscillatorType::WavetableVowel => {
                // Vowel formant simulation (approximating 'ah' sound)
                // Demonstrates using Signal::iter() to generate wavetables from oscillators

                // Create multiple oscillators for different formants
                let mut f0 = SineOscillator::<SAMPLE_RATE>::new(1.0); // Fundamental
                let mut f1_a = SineOscillator::<SAMPLE_RATE>::new(2.0);
                let mut f1_b = SineOscillator::<SAMPLE_RATE>::new(3.0);
                let mut f2_a = SineOscillator::<SAMPLE_RATE>::new(6.0);
                let mut f2_b = SineOscillator::<SAMPLE_RATE>::new(8.0);
                let mut f3 = SineOscillator::<SAMPLE_RATE>::new(12.0);

                // Generate and combine using iterator API
                let samples: Vec<f64> = f0
                    .iter()
                    .zip(f1_a.iter())
                    .zip(f1_b.iter())
                    .zip(f2_a.iter())
                    .zip(f2_b.iter())
                    .zip(f3.iter())
                    .take(2048)
                    .map(|(((((s0, s1a), s1b), s2a), s2b), s3)| {
                        (s0 + 0.6 * s1a + 0.4 * s1b + 0.7 * s2a + 0.3 * s2b + 0.2 * s3) / 3.2
                    })
                    .collect();

                OscillatorWrapper::WavetableVowel(
                    WavetableOscillator::<SAMPLE_RATE>::from_samples(frequency, samples)
                        .with_interpolation(InterpolationMode::Cubic),
                )
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
            OscillatorWrapper::WavetableHarmonics(osc) => osc.next_sample(),
            OscillatorWrapper::WavetableOrgan(osc) => osc.next_sample(),
            OscillatorWrapper::WavetableVowel(osc) => osc.next_sample(),
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
