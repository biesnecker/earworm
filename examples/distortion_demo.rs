//! Interactive distortion effect demo.
//!
//! Press SPACE to cycle through distortion types.
//! Press UP/DOWN to adjust drive, LEFT/RIGHT to adjust mix.
//! Press Q or ESC to quit.

mod common;

use anyhow::Result;
use common::{ExampleAudioState, KeyAction, KeyboardConfig, is_quit_key, run_interactive_example};
use crossterm::{
    ExecutableCommand,
    event::{KeyCode, KeyEvent},
};
use earworm::{Distortion, Signal, TriangleOscillator};
use std::io::{Write, stdout};

const SAMPLE_RATE: u32 = 44100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DistortionType {
    Clean,
    Overdrive,
    Classic,
    Fuzz,
    Custom,
}

impl DistortionType {
    fn next(self) -> Self {
        match self {
            DistortionType::Clean => DistortionType::Overdrive,
            DistortionType::Overdrive => DistortionType::Classic,
            DistortionType::Classic => DistortionType::Fuzz,
            DistortionType::Fuzz => DistortionType::Custom,
            DistortionType::Custom => DistortionType::Clean,
        }
    }

    fn name(self) -> &'static str {
        match self {
            DistortionType::Clean => "Clean",
            DistortionType::Overdrive => "Overdrive",
            DistortionType::Classic => "Classic",
            DistortionType::Fuzz => "Fuzz",
            DistortionType::Custom => "Custom",
        }
    }
}

enum DistortionWrapper {
    Clean(TriangleOscillator<SAMPLE_RATE>),
    Overdrive(Distortion<SAMPLE_RATE, TriangleOscillator<SAMPLE_RATE>>),
    Classic(Distortion<SAMPLE_RATE, TriangleOscillator<SAMPLE_RATE>>),
    Fuzz(Distortion<SAMPLE_RATE, TriangleOscillator<SAMPLE_RATE>>),
    Custom(Distortion<SAMPLE_RATE, TriangleOscillator<SAMPLE_RATE>>),
}

impl Signal for DistortionWrapper {
    fn next_sample(&mut self) -> f64 {
        match self {
            DistortionWrapper::Clean(osc) => osc.next_sample() * 0.3,
            DistortionWrapper::Overdrive(d) => d.next_sample() * 0.4,
            DistortionWrapper::Classic(d) => d.next_sample() * 0.4,
            DistortionWrapper::Fuzz(d) => d.next_sample() * 0.4,
            DistortionWrapper::Custom(d) => d.next_sample() * 0.4,
        }
    }
}

struct AudioState {
    signal: DistortionWrapper,
    dist_type: DistortionType,
    frequency: f64,
    drive: f64,
    mix: f64,
}

impl AudioState {
    fn new(frequency: f64) -> Self {
        let dist_type = DistortionType::Clean;
        Self {
            signal: Self::create_signal(dist_type, frequency, 5.0, 0.7),
            dist_type,
            frequency,
            drive: 5.0,
            mix: 0.7,
        }
    }

    fn create_signal(
        dist_type: DistortionType,
        frequency: f64,
        drive: f64,
        mix: f64,
    ) -> DistortionWrapper {
        let osc = TriangleOscillator::new(frequency);
        match dist_type {
            DistortionType::Clean => DistortionWrapper::Clean(osc),
            DistortionType::Overdrive => DistortionWrapper::Overdrive(Distortion::overdrive(osc)),
            DistortionType::Classic => DistortionWrapper::Classic(Distortion::classic(osc)),
            DistortionType::Fuzz => DistortionWrapper::Fuzz(Distortion::fuzz(osc)),
            DistortionType::Custom => DistortionWrapper::Custom(Distortion::new(osc, drive, mix)),
        }
    }

    fn switch_type(&mut self) {
        self.dist_type = self.dist_type.next();
        self.signal = Self::create_signal(self.dist_type, self.frequency, self.drive, self.mix);
    }

    fn adjust_drive(&mut self, delta: f64) {
        self.drive = (self.drive + delta).clamp(1.0, 50.0);
        if self.dist_type == DistortionType::Custom {
            self.signal = Self::create_signal(self.dist_type, self.frequency, self.drive, self.mix);
        }
    }

    fn adjust_mix(&mut self, delta: f64) {
        self.mix = (self.mix + delta).clamp(0.0, 1.0);
        if self.dist_type == DistortionType::Custom {
            self.signal = Self::create_signal(self.dist_type, self.frequency, self.drive, self.mix);
        }
    }
}

impl ExampleAudioState for AudioState {
    fn next_sample(&mut self) -> f64 {
        self.signal.next_sample()
    }
}

fn draw_ui(state: &AudioState) -> Result<()> {
    let mut stdout = stdout();
    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;
    stdout.execute(crossterm::cursor::MoveTo(0, 0))?;

    let type_str = state.dist_type.name();
    let params = if state.dist_type == DistortionType::Custom {
        format!(" | Drive: {:.1} | Mix: {:.2}", state.drive, state.mix)
    } else {
        String::new()
    };

    write!(
        stdout,
        "Distortion: {}{} | SPACE=switch ↑↓=drive ←→=mix Q=quit",
        type_str, params
    )?;

    stdout.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    run_interactive_example(
        AudioState::new(220.0), // A3 - lower frequency shows distortion better
        KeyboardConfig::default(),
        |state| {
            let state = state.lock().unwrap();
            draw_ui(&state)
        },
        |state, key_event: &KeyEvent| {
            match key_event.code {
                KeyCode::Char(' ') => {
                    state.lock().unwrap().switch_type();
                }
                KeyCode::Up => {
                    state.lock().unwrap().adjust_drive(1.0);
                }
                KeyCode::Down => {
                    state.lock().unwrap().adjust_drive(-1.0);
                }
                KeyCode::Right => {
                    state.lock().unwrap().adjust_mix(0.05);
                }
                KeyCode::Left => {
                    state.lock().unwrap().adjust_mix(-0.05);
                }
                code if is_quit_key(code) => return Ok(KeyAction::Exit),
                _ => return Ok(KeyAction::Continue),
            }

            let state_guard = state.lock().unwrap();
            draw_ui(&state_guard)?;
            Ok(KeyAction::Continue)
        },
    )
}
