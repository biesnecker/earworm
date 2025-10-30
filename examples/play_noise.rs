//! Interactive TUI for switching between noise types.
//!
//! Press SPACE to cycle through noise types.
//! Press Q or ESC to quit.

mod common;

use anyhow::Result;
use common::{ExampleAudioState, KeyAction, KeyboardConfig, is_quit_key, run_interactive_example};
use crossterm::{
    ExecutableCommand,
    event::{KeyCode, KeyEvent},
};
use earworm::{PinkNoise, Signal, WhiteNoise};
use rand::SeedableRng;
use std::io::{Write, stdout};

const SAMPLE_RATE: u32 = 44100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NoiseType {
    White,
    Pink,
}

impl NoiseType {
    fn next(self) -> Self {
        match self {
            NoiseType::White => NoiseType::Pink,
            NoiseType::Pink => NoiseType::White,
        }
    }

    fn name(self) -> &'static str {
        match self {
            NoiseType::White => "White Noise",
            NoiseType::Pink => "Pink Noise",
        }
    }
}

enum NoiseGenerator {
    White(WhiteNoise<SAMPLE_RATE, rand::rngs::StdRng>),
    Pink(PinkNoise<SAMPLE_RATE, rand::rngs::StdRng>),
}

impl NoiseGenerator {
    fn new(noise_type: NoiseType) -> Self {
        let rng = rand::rngs::StdRng::from_entropy();
        match noise_type {
            NoiseType::White => NoiseGenerator::White(WhiteNoise::with_rng(rng)),
            NoiseType::Pink => {
                let rng = rand::rngs::StdRng::from_entropy();
                NoiseGenerator::Pink(PinkNoise::with_rng(rng))
            }
        }
    }
}

impl Signal for NoiseGenerator {
    fn next_sample(&mut self) -> f64 {
        match self {
            NoiseGenerator::White(noise) => noise.next_sample(),
            NoiseGenerator::Pink(noise) => noise.next_sample(),
        }
    }
}

struct AudioState {
    generator: NoiseGenerator,
    noise_type: NoiseType,
}

impl AudioState {
    fn new() -> Self {
        let noise_type = NoiseType::White;
        Self {
            generator: NoiseGenerator::new(noise_type),
            noise_type,
        }
    }

    fn switch_noise_type(&mut self) {
        self.noise_type = self.noise_type.next();
        self.generator = NoiseGenerator::new(self.noise_type);
    }
}

impl ExampleAudioState for AudioState {
    fn next_sample(&mut self) -> f64 {
        self.generator.next_sample()
    }
}

fn draw_ui(noise_type: NoiseType) -> Result<()> {
    let mut stdout = stdout();
    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;
    stdout.execute(crossterm::cursor::MoveTo(0, 0))?;
    write!(
        stdout,
        "Playing: {} | SPACE=switch Q=quit",
        noise_type.name()
    )?;
    stdout.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    run_interactive_example(
        AudioState::new(),
        KeyboardConfig::default(),
        |state| {
            let noise_type = state.lock().unwrap().noise_type;
            draw_ui(noise_type)
        },
        |state, key_event: &KeyEvent| match key_event.code {
            KeyCode::Char(' ') => {
                let mut state = state.lock().unwrap();
                state.switch_noise_type();
                let noise_type = state.noise_type;
                drop(state);
                draw_ui(noise_type)?;
                Ok(KeyAction::Continue)
            }
            code if is_quit_key(code) => Ok(KeyAction::Exit),
            _ => Ok(KeyAction::Continue),
        },
    )
}
