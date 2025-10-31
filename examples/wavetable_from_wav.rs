//! Interactive example demonstrating loading wavetables from WAV files.
//!
//! Loads a vocal sample ("hey") and plays it as a wavetable oscillator.
//! Press SPACE to toggle playback. The sample plays at normal speed initially,
//! and you can adjust the pitch up or down with arrow keys.
//!
//! Controls:
//! - SPACE: Toggle playback on/off
//! - UP/DOWN arrows: Adjust pitch (±100 cents)
//! - LEFT/RIGHT arrows: Adjust pitch (±10 cents)
//! - R: Reset pitch to 0
//! - Q or ESC: Quit

mod common;

use anyhow::Result;
use common::{ExampleAudioState, KeyAction, KeyboardConfig, is_quit_key, run_interactive_example};
use crossterm::{
    ExecutableCommand,
    event::{KeyCode, KeyEvent},
};
use earworm::{Gain, InterpolationMode, Pitched, Signal, WavetableOscillator};
use std::io::{Write, stdout};

const SAMPLE_RATE: u32 = 44100;

struct AudioState {
    oscillator: Gain<WavetableOscillator<SAMPLE_RATE>>,
    playing: bool,
    pitch_offset_cents: i32, // Pitch offset in cents (100 cents = 1 semitone)
    base_frequency: f64,     // Frequency for normal playback (no pitch shift)
    table_size: usize,
}

impl AudioState {
    fn new() -> Result<Self> {
        // Load the vocal sample
        let mut osc = WavetableOscillator::<SAMPLE_RATE>::from_wav_file(
            SAMPLE_RATE as f64,
            "resources/short-male-vox-hey.wav",
        )
        .map_err(|e| anyhow::anyhow!("Failed to load WAV file: {}", e))?
        .with_interpolation(InterpolationMode::Cubic);

        let table_size = osc.table_size();

        // Calculate base frequency for normal playback (1 loop per second would be SAMPLE_RATE / table_size Hz)
        // For normal playback speed, we want the wavetable to play at its original rate
        let base_frequency = SAMPLE_RATE as f64 / table_size as f64;

        // Set initial frequency
        osc.set_frequency(base_frequency);

        Ok(Self {
            oscillator: Gain {
                source: osc,
                gain: 0.0.into(), // Start muted
            },
            playing: false,
            pitch_offset_cents: 0,
            base_frequency,
            table_size,
        })
    }

    fn toggle_playback(&mut self) {
        self.playing = !self.playing;
        self.oscillator.gain = if self.playing { 0.5.into() } else { 0.0.into() };
    }

    fn adjust_pitch(&mut self, cents: i32) {
        self.pitch_offset_cents += cents;
        // Clamp to ±2400 cents (±2 octaves)
        self.pitch_offset_cents = self.pitch_offset_cents.clamp(-2400, 2400);
        self.update_frequency();
    }

    fn reset_pitch(&mut self) {
        self.pitch_offset_cents = 0;
        self.update_frequency();
    }

    fn update_frequency(&mut self) {
        // Convert cents to frequency multiplier
        // freq = base_freq * 2^(cents/1200)
        let multiplier = 2.0_f64.powf(self.pitch_offset_cents as f64 / 1200.0);
        let frequency = self.base_frequency * multiplier;
        self.oscillator.source.set_frequency(frequency);
    }
}

impl ExampleAudioState for AudioState {
    fn next_sample(&mut self) -> f64 {
        self.oscillator.next_sample()
    }
}

fn draw_ui(state: &AudioState) -> Result<()> {
    let mut stdout = stdout();
    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;
    stdout.execute(crossterm::cursor::MoveTo(0, 0))?;

    write!(stdout, "=== Wavetable from WAV File ===\r\n")?;
    write!(stdout, "\r\n")?;
    write!(
        stdout,
        "File: resources/short-male-vox-hey.wav ({} samples)\r\n",
        state.table_size
    )?;
    write!(
        stdout,
        "Duration: {:.2} seconds\r\n",
        state.table_size as f64 / SAMPLE_RATE as f64
    )?;
    write!(
        stdout,
        "Interpolation: {:?}\r\n",
        state.oscillator.source.interpolation()
    )?;
    write!(stdout, "\r\n")?;
    write!(
        stdout,
        "Status: {}\r\n",
        if state.playing {
            "▶ PLAYING"
        } else {
            "⏸ PAUSED"
        }
    )?;
    write!(
        stdout,
        "Pitch: {:+} cents ({:+.1} semitones)\r\n",
        state.pitch_offset_cents,
        state.pitch_offset_cents as f64 / 100.0
    )?;
    let multiplier = 2.0_f64.powf(state.pitch_offset_cents as f64 / 1200.0);
    write!(stdout, "Playback speed: {:.2}x\r\n", multiplier)?;
    write!(stdout, "\r\n")?;
    write!(stdout, "Controls:\r\n")?;
    write!(stdout, "  SPACE       - Toggle playback\r\n")?;
    write!(stdout, "  UP/DOWN     - Adjust pitch (±100 cents)\r\n")?;
    write!(stdout, "  LEFT/RIGHT  - Adjust pitch (±10 cents)\r\n")?;
    write!(stdout, "  R           - Reset pitch to 0\r\n")?;
    write!(stdout, "  Q/ESC       - Quit\r\n")?;

    stdout.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let audio_state = AudioState::new()?;

    run_interactive_example(
        audio_state,
        KeyboardConfig::default(),
        |state| {
            let state = state.lock().unwrap();
            draw_ui(&state)
        },
        |state, key_event: &KeyEvent| {
            let result = match key_event.code {
                KeyCode::Char(' ') => {
                    let mut s = state.lock().unwrap();
                    s.toggle_playback();
                    drop(s);
                    Ok(KeyAction::Continue)
                }
                KeyCode::Up => {
                    let mut s = state.lock().unwrap();
                    s.adjust_pitch(100);
                    drop(s);
                    Ok(KeyAction::Continue)
                }
                KeyCode::Down => {
                    let mut s = state.lock().unwrap();
                    s.adjust_pitch(-100);
                    drop(s);
                    Ok(KeyAction::Continue)
                }
                KeyCode::Right => {
                    let mut s = state.lock().unwrap();
                    s.adjust_pitch(10);
                    drop(s);
                    Ok(KeyAction::Continue)
                }
                KeyCode::Left => {
                    let mut s = state.lock().unwrap();
                    s.adjust_pitch(-10);
                    drop(s);
                    Ok(KeyAction::Continue)
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    let mut s = state.lock().unwrap();
                    s.reset_pitch();
                    drop(s);
                    Ok(KeyAction::Continue)
                }
                code if is_quit_key(code) => Ok(KeyAction::Exit),
                _ => Ok(KeyAction::Continue),
            };

            // Redraw UI after any key press
            if result.is_ok() {
                let s = state.lock().unwrap();
                let _ = draw_ui(&s);
            }

            result
        },
    )
}
