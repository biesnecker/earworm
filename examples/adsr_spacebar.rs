//! Interactive ADSR envelope example.
//!
//! Press and hold SPACE to play a note with an ADSR envelope.
//! The envelope goes through Attack → Decay → Sustain while held.
//! Release SPACE to trigger the Release phase.
//! Press Q or ESC to quit.

mod common;

use anyhow::Result;
use common::{ExampleAudioState, KeyAction, KeyboardConfig, is_quit_key, run_interactive_example};
use crossterm::{
    ExecutableCommand,
    event::{KeyCode, KeyEvent, KeyEventKind},
};
use earworm::{ADSR, Curve, Signal, SineOscillator};
use std::io::{Write, stdout};

const SAMPLE_RATE: u32 = 44100;

struct AudioState {
    oscillator: SineOscillator<SAMPLE_RATE>,
    envelope: ADSR,
    space_pressed: bool,
}

impl AudioState {
    fn new(frequency: f64) -> Self {
        let oscillator = SineOscillator::new(frequency);

        let envelope = ADSR::new(
            0.05, // 50ms attack
            0.1,  // 100ms decay
            0.7,  // 70% sustain level
            0.3,  // 300ms release
            SAMPLE_RATE as f64,
        )
        .with_attack_curve(Curve::Exponential(2.0))
        .with_decay_curve(Curve::Exponential(2.0))
        .with_release_curve(Curve::Exponential(3.0));

        Self {
            oscillator,
            envelope,
            space_pressed: false,
        }
    }

    fn handle_key_event(&mut self, code: KeyCode, kind: KeyEventKind) {
        if let KeyCode::Char(' ') = code {
            if matches!(kind, KeyEventKind::Press | KeyEventKind::Repeat) {
                if !self.space_pressed {
                    self.space_pressed = true;
                    self.envelope.note_on();
                }
            } else if matches!(kind, KeyEventKind::Release) {
                self.space_pressed = false;
                self.envelope.note_off();
            }
        }
    }

    fn is_active(&self) -> bool {
        self.envelope.is_active()
    }
}

impl ExampleAudioState for AudioState {
    fn next_sample(&mut self) -> f64 {
        let oscillator_sample = self.oscillator.next_sample();
        let envelope_level = self.envelope.next_sample();
        oscillator_sample * envelope_level * 0.3
    }
}

fn draw_ui(is_active: bool) -> Result<()> {
    let mut stdout = stdout();
    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;
    stdout.execute(crossterm::cursor::MoveTo(0, 0))?;
    write!(
        stdout,
        "ADSR Envelope: {} | HOLD SPACE=play  RELEASE=stop  Q=quit",
        if is_active { "PLAYING" } else { "IDLE   " }
    )?;
    stdout.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    run_interactive_example(
        AudioState::new(440.0),
        KeyboardConfig::with_enhancements(),
        |_state| draw_ui(false),
        |state, key_event: &KeyEvent| {
            let mut state_guard = state.lock().unwrap();

            if is_quit_key(key_event.code) && matches!(key_event.kind, KeyEventKind::Press) {
                return Ok(KeyAction::Exit);
            }

            state_guard.handle_key_event(key_event.code, key_event.kind);
            let is_active = state_guard.is_active();
            drop(state_guard);

            draw_ui(is_active)?;
            Ok(KeyAction::Continue)
        },
    )?;

    println!("\nGoodbye!");
    Ok(())
}
