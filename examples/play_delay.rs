//! Interactive example for demonstrating delay effects.
//!
//! Press SPACE to cycle through delay types.
//! Press Q or ESC to quit.

mod common;

use anyhow::Result;
use common::{ExampleAudioState, KeyAction, KeyboardConfig, is_quit_key, run_interactive_example};
use crossterm::{
    ExecutableCommand,
    event::{KeyCode, KeyEvent},
};
use earworm::{Delay, Gain, Gate, Signal, SignalExt, SineOscillator, SquareOscillator};
use std::io::{Write, stdout};

const SAMPLE_RATE: u32 = 44100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DelayType {
    Slapback,
    ShortEcho,
    MediumEcho,
    LongEcho,
    ModulatedDelay,
    NoDry,
}

impl DelayType {
    fn next(self) -> Self {
        match self {
            DelayType::Slapback => DelayType::ShortEcho,
            DelayType::ShortEcho => DelayType::MediumEcho,
            DelayType::MediumEcho => DelayType::LongEcho,
            DelayType::LongEcho => DelayType::ModulatedDelay,
            DelayType::ModulatedDelay => DelayType::NoDry,
            DelayType::NoDry => DelayType::Slapback,
        }
    }

    fn name(self) -> &'static str {
        match self {
            DelayType::Slapback => "Slapback (75ms)",
            DelayType::ShortEcho => "Short Echo (200ms)",
            DelayType::MediumEcho => "Medium Echo (375ms)",
            DelayType::LongEcho => "Long Echo (500ms)",
            DelayType::ModulatedDelay => "Modulated (PWM)",
            DelayType::NoDry => "100% Wet (500ms)",
        }
    }
}

enum DelayWrapper {
    Slapback(Delay<SAMPLE_RATE, Gate<Gain<SineOscillator<SAMPLE_RATE>>>>),
    ShortEcho(Delay<SAMPLE_RATE, Gate<Gain<SineOscillator<SAMPLE_RATE>>>>),
    MediumEcho(Delay<SAMPLE_RATE, Gate<Gain<SineOscillator<SAMPLE_RATE>>>>),
    LongEcho(Delay<SAMPLE_RATE, Gate<Gain<SineOscillator<SAMPLE_RATE>>>>),
    ModulatedDelay(Delay<SAMPLE_RATE, Gate<Gain<SineOscillator<SAMPLE_RATE>>>>),
    NoDry(Delay<SAMPLE_RATE, Gate<Gain<SineOscillator<SAMPLE_RATE>>>>),
}

impl DelayWrapper {
    fn new(delay_type: DelayType, frequency: f64) -> Self {
        let sine = SineOscillator::new(frequency);
        let gained = sine.gain(0.5);
        let lfo = SquareOscillator::<SAMPLE_RATE>::new(2.0);
        let source = gained.gate(lfo);

        match delay_type {
            DelayType::Slapback => DelayWrapper::Slapback(Delay::slapback(source)),
            DelayType::ShortEcho => DelayWrapper::ShortEcho(Delay::echo(source, 0.2, 0.5)),
            DelayType::MediumEcho => DelayWrapper::MediumEcho(Delay::echo(source, 0.375, 0.6)),
            DelayType::LongEcho => DelayWrapper::LongEcho(Delay::echo(source, 0.5, 0.75)),
            DelayType::ModulatedDelay => {
                let mod_lfo = SineOscillator::<SAMPLE_RATE>::new(0.3);
                DelayWrapper::ModulatedDelay(Delay::new(source, 0.6, mod_lfo, 0.6, 0.5))
            }
            DelayType::NoDry => DelayWrapper::NoDry(Delay::new(source, 0.5, 0.5, 0.6, 1.0)),
        }
    }
}

impl Signal for DelayWrapper {
    fn next_sample(&mut self) -> f64 {
        match self {
            DelayWrapper::Slapback(d) => d.next_sample(),
            DelayWrapper::ShortEcho(d) => d.next_sample(),
            DelayWrapper::MediumEcho(d) => d.next_sample(),
            DelayWrapper::LongEcho(d) => d.next_sample(),
            DelayWrapper::ModulatedDelay(d) => d.next_sample(),
            DelayWrapper::NoDry(d) => d.next_sample(),
        }
    }
}

struct AudioState {
    delay: DelayWrapper,
    delay_type: DelayType,
    frequency: f64,
    fade_samples: usize,
}

impl AudioState {
    fn new(frequency: f64) -> Self {
        let delay_type = DelayType::Slapback;
        Self {
            delay: DelayWrapper::new(delay_type, frequency),
            delay_type,
            frequency,
            fade_samples: 0,
        }
    }

    fn switch_delay(&mut self) {
        self.delay_type = self.delay_type.next();
        self.delay = DelayWrapper::new(self.delay_type, self.frequency);
        self.fade_samples = (SAMPLE_RATE as f64 * 0.01) as usize;
    }
}

impl ExampleAudioState for AudioState {
    fn next_sample(&mut self) -> f64 {
        let sample = self.delay.next_sample();

        let sample = if self.fade_samples > 0 {
            let fade_start = (SAMPLE_RATE as f64 * 0.01) as usize;
            let fade_progress = 1.0 - (self.fade_samples as f64 / fade_start as f64);
            self.fade_samples -= 1;
            sample * fade_progress
        } else {
            sample
        };

        sample * 0.5
    }
}

fn draw_ui(delay_type: DelayType, frequency: f64) -> Result<()> {
    let mut stdout = stdout();
    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;
    stdout.execute(crossterm::cursor::MoveTo(0, 0))?;
    write!(
        stdout,
        "Playing: {} @ {:.0}Hz | SPACE=switch Q=quit",
        delay_type.name(),
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
            draw_ui(state.delay_type, state.frequency)
        },
        |state, key_event: &KeyEvent| match key_event.code {
            KeyCode::Char(' ') => {
                let mut state = state.lock().unwrap();
                state.switch_delay();
                let delay_type = state.delay_type;
                drop(state);
                draw_ui(delay_type, frequency)?;
                Ok(KeyAction::Continue)
            }
            code if is_quit_key(code) => Ok(KeyAction::Exit),
            _ => Ok(KeyAction::Continue),
        },
    )
}
