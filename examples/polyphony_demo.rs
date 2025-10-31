//! Interactive polyphonic synthesizer demo using VoiceAllocator.
//!
//! This example demonstrates:
//! - Playing multiple notes simultaneously (polyphony)
//! - Voice allocation and stealing
//! - Visual display of active voices
//! - Dynamic voice count adjustment
//!
//! ## Controls
//!
//! **Play notes:**
//! - Bottom row (A-L): White keys (C4-D5)
//! - Top row (W-O, T-Y-U, P): Black keys (sharps)
//!
//! **Voice count:**
//! - 1-9: Set voice count (try setting to 4 then playing a 5-note chord!)
//!
//! **Other:**
//! - Q or ESC: Quit
//!
//! ## Voice Stealing
//!
//! When you trigger more notes than available voices, the oldest voice
//! will be stolen to play the new note. Try setting voices to 4 and
//! playing all keys in a row to hear the stealing in action!

mod common;

use anyhow::Result;
use common::{
    ExampleAudioState, KeyAction, KeyboardConfig, draw_keyboard_ui, is_quit_key, key_to_midi_note,
    midi_note_to_name, run_interactive_example,
};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use earworm::{ADSR, Signal, SineOscillator, music::VoiceAllocator};
use std::sync::{Arc, Mutex};

const SAMPLE_RATE: u32 = 44100;

// We'll use different const voice counts
// Start with 8 voices
type Allocator8 = VoiceAllocator<SAMPLE_RATE, 8, SineOscillator<SAMPLE_RATE>, ADSR>;
type Allocator4 = VoiceAllocator<SAMPLE_RATE, 4, SineOscillator<SAMPLE_RATE>, ADSR>;
type Allocator2 = VoiceAllocator<SAMPLE_RATE, 2, SineOscillator<SAMPLE_RATE>, ADSR>;
type Allocator1 = VoiceAllocator<SAMPLE_RATE, 1, SineOscillator<SAMPLE_RATE>, ADSR>;

enum PolyAllocator {
    Voices1(Box<Allocator1>),
    Voices2(Box<Allocator2>),
    Voices4(Box<Allocator4>),
    Voices8(Box<Allocator8>),
}

impl PolyAllocator {
    fn new(voice_count: usize) -> Self {
        match voice_count {
            1 => PolyAllocator::Voices1(Box::new(VoiceAllocator::new(|| {
                let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
                let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
                (osc, env)
            }))),
            2 => PolyAllocator::Voices2(Box::new(VoiceAllocator::new(|| {
                let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
                let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
                (osc, env)
            }))),
            4 => PolyAllocator::Voices4(Box::new(VoiceAllocator::new(|| {
                let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
                let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
                (osc, env)
            }))),
            _ => PolyAllocator::Voices8(Box::new(VoiceAllocator::new(|| {
                let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
                let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
                (osc, env)
            }))),
        }
    }

    fn note_on(&mut self, note: u8, velocity: f64) {
        match self {
            PolyAllocator::Voices1(a) => a.note_on(note, velocity),
            PolyAllocator::Voices2(a) => a.note_on(note, velocity),
            PolyAllocator::Voices4(a) => a.note_on(note, velocity),
            PolyAllocator::Voices8(a) => a.note_on(note, velocity),
        }
    }

    fn note_off(&mut self, note: u8) {
        match self {
            PolyAllocator::Voices1(a) => a.note_off(note),
            PolyAllocator::Voices2(a) => a.note_off(note),
            PolyAllocator::Voices4(a) => a.note_off(note),
            PolyAllocator::Voices8(a) => a.note_off(note),
        }
    }

    fn active_voice_count(&self) -> usize {
        match self {
            PolyAllocator::Voices1(a) => a.active_voice_count(),
            PolyAllocator::Voices2(a) => a.active_voice_count(),
            PolyAllocator::Voices4(a) => a.active_voice_count(),
            PolyAllocator::Voices8(a) => a.active_voice_count(),
        }
    }

    fn max_voices(&self) -> usize {
        match self {
            PolyAllocator::Voices1(_) => 1,
            PolyAllocator::Voices2(_) => 2,
            PolyAllocator::Voices4(_) => 4,
            PolyAllocator::Voices8(_) => 8,
        }
    }

    fn next_sample(&mut self) -> f64 {
        match self {
            PolyAllocator::Voices1(a) => a.next_sample(),
            PolyAllocator::Voices2(a) => a.next_sample(),
            PolyAllocator::Voices4(a) => a.next_sample(),
            PolyAllocator::Voices8(a) => a.next_sample(),
        }
    }
}

struct PolyphonyDemoState {
    allocator: PolyAllocator,
    active_notes: Vec<u8>, // Track which notes are currently pressed
}

impl PolyphonyDemoState {
    fn new(voice_count: usize) -> Self {
        Self {
            allocator: PolyAllocator::new(voice_count),
            active_notes: Vec::new(),
        }
    }

    fn set_voice_count(&mut self, count: usize) {
        self.allocator = PolyAllocator::new(count);
        self.active_notes.clear();
    }
}

impl ExampleAudioState for PolyphonyDemoState {
    fn next_sample(&mut self) -> f64 {
        self.allocator.next_sample() * 0.3 // Reduce volume
    }

    fn output_info(&self) -> Option<String> {
        let max_voices = self.allocator.max_voices();
        let active_voices = self.allocator.active_voice_count();

        let notes_str = if self.active_notes.is_empty() {
            "No notes playing".to_string()
        } else {
            self.active_notes
                .iter()
                .map(|&n| midi_note_to_name(n))
                .collect::<Vec<_>>()
                .join(", ")
        };

        Some(format!(
            "Voices: {}/{} active | Notes: {} | Press 1-9 to change voice count",
            active_voices, max_voices, notes_str
        ))
    }
}

fn draw_ui() -> Result<()> {
    draw_keyboard_ui(
        "Polyphony Demo - Multi-Voice Synthesizer",
        Some("1-9 = Set voice count | Try 4 voices + 5-note chord!"),
    )
}

fn handle_key(state: &Arc<Mutex<PolyphonyDemoState>>, key_event: &KeyEvent) -> Result<KeyAction> {
    match key_event.code {
        code if is_quit_key(code) => return Ok(KeyAction::Exit),
        KeyCode::Char(c @ '1'..='9') if key_event.kind == KeyEventKind::Press => {
            let count = c.to_digit(10).unwrap() as usize;
            let mut s = state.lock().unwrap();
            s.set_voice_count(count);
            return Ok(KeyAction::Continue);
        }
        _ => {}
    }

    // Handle note on/off based on key press/release
    match key_event.kind {
        KeyEventKind::Press => {
            if let Some(midi_note) = key_to_midi_note(key_event.code) {
                let mut s = state.lock().unwrap();
                // Only trigger note_on if this note isn't already active
                if !s.active_notes.contains(&midi_note) {
                    s.active_notes.push(midi_note);
                    s.allocator.note_on(midi_note, 0.8);
                }
            }
        }
        KeyEventKind::Release => {
            if let Some(released_note) = key_to_midi_note(key_event.code) {
                let mut s = state.lock().unwrap();
                // Remove from active notes and trigger note_off
                if let Some(pos) = s.active_notes.iter().position(|&n| n == released_note) {
                    s.active_notes.remove(pos);
                    s.allocator.note_off(released_note);
                }
            }
        }
        _ => {}
    }

    Ok(KeyAction::Continue)
}

fn main() -> Result<()> {
    run_interactive_example(
        PolyphonyDemoState::new(4), // Start with 4 voices
        KeyboardConfig::with_enhancements(),
        |_state| draw_ui(),
        handle_key,
    )
}
