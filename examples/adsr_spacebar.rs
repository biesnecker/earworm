//! Interactive ADSR envelope example.
//!
//! Press and hold SPACE to play a note with an ADSR envelope.
//! The envelope goes through Attack → Decay → Sustain while held.
//! Release SPACE to trigger the Release phase.
//! Press Q or ESC to quit.

use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, StreamConfig};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags, KeyboardEnhancementFlags},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use earworm::{Curve, Signal, SineOscillator, ADSR};
use std::io::{stdout, Write};
use std::sync::{Arc, Mutex};
use std::panic;

/// Audio state containing both oscillator and envelope.
struct AudioState {
    oscillator: SineOscillator,
    envelope: ADSR,
}

impl AudioState {
    fn new(frequency: f64, sample_rate: f64) -> Self {
        let oscillator = SineOscillator::new(frequency, sample_rate);

        // Create ADSR with exponential curves for natural sound
        let envelope = ADSR::new(
            0.05,  // 50ms attack
            0.1,   // 100ms decay
            0.7,   // 70% sustain level
            0.3,   // 300ms release
            sample_rate,
        )
        .with_attack_curve(Curve::Exponential(2.0))
        .with_decay_curve(Curve::Exponential(2.0))
        .with_release_curve(Curve::Exponential(3.0));

        Self {
            oscillator,
            envelope,
        }
    }

    fn note_on(&mut self) {
        self.envelope.note_on();
    }

    fn note_off(&mut self) {
        self.envelope.note_off();
    }

    fn is_active(&self) -> bool {
        self.envelope.is_active()
    }

    fn next_sample(&mut self) -> f64 {
        let oscillator_sample = self.oscillator.next_sample();
        let envelope_level = self.envelope.next_sample();

        // Apply envelope to oscillator output
        oscillator_sample * envelope_level * 0.3 // Scale to 30% to avoid clipping
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

/// Cleanup function to restore terminal state
fn cleanup_terminal() {
    let _ = stdout().execute(PopKeyboardEnhancementFlags);
    let _ = stdout().execute(crossterm::cursor::Show);
    let _ = stdout().execute(LeaveAlternateScreen);
    let _ = disable_raw_mode();
}

fn main() -> Result<()> {
    let frequency = 440.0; // A4 note

    // Setup audio
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::anyhow!("No output device available"))?;

    let config = device.default_output_config()?;
    let sample_rate = config.sample_rate().0 as f64;

    println!(
        "Audio device: {} @ {} Hz",
        device.name()?,
        sample_rate
    );

    let state = Arc::new(Mutex::new(AudioState::new(frequency, sample_rate)));

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

    // Setup terminal - ORDER MATTERS!
    // Must enable keyboard enhancement BEFORE entering alternate screen
    stdout().execute(PushKeyboardEnhancementFlags(
        KeyboardEnhancementFlags::REPORT_EVENT_TYPES
    ))?;

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
    draw_ui(false)?;

    let mut space_pressed = false;

    // Event loop
    loop {
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(KeyEvent { code, kind, .. }) = event::read()? {
                match code {
                    // Quit on Q or ESC
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                        if matches!(kind, KeyEventKind::Press) {
                            break;
                        }
                    }

                    // Spacebar - handle press/repeat/release
                    KeyCode::Char(' ') => {
                        if matches!(kind, KeyEventKind::Press | KeyEventKind::Repeat) {
                            if !space_pressed {
                                space_pressed = true;
                                state.lock().unwrap().note_on();
                            }
                        } else if matches!(kind, KeyEventKind::Release) {
                            space_pressed = false;
                            state.lock().unwrap().note_off();
                        }
                    }

                    _ => {}
                }
            }
        }

        // Update UI periodically to show envelope state changes
        let is_active = state.lock().unwrap().is_active();
        draw_ui(is_active)?;
    }

    // Cleanup terminal
    cleanup_terminal();

    println!("\nGoodbye!");
    Ok(())
}
