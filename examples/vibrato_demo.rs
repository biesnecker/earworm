//! Interactive vibrato effect demo.
//!
//! This example demonstrates the Vibrato effect with various presets.
//! Press SPACE to cycle through vibrato presets.
//! Press UP/DOWN to adjust rate, LEFT/RIGHT to adjust depth.
//! Press Q or ESC to quit.

mod common;

use anyhow::Result;
use common::{ExampleAudioState, KeyAction, KeyboardConfig, is_quit_key, run_interactive_example};
use crossterm::{
    ExecutableCommand,
    event::{KeyCode, KeyEvent, KeyEventKind},
};
use earworm::{Signal, SineOscillator, Vibrato};
use std::io::{Write, stdout};

const SAMPLE_RATE: u32 = 44100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VibratoPreset {
    Off,
    Subtle,
    Guitar,
    Wide,
    Custom,
}

impl VibratoPreset {
    fn next(self) -> Self {
        match self {
            VibratoPreset::Off => VibratoPreset::Subtle,
            VibratoPreset::Subtle => VibratoPreset::Guitar,
            VibratoPreset::Guitar => VibratoPreset::Wide,
            VibratoPreset::Wide => VibratoPreset::Custom,
            VibratoPreset::Custom => VibratoPreset::Off,
        }
    }

    fn name(self) -> &'static str {
        match self {
            VibratoPreset::Off => "Off",
            VibratoPreset::Subtle => "Subtle (5Hz, 15¢)",
            VibratoPreset::Guitar => "Guitar (5.5Hz, 30¢)",
            VibratoPreset::Wide => "Wide (6Hz, 50¢)",
            VibratoPreset::Custom => "Custom",
        }
    }
}

enum VibratoWrapper {
    Off(SineOscillator<SAMPLE_RATE>),
    Subtle(Vibrato<SAMPLE_RATE, SineOscillator<SAMPLE_RATE>>),
    Guitar(Vibrato<SAMPLE_RATE, SineOscillator<SAMPLE_RATE>>),
    Wide(Vibrato<SAMPLE_RATE, SineOscillator<SAMPLE_RATE>>),
    Custom(Vibrato<SAMPLE_RATE, SineOscillator<SAMPLE_RATE>>),
}

impl Signal for VibratoWrapper {
    fn next_sample(&mut self) -> f64 {
        match self {
            VibratoWrapper::Off(osc) => osc.next_sample() * 0.3,
            VibratoWrapper::Subtle(vib) => vib.next_sample() * 0.3,
            VibratoWrapper::Guitar(vib) => vib.next_sample() * 0.3,
            VibratoWrapper::Wide(vib) => vib.next_sample() * 0.3,
            VibratoWrapper::Custom(vib) => vib.next_sample() * 0.3,
        }
    }
}

struct AudioState {
    signal: VibratoWrapper,
    preset: VibratoPreset,
    frequency: f64,
    rate: f64,
    depth: f64,
}

impl AudioState {
    fn new(frequency: f64) -> Self {
        let preset = VibratoPreset::Off;
        Self {
            signal: Self::create_signal(preset, frequency, 5.0, 20.0),
            preset,
            frequency,
            rate: 5.0,
            depth: 20.0,
        }
    }

    fn create_oscillator(frequency: f64) -> SineOscillator<SAMPLE_RATE> {
        SineOscillator::<SAMPLE_RATE>::new(frequency)
    }

    fn create_signal(
        preset: VibratoPreset,
        frequency: f64,
        rate: f64,
        depth: f64,
    ) -> VibratoWrapper {
        let osc = Self::create_oscillator(frequency);
        match preset {
            VibratoPreset::Off => VibratoWrapper::Off(osc),
            VibratoPreset::Subtle => VibratoWrapper::Subtle(Vibrato::subtle(osc)),
            VibratoPreset::Guitar => VibratoWrapper::Guitar(Vibrato::guitar(osc)),
            VibratoPreset::Wide => VibratoWrapper::Wide(Vibrato::wide(osc)),
            VibratoPreset::Custom => VibratoWrapper::Custom(Vibrato::new(osc, rate, depth)),
        }
    }

    fn switch_preset(&mut self) {
        self.preset = self.preset.next();
        self.signal = Self::create_signal(self.preset, self.frequency, self.rate, self.depth);
    }

    fn adjust_rate(&mut self, delta: f64) {
        self.rate = (self.rate + delta).clamp(0.5, 20.0);
        if self.preset == VibratoPreset::Custom {
            self.signal = Self::create_signal(self.preset, self.frequency, self.rate, self.depth);
        }
    }

    fn adjust_depth(&mut self, delta: f64) {
        self.depth = (self.depth + delta).clamp(0.0, 100.0);
        if self.preset == VibratoPreset::Custom {
            self.signal = Self::create_signal(self.preset, self.frequency, self.rate, self.depth);
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

    let preset_str = state.preset.name();
    let params = if state.preset == VibratoPreset::Custom {
        format!(
            " | Rate: {:.1} Hz | Depth: {:.0} cents",
            state.rate, state.depth
        )
    } else {
        String::new()
    };

    write!(
        stdout,
        "Vibrato: {}{}  |  SPACE=switch  ↑↓=rate  ←→=depth  Q=quit",
        preset_str, params
    )?;

    stdout.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    run_interactive_example(
        AudioState::new(440.0), // A4
        KeyboardConfig::default(),
        |state| {
            let state = state.lock().unwrap();
            draw_ui(&state)
        },
        |state, key_event: &KeyEvent| {
            if !matches!(key_event.kind, KeyEventKind::Press) {
                return Ok(KeyAction::Continue);
            }

            match key_event.code {
                KeyCode::Char(' ') => {
                    state.lock().unwrap().switch_preset();
                }
                KeyCode::Up => {
                    state.lock().unwrap().adjust_rate(0.5);
                }
                KeyCode::Down => {
                    state.lock().unwrap().adjust_rate(-0.5);
                }
                KeyCode::Right => {
                    state.lock().unwrap().adjust_depth(5.0);
                }
                KeyCode::Left => {
                    state.lock().unwrap().adjust_depth(-5.0);
                }
                code if is_quit_key(code) => return Ok(KeyAction::Exit),
                _ => return Ok(KeyAction::Continue),
            }

            let state_guard = state.lock().unwrap();
            draw_ui(&state_guard)?;
            Ok(KeyAction::Continue)
        },
    )?;

    println!("\nGoodbye!");
    Ok(())
}
