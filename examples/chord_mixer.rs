//! Advanced mixing example using signal combinators.
//!
//! This example demonstrates the Mix combinator by creating musical chords
//! from multiple oscillators, with different waveforms and dynamic gain control.
//!
//! Press 1-5 to play different chords
//! Press Q or ESC to quit.

use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, StreamConfig};
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use earworm::{
    Mix3, Mix4, SawtoothOscillator, Signal, SignalExt, SineOscillator, SquareOscillator,
    TriangleOscillator,
};
use std::io::{Write, stdout};
use std::panic;
use std::sync::{Arc, Mutex};

const SAMPLE_RATE: u32 = 44100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChordType {
    /// Major triad (root, major third, perfect fifth)
    Major,
    /// Minor triad (root, minor third, perfect fifth)
    Minor,
    /// Dominant seventh (root, major third, perfect fifth, minor seventh)
    Dominant7,
    /// Complex chord with multiple oscillator types
    Complex,
    /// Octaves with different waveforms
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
        // Note frequencies (approximately)
        let c4 = 261.63; // Middle C
        let eb4 = 311.13; // E flat
        let e4 = 329.63; // E
        let g4 = 392.00; // G
        let bb4 = 466.16; // B flat
        let c3 = 130.81; // Low C
        let c5 = 523.25; // High C

        match chord_type {
            ChordType::Major => {
                // Major triad using sine waves
                Box::new(Mix3::new(
                    SineOscillator::<SAMPLE_RATE>::new(c4),
                    0.33,
                    SineOscillator::<SAMPLE_RATE>::new(e4),
                    0.33,
                    SineOscillator::<SAMPLE_RATE>::new(g4),
                    0.33,
                ))
            }
            ChordType::Minor => {
                // Minor triad using triangle waves for a warmer sound
                Box::new(Mix3::new(
                    TriangleOscillator::<SAMPLE_RATE>::new(c4),
                    0.33,
                    TriangleOscillator::<SAMPLE_RATE>::new(eb4),
                    0.33,
                    TriangleOscillator::<SAMPLE_RATE>::new(g4),
                    0.33,
                ))
            }
            ChordType::Dominant7 => {
                // Seventh chord using square waves for a bright sound
                Box::new(Mix4::new(
                    SquareOscillator::<SAMPLE_RATE>::new(c4),
                    0.25,
                    SquareOscillator::<SAMPLE_RATE>::new(e4),
                    0.25,
                    SquareOscillator::<SAMPLE_RATE>::new(g4),
                    0.25,
                    SquareOscillator::<SAMPLE_RATE>::new(bb4),
                    0.25,
                ))
            }
            ChordType::Complex => {
                // Complex chord mixing different waveform types
                // Also demonstrates using the SignalExt trait for effects
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
                    // Add a slow tremolo effect using the LFO
                    .multiply(lfo.offset(1.0).gain(0.5)),
                )
            }
            ChordType::Octaves => {
                // Same note across three octaves using sawtooth waves
                Box::new(Mix3::new(
                    SawtoothOscillator::<SAMPLE_RATE>::new(c3),
                    0.40,
                    SawtoothOscillator::<SAMPLE_RATE>::new(c4),
                    0.35,
                    SawtoothOscillator::<SAMPLE_RATE>::new(c5),
                    0.25,
                ))
            }
        }
    }

    fn play_chord(&mut self, chord_type: ChordType) {
        self.chord_type = Some(chord_type);
        self.signal = Self::create_signal(chord_type);
        // Add a brief fade-in to avoid clicks (5ms)
        self.fade_samples = (SAMPLE_RATE as f64 * 0.005) as usize;
    }

    fn stop(&mut self) {
        self.chord_type = None;
        self.signal = Box::new(SineOscillator::<SAMPLE_RATE>::new(0.0).gain(0.0));
    }

    fn next_sample(&mut self) -> f64 {
        let sample = self.signal.next_sample();

        // Apply fade-in if we just switched
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

fn run_audio_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    state: Arc<Mutex<AudioState>>,
) -> Result<cpal::Stream>
where
    T: Sample + FromSample<f64> + cpal::SizedSample,
{
    let channels = config.channels as usize;

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            let mut state = state.lock().unwrap();
            for frame in data.chunks_mut(channels) {
                let sample = state.next_sample();
                let value: T = T::from_sample(sample);
                for s in frame.iter_mut() {
                    *s = value;
                }
            }
        },
        |err| eprintln!("Audio stream error: {}", err),
        None,
    )?;

    stream.play()?;
    Ok(stream)
}

fn draw_ui(chord_type: Option<ChordType>) -> Result<()> {
    let mut stdout = stdout();

    // Clear and show simple status
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

/// Cleanup function to restore terminal state
fn cleanup_terminal() {
    let _ = stdout().execute(crossterm::cursor::Show);
    let _ = stdout().execute(LeaveAlternateScreen);
    let _ = disable_raw_mode();
}

fn main() -> Result<()> {
    // Setup audio
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::anyhow!("No output device available"))?;

    let config = device.default_output_config()?;

    let state = Arc::new(Mutex::new(AudioState::new()));

    // Start audio stream
    let _stream = match config.sample_format() {
        SampleFormat::F32 => run_audio_stream::<f32>(&device, &config.into(), state.clone())?,
        SampleFormat::I16 => run_audio_stream::<i16>(&device, &config.into(), state.clone())?,
        SampleFormat::U16 => run_audio_stream::<u16>(&device, &config.into(), state.clone())?,
        sample_format => {
            return Err(anyhow::anyhow!(
                "Unsupported sample format: {}",
                sample_format
            ));
        }
    };

    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(crossterm::cursor::Hide)?;

    // Set up panic hook to restore terminal on panic
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        cleanup_terminal();
        original_hook(panic_info);
    }));

    // Draw initial UI
    draw_ui(None)?;

    // Event loop
    loop {
        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(KeyEvent { code, .. }) = event::read()?
        {
            match code {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break,
                KeyCode::Char('1') => {
                    let mut state = state.lock().unwrap();
                    state.play_chord(ChordType::Major);
                    let chord_type = state.chord_type;
                    drop(state);
                    draw_ui(chord_type)?;
                }
                KeyCode::Char('2') => {
                    let mut state = state.lock().unwrap();
                    state.play_chord(ChordType::Minor);
                    let chord_type = state.chord_type;
                    drop(state);
                    draw_ui(chord_type)?;
                }
                KeyCode::Char('3') => {
                    let mut state = state.lock().unwrap();
                    state.play_chord(ChordType::Dominant7);
                    let chord_type = state.chord_type;
                    drop(state);
                    draw_ui(chord_type)?;
                }
                KeyCode::Char('4') => {
                    let mut state = state.lock().unwrap();
                    state.play_chord(ChordType::Complex);
                    let chord_type = state.chord_type;
                    drop(state);
                    draw_ui(chord_type)?;
                }
                KeyCode::Char('5') => {
                    let mut state = state.lock().unwrap();
                    state.play_chord(ChordType::Octaves);
                    let chord_type = state.chord_type;
                    drop(state);
                    draw_ui(chord_type)?;
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    let mut state = state.lock().unwrap();
                    state.stop();
                    drop(state);
                    draw_ui(None)?;
                }
                _ => {}
            }
        }
    }

    // Cleanup terminal
    cleanup_terminal();

    Ok(())
}
