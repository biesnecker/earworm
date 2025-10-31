//! Envelope comparison demo.
//!
//! This example demonstrates different envelope types (ADSR, AR, AHD) by playing
//! a fixed note with each envelope. Press and hold SPACE to trigger and sustain the note,
//! release to hear the release phase.
//!
//! Controls:
//! - SPACE: Trigger/sustain note (hold for sustain, release to hear release phase)
//! - 1: ADSR envelope (classic synth envelope)
//! - 2: AR envelope (percussive)
//! - 3: AHD envelope (bell-like)
//! - Q/ESC: Quit

mod common;

use anyhow::Result;
use common::{ExampleAudioState, KeyAction, KeyboardConfig, is_quit_key, run_interactive_example};
use crossterm::{
    ExecutableCommand,
    event::{KeyCode, KeyEvent, KeyEventKind},
};
use earworm::{Signal, SineOscillator};
use std::io::{Write, stdout};
use std::sync::{Arc, Mutex};

const SAMPLE_RATE: u32 = 44100;
const NOTE_FREQ: f64 = 220.0; // A3

// Envelope types we can switch between
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
enum EnvelopeType {
    ADSR,
    AR,
    AHD,
}

impl EnvelopeType {
    fn name(&self) -> &'static str {
        match self {
            EnvelopeType::ADSR => "ADSR (Attack-Decay-Sustain-Release)",
            EnvelopeType::AR => "AR (Attack-Release)",
            EnvelopeType::AHD => "AHD (Attack-Hold-Decay)",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            EnvelopeType::ADSR => "Classic synth envelope with sustain phase",
            EnvelopeType::AR => "Simple percussive envelope, no sustain",
            EnvelopeType::AHD => "Bell-like envelope with hold at peak",
        }
    }

    fn parameters(&self) -> &'static str {
        match self {
            EnvelopeType::ADSR => "A:10ms D:100ms S:70% R:200ms",
            EnvelopeType::AR => "A:10ms R:300ms",
            EnvelopeType::AHD => "A:10ms H:100ms D:400ms",
        }
    }
}

// Trait-object wrapper for different envelope types
trait EnvelopeWrapper: Send {
    fn trigger(&mut self, velocity: f64);
    fn release(&mut self);
    fn next_sample(&mut self) -> f64;
    fn is_active(&self) -> bool;
}

// Implement for each envelope type
impl EnvelopeWrapper for earworm::ADSR {
    fn trigger(&mut self, velocity: f64) {
        earworm::music::Envelope::trigger(self, velocity);
    }
    fn release(&mut self) {
        earworm::music::Envelope::release(self);
    }
    fn next_sample(&mut self) -> f64 {
        earworm::music::Envelope::next_sample(self)
    }
    fn is_active(&self) -> bool {
        earworm::music::Envelope::is_active(self)
    }
}

impl EnvelopeWrapper for earworm::AR {
    fn trigger(&mut self, velocity: f64) {
        earworm::music::Envelope::trigger(self, velocity);
    }
    fn release(&mut self) {
        earworm::music::Envelope::release(self);
    }
    fn next_sample(&mut self) -> f64 {
        earworm::music::Envelope::next_sample(self)
    }
    fn is_active(&self) -> bool {
        earworm::music::Envelope::is_active(self)
    }
}

impl EnvelopeWrapper for earworm::AHD {
    fn trigger(&mut self, velocity: f64) {
        earworm::music::Envelope::trigger(self, velocity);
    }
    fn release(&mut self) {
        earworm::music::Envelope::release(self);
    }
    fn next_sample(&mut self) -> f64 {
        earworm::music::Envelope::next_sample(self)
    }
    fn is_active(&self) -> bool {
        earworm::music::Envelope::is_active(self)
    }
}

struct EnvelopeState {
    oscillator: SineOscillator<SAMPLE_RATE>,
    envelope: Box<dyn EnvelopeWrapper>,
    current_envelope_type: EnvelopeType,
    note_is_held: bool,
}

impl EnvelopeState {
    fn new() -> Self {
        let oscillator = SineOscillator::new(NOTE_FREQ);
        let envelope = Self::create_envelope(EnvelopeType::ADSR);

        Self {
            oscillator,
            envelope,
            current_envelope_type: EnvelopeType::ADSR,
            note_is_held: false,
        }
    }

    fn create_envelope(env_type: EnvelopeType) -> Box<dyn EnvelopeWrapper> {
        match env_type {
            EnvelopeType::ADSR => {
                Box::new(earworm::ADSR::new(0.01, 0.1, 0.7, 0.2, SAMPLE_RATE as f64))
            }
            EnvelopeType::AR => Box::new(earworm::AR::new(0.01, 0.3, SAMPLE_RATE as f64)),
            EnvelopeType::AHD => Box::new(earworm::AHD::new(0.01, 0.1, 0.4, SAMPLE_RATE as f64)),
        }
    }

    fn switch_envelope(&mut self, env_type: EnvelopeType) {
        if env_type != self.current_envelope_type {
            self.current_envelope_type = env_type;
            self.envelope = Self::create_envelope(env_type);
        }
    }

    fn trigger_note(&mut self) {
        // Only trigger if note isn't already held
        if !self.note_is_held {
            self.envelope.trigger(0.8);
            self.note_is_held = true;
        }
    }

    fn release_note(&mut self) {
        if self.note_is_held {
            self.envelope.release();
            self.note_is_held = false;
        }
    }
}

impl ExampleAudioState for EnvelopeState {
    fn next_sample(&mut self) -> f64 {
        let osc_sample = self.oscillator.next_sample();
        let env_sample = self.envelope.next_sample();
        osc_sample * env_sample * 0.3 // Reduce volume
    }

    fn output_info(&self) -> Option<String> {
        let status = if self.envelope.is_active() {
            "PLAYING"
        } else {
            "IDLE"
        };

        Some(format!(
            "{} | {} | {} | {}",
            status,
            self.current_envelope_type.name(),
            self.current_envelope_type.description(),
            self.current_envelope_type.parameters()
        ))
    }
}

fn draw_ui(state: &Arc<Mutex<EnvelopeState>>) -> Result<()> {
    let mut stdout = stdout();
    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;
    stdout.execute(crossterm::cursor::MoveTo(0, 0))?;

    let mode = state.lock().unwrap().current_envelope_type;
    write!(
        stdout,
        "Envelope Demo | {} | SPACE=play 1/2/3=switch Q=quit",
        mode.name()
    )?;
    stdout.flush()?;
    Ok(())
}

fn handle_key(state: &Arc<Mutex<EnvelopeState>>, key: &KeyEvent) -> Result<KeyAction> {
    match key.code {
        KeyCode::Char(' ') => {
            // Only trigger on key press, release on key release
            match key.kind {
                KeyEventKind::Press => {
                    state.lock().unwrap().trigger_note();
                }
                KeyEventKind::Release => {
                    state.lock().unwrap().release_note();
                }
                _ => {}
            }
            Ok(KeyAction::Continue)
        }
        KeyCode::Char('1') => {
            state.lock().unwrap().switch_envelope(EnvelopeType::ADSR);
            draw_ui(state)?;
            Ok(KeyAction::Continue)
        }
        KeyCode::Char('2') => {
            state.lock().unwrap().switch_envelope(EnvelopeType::AR);
            draw_ui(state)?;
            Ok(KeyAction::Continue)
        }
        KeyCode::Char('3') => {
            state.lock().unwrap().switch_envelope(EnvelopeType::AHD);
            draw_ui(state)?;
            Ok(KeyAction::Continue)
        }
        code if is_quit_key(code) => Ok(KeyAction::Exit),
        _ => Ok(KeyAction::Continue),
    }
}

fn main() -> Result<()> {
    let state = EnvelopeState::new();

    run_interactive_example(
        state,
        KeyboardConfig::with_enhancements(), // Enable key press/release detection
        draw_ui,
        handle_key,
    )?;

    Ok(())
}
