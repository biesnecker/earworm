//! Interactive monophonic synthesizer demo using the Voice struct.
//!
//! This example demonstrates:
//! - Voice with ADSR envelope control
//! - Note on/off behavior
//! - Keyboard-to-MIDI mapping
//! - Real-time envelope state visualization
//!
//! ## Controls
//!
//! **Play notes:**
//! - Bottom row (A-L): White keys (C4-D5)
//! - Top row (W-O, T-Y-U, P): Black keys (sharps)
//!
//! **Other:**
//! - Q or ESC: Quit
//!
//! The keyboard layout mimics a piano:
//! ```text
//! W E   T Y U   O P     (black keys/sharps)
//!  ↓ ↓   ↓ ↓ ↓   ↓ ↓
//! A S D F G H J K L     (white keys)
//! C D E F G A B C D     (note names)
//! ```

mod common;

use anyhow::Result;
use common::{
    ExampleAudioState, KeyAction, KeyboardConfig, draw_keyboard_ui, is_quit_key, key_to_midi_note,
    midi_note_to_name, run_interactive_example,
};
use crossterm::event::{KeyEvent, KeyEventKind};
use earworm::{ADSR, Signal, SineOscillator, music::Voice};

const SAMPLE_RATE: u32 = 44100;

struct VoiceDemoState {
    voice: Voice<SAMPLE_RATE, SineOscillator<SAMPLE_RATE>, ADSR>,
    current_note: Option<u8>,
}

impl VoiceDemoState {
    fn new() -> Self {
        let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
        let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
        let voice = Voice::new(osc, env);

        Self {
            voice,
            current_note: None,
        }
    }

    fn note_on(&mut self, midi_note: u8) {
        self.current_note = Some(midi_note);
        self.voice.note_on(midi_note, 0.8);
    }

    fn note_off(&mut self) {
        self.voice.note_off();
        self.current_note = None;
    }

    fn is_active(&self) -> bool {
        self.voice.is_active()
    }
}

impl ExampleAudioState for VoiceDemoState {
    fn next_sample(&mut self) -> f64 {
        self.voice.next_sample() * 0.3 // Reduce volume
    }

    fn output_info(&self) -> Option<String> {
        if let Some(note) = self.current_note {
            let note_name = midi_note_to_name(note);
            let freq = 440.0 * 2.0_f64.powf((note as f64 - 69.0) / 12.0);
            let status = if self.is_active() {
                "PLAYING"
            } else {
                "RELEASED"
            };
            Some(format!(
                "Note: {} ({:.1} Hz) | Status: {}",
                note_name, freq, status
            ))
        } else if self.is_active() {
            Some("Status: RELEASING...".to_string())
        } else {
            Some("Status: IDLE | Press keys to play".to_string())
        }
    }
}

fn draw_ui() -> Result<()> {
    draw_keyboard_ui("Voice Demo - Monophonic Synthesizer", None)
}

fn main() -> Result<()> {
    run_interactive_example(
        VoiceDemoState::new(),
        KeyboardConfig::with_enhancements(),
        |_state| draw_ui(),
        |state, key_event: &KeyEvent| {
            match key_event.code {
                code if is_quit_key(code) => return Ok(KeyAction::Exit),
                _ => {}
            }

            // Handle note on/off based on key press/release
            match key_event.kind {
                KeyEventKind::Press => {
                    if let Some(midi_note) = key_to_midi_note(key_event.code) {
                        let mut s = state.lock().unwrap();
                        // Only trigger note_on if this is a new note (prevent key repeat retriggering)
                        if s.current_note != Some(midi_note) {
                            s.note_on(midi_note);
                        }
                    }
                }
                KeyEventKind::Release => {
                    if let Some(released_note) = key_to_midi_note(key_event.code) {
                        let mut s = state.lock().unwrap();
                        // Only trigger note_off if the released key matches the currently playing note
                        if s.current_note == Some(released_note) {
                            s.note_off();
                        }
                    }
                }
                _ => {}
            }

            Ok(KeyAction::Continue)
        },
    )
}
