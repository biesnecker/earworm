//! Ring modulation example using signal combinators.
//!
//! This example demonstrates ring modulation by multiplying a carrier wave
//! with a modulator wave to create interesting harmonic effects. It also
//! showcases amplitude modulation (tremolo) with an LFO.
//!
//! Press SPACE to cycle through different modulation effects.
//! Press Q or ESC to quit.

mod common;

use anyhow::Result;
use common::{ExampleAudioState, KeyAction, KeyboardConfig, is_quit_key, run_interactive_example};
use crossterm::{
    ExecutableCommand,
    event::{KeyCode, KeyEvent},
};
use earworm::{Signal, SignalExt, SineOscillator};
use std::io::{Write, stdout};

const SAMPLE_RATE: u32 = 44100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModulationType {
    None,
    Tremolo,
    RingLow,
    RingHarmonic,
    RingInharmonic,
}

impl ModulationType {
    fn next(self) -> Self {
        match self {
            ModulationType::None => ModulationType::Tremolo,
            ModulationType::Tremolo => ModulationType::RingLow,
            ModulationType::RingLow => ModulationType::RingHarmonic,
            ModulationType::RingHarmonic => ModulationType::RingInharmonic,
            ModulationType::RingInharmonic => ModulationType::None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            ModulationType::None => "No Modulation",
            ModulationType::Tremolo => "Tremolo (6 Hz LFO)",
            ModulationType::RingLow => "Ring Mod (30 Hz)",
            ModulationType::RingHarmonic => "Ring Mod (660 Hz - 3:2 ratio)",
            ModulationType::RingInharmonic => "Ring Mod (573 Hz - inharmonic)",
        }
    }
}

struct AudioState {
    carrier_freq: f64,
    mod_type: ModulationType,
    signal: Box<dyn Signal + Send>,
    fade_samples: usize,
}

impl AudioState {
    fn new(carrier_freq: f64) -> Self {
        let mod_type = ModulationType::None;
        let signal = Self::create_signal(mod_type, carrier_freq);
        Self {
            carrier_freq,
            mod_type,
            signal,
            fade_samples: 0,
        }
    }

    fn create_signal(mod_type: ModulationType, carrier_freq: f64) -> Box<dyn Signal + Send> {
        let carrier = SineOscillator::<SAMPLE_RATE>::new(carrier_freq);

        match mod_type {
            ModulationType::None => Box::new(carrier.gain(0.3)),
            ModulationType::Tremolo => {
                let lfo = SineOscillator::<SAMPLE_RATE>::new(6.0);
                Box::new(carrier.multiply(lfo.offset(1.0).gain(0.5)).gain(0.3))
            }
            ModulationType::RingLow => {
                let modulator = SineOscillator::<SAMPLE_RATE>::new(30.0);
                Box::new(carrier.multiply(modulator).gain(0.3))
            }
            ModulationType::RingHarmonic => {
                let modulator = SineOscillator::<SAMPLE_RATE>::new(carrier_freq * 1.5);
                Box::new(carrier.multiply(modulator).gain(0.3))
            }
            ModulationType::RingInharmonic => {
                let modulator = SineOscillator::<SAMPLE_RATE>::new(573.0);
                Box::new(carrier.multiply(modulator).gain(0.3))
            }
        }
    }

    fn switch_modulation(&mut self) {
        self.mod_type = self.mod_type.next();
        self.signal = Self::create_signal(self.mod_type, self.carrier_freq);
        self.fade_samples = (SAMPLE_RATE as f64 * 0.002) as usize;
    }
}

impl ExampleAudioState for AudioState {
    fn next_sample(&mut self) -> f64 {
        let sample = self.signal.next_sample();

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

fn draw_ui(mod_type: ModulationType) -> Result<()> {
    let mut stdout = stdout();
    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;
    stdout.execute(crossterm::cursor::MoveTo(0, 0))?;
    write!(stdout, "Playing: {} | SPACE=switch Q=quit", mod_type.name())?;
    stdout.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    run_interactive_example(
        AudioState::new(440.0),
        KeyboardConfig::default(),
        |state| draw_ui(state.lock().unwrap().mod_type),
        |state, key_event: &KeyEvent| match key_event.code {
            KeyCode::Char(' ') => {
                let mut state = state.lock().unwrap();
                state.switch_modulation();
                let mod_type = state.mod_type;
                drop(state);
                draw_ui(mod_type)?;
                Ok(KeyAction::Continue)
            }
            code if is_quit_key(code) => Ok(KeyAction::Exit),
            _ => Ok(KeyAction::Continue),
        },
    )
}
