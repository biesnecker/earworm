//! Interactive Tremolo effect demo.
//!
//! Press SPACE to toggle tremolo on/off.
//! Press UP/DOWN to adjust rate, LEFT/RIGHT to adjust depth.
//! Press Q or ESC to quit.

mod common;

use anyhow::Result;
use common::{ExampleAudioState, KeyAction, KeyboardConfig, is_quit_key, run_interactive_example};
use crossterm::{
    ExecutableCommand,
    event::{KeyCode, KeyEvent},
};
use earworm::{Signal, SineOscillator};
use std::io::{Write, stdout};

const SAMPLE_RATE: u32 = 44100;

struct AudioState {
    oscillator: SineOscillator<SAMPLE_RATE>,
    lfo: SineOscillator<SAMPLE_RATE>,
    rate: f64,
    depth: f64,
    tremolo_enabled: bool,
}

impl AudioState {
    fn new(frequency: f64) -> Self {
        let rate = 5.0;
        Self {
            oscillator: SineOscillator::new(frequency),
            lfo: SineOscillator::new(rate),
            rate,
            depth: 0.5,
            tremolo_enabled: false,
        }
    }

    fn toggle_tremolo(&mut self) {
        self.tremolo_enabled = !self.tremolo_enabled;
    }

    fn adjust_rate(&mut self, delta: f64) {
        self.rate = (self.rate + delta).clamp(0.5, 20.0);
        self.lfo = SineOscillator::new(self.rate);
    }

    fn adjust_depth(&mut self, delta: f64) {
        self.depth = (self.depth + delta).clamp(0.0, 1.0);
    }
}

impl ExampleAudioState for AudioState {
    fn next_sample(&mut self) -> f64 {
        let audio = self.oscillator.next_sample();

        if self.tremolo_enabled {
            let mod_value = self.lfo.next_sample();
            let gain = 1.0 + self.depth / 2.0 * (mod_value - 1.0);
            audio * gain * 0.3
        } else {
            self.lfo.next_sample();
            audio * 0.3
        }
    }
}

fn draw_ui(state: &AudioState) -> Result<()> {
    let mut stdout = stdout();
    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;
    stdout.execute(crossterm::cursor::MoveTo(0, 0))?;

    let status = if state.tremolo_enabled { "ON" } else { "OFF" };
    write!(
        stdout,
        "Tremolo: {} | Rate: {:.1}Hz | Depth: {:.2} | SPACE=toggle ↑↓=rate ←→=depth Q=quit",
        status, state.rate, state.depth
    )?;
    stdout.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    run_interactive_example(
        AudioState::new(440.0),
        KeyboardConfig::default(),
        |state| {
            let state = state.lock().unwrap();
            draw_ui(&state)
        },
        |state, key_event: &KeyEvent| {
            match key_event.code {
                KeyCode::Char(' ') => {
                    state.lock().unwrap().toggle_tremolo();
                }
                KeyCode::Up => {
                    state.lock().unwrap().adjust_rate(0.5);
                }
                KeyCode::Down => {
                    state.lock().unwrap().adjust_rate(-0.5);
                }
                KeyCode::Right => {
                    state.lock().unwrap().adjust_depth(0.05);
                }
                KeyCode::Left => {
                    state.lock().unwrap().adjust_depth(-0.05);
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
