//! Advanced mixing example using signal combinators.
//!
//! This example demonstrates the Mix combinator by creating musical chords
//! from multiple oscillators, with different waveforms and dynamic gain control.
//!
//! Press 1-5 to play different chords, S to stop.
//! Press Q or ESC to quit.

mod common;

use anyhow::Result;
use common::{ExampleAudioState, KeyAction, KeyboardConfig, is_quit_key, run_interactive_example};
use crossterm::{
    ExecutableCommand,
    event::{KeyCode, KeyEvent},
};
use earworm::{
    Mix3, Mix4, SawtoothOscillator, Signal, SignalExt, SineOscillator, SquareOscillator,
    TriangleOscillator,
};
use std::io::{Write, stdout};

const SAMPLE_RATE: u32 = 44100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChordType {
    Major,
    Minor,
    Dominant7,
    Complex,
    Octaves,
}

impl ChordType {
    fn name(self) -> &'static str {
        match self {
            ChordType::Major => "C Major (Sine waves)",
            ChordType::Minor => "C Minor (Triangle waves)",
            ChordType::Dominant7 => "C7 (Square waves)",
            ChordType::Complex => "Complex (Mixed waveforms)",
            ChordType::Octaves => "C Octaves (Sawtooth)",
        }
    }
}

struct AudioState {
    chord_type: Option<ChordType>,
    signal: Box<dyn Signal + Send>,
    fade_samples: usize,
}

impl AudioState {
    fn new() -> Self {
        Self {
            chord_type: None,
            signal: Box::new(SineOscillator::<SAMPLE_RATE>::new(0.0).gain(0.0)),
            fade_samples: 0,
        }
    }

    fn create_signal(chord_type: ChordType) -> Box<dyn Signal + Send> {
        let c4 = 261.63;
        let eb4 = 311.13;
        let e4 = 329.63;
        let g4 = 392.00;
        let bb4 = 466.16;
        let c3 = 130.81;
        let c5 = 523.25;

        match chord_type {
            ChordType::Major => Box::new(Mix3::new(
                SineOscillator::<SAMPLE_RATE>::new(c4),
                0.33,
                SineOscillator::<SAMPLE_RATE>::new(e4),
                0.33,
                SineOscillator::<SAMPLE_RATE>::new(g4),
                0.33,
            )),
            ChordType::Minor => Box::new(Mix3::new(
                TriangleOscillator::<SAMPLE_RATE>::new(c4),
                0.33,
                TriangleOscillator::<SAMPLE_RATE>::new(eb4),
                0.33,
                TriangleOscillator::<SAMPLE_RATE>::new(g4),
                0.33,
            )),
            ChordType::Dominant7 => Box::new(Mix4::new(
                SquareOscillator::<SAMPLE_RATE>::new(c4),
                0.25,
                SquareOscillator::<SAMPLE_RATE>::new(e4),
                0.25,
                SquareOscillator::<SAMPLE_RATE>::new(g4),
                0.25,
                SquareOscillator::<SAMPLE_RATE>::new(bb4),
                0.25,
            )),
            ChordType::Complex => {
                let lfo = SineOscillator::<SAMPLE_RATE>::new(2.0);
                Box::new(
                    Mix4::new(
                        SineOscillator::<SAMPLE_RATE>::new(c4),
                        0.25,
                        TriangleOscillator::<SAMPLE_RATE>::new(e4),
                        0.25,
                        SquareOscillator::<SAMPLE_RATE>::new(g4),
                        0.20,
                        SawtoothOscillator::<SAMPLE_RATE>::new(c3),
                        0.15,
                    )
                    .multiply(lfo.offset(1.0).gain(0.5)),
                )
            }
            ChordType::Octaves => Box::new(Mix3::new(
                SawtoothOscillator::<SAMPLE_RATE>::new(c3),
                0.40,
                SawtoothOscillator::<SAMPLE_RATE>::new(c4),
                0.35,
                SawtoothOscillator::<SAMPLE_RATE>::new(c5),
                0.25,
            )),
        }
    }

    fn play_chord(&mut self, chord_type: ChordType) {
        self.chord_type = Some(chord_type);
        self.signal = Self::create_signal(chord_type);
        self.fade_samples = (SAMPLE_RATE as f64 * 0.005) as usize;
    }

    fn stop(&mut self) {
        self.chord_type = None;
        self.signal = Box::new(SineOscillator::<SAMPLE_RATE>::new(0.0).gain(0.0));
    }
}

impl ExampleAudioState for AudioState {
    fn next_sample(&mut self) -> f64 {
        let sample = self.signal.next_sample();

        if self.fade_samples > 0 {
            let fade_start = (SAMPLE_RATE as f64 * 0.005) as usize;
            let fade_progress = 1.0 - (self.fade_samples as f64 / fade_start as f64);
            self.fade_samples -= 1;
            sample * fade_progress
        } else {
            sample
        }
    }
}

fn draw_ui(chord_type: Option<ChordType>) -> Result<()> {
    let mut stdout = stdout();
    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;
    stdout.execute(crossterm::cursor::MoveTo(0, 0))?;

    if let Some(chord) = chord_type {
        write!(
            stdout,
            "Playing: {} | 1-5=chord S=stop Q=quit",
            chord.name()
        )?;
    } else {
        write!(
            stdout,
            "Stopped | 1=Major 2=Minor 3=Dom7 4=Complex 5=Octaves Q=quit"
        )?;
    }

    stdout.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    run_interactive_example(
        AudioState::new(),
        KeyboardConfig::default(),
        |state| draw_ui(state.lock().unwrap().chord_type),
        |state, key_event: &KeyEvent| {
            match key_event.code {
                KeyCode::Char('1') => {
                    let mut s = state.lock().unwrap();
                    s.play_chord(ChordType::Major);
                    let chord_type = s.chord_type;
                    drop(s);
                    draw_ui(chord_type)?;
                }
                KeyCode::Char('2') => {
                    let mut s = state.lock().unwrap();
                    s.play_chord(ChordType::Minor);
                    let chord_type = s.chord_type;
                    drop(s);
                    draw_ui(chord_type)?;
                }
                KeyCode::Char('3') => {
                    let mut s = state.lock().unwrap();
                    s.play_chord(ChordType::Dominant7);
                    let chord_type = s.chord_type;
                    drop(s);
                    draw_ui(chord_type)?;
                }
                KeyCode::Char('4') => {
                    let mut s = state.lock().unwrap();
                    s.play_chord(ChordType::Complex);
                    let chord_type = s.chord_type;
                    drop(s);
                    draw_ui(chord_type)?;
                }
                KeyCode::Char('5') => {
                    let mut s = state.lock().unwrap();
                    s.play_chord(ChordType::Octaves);
                    let chord_type = s.chord_type;
                    drop(s);
                    draw_ui(chord_type)?;
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    let mut s = state.lock().unwrap();
                    s.stop();
                    drop(s);
                    draw_ui(None)?;
                }
                code if is_quit_key(code) => return Ok(KeyAction::Exit),
                _ => {}
            }
            Ok(KeyAction::Continue)
        },
    )
}
