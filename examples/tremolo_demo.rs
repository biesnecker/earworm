//! Interactive Tremolo effect demo.
//!
//! Press SPACE to toggle tremolo on/off.
//! Press UP/DOWN to adjust rate, LEFT/RIGHT to adjust depth.
//! Press Q or ESC to quit.

use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, StreamConfig};
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use earworm::{Signal, SineOscillator};
use std::io::{Write, stdout};
use std::panic;
use std::sync::{Arc, Mutex};

const SAMPLE_RATE: u32 = 44100;

/// Audio state with tremolo effect.
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
        // Update the LFO frequency
        self.lfo = SineOscillator::new(self.rate);
    }

    fn adjust_depth(&mut self, delta: f64) {
        self.depth = (self.depth + delta).clamp(0.0, 1.0);
    }

    fn is_tremolo_active(&self) -> bool {
        self.tremolo_enabled
    }

    fn get_rate(&self) -> f64 {
        self.rate
    }

    fn get_depth(&self) -> f64 {
        self.depth
    }

    fn next_sample(&mut self) -> f64 {
        let audio = self.oscillator.next_sample();

        if self.tremolo_enabled {
            // Apply tremolo effect manually
            let mod_value = self.lfo.next_sample();
            let gain = 1.0 + self.depth / 2.0 * (mod_value - 1.0);
            audio * gain * 0.3
        } else {
            // Just consume the LFO sample to keep it in sync
            self.lfo.next_sample();
            audio * 0.3
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

fn draw_ui(state: &AudioState) -> Result<()> {
    let mut stdout = stdout();

    // Clear and show simple status
    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;
    stdout.execute(crossterm::cursor::MoveTo(0, 0))?;

    let status = if state.is_tremolo_active() {
        "ON"
    } else {
        "OFF"
    };

    write!(
        stdout,
        "Tremolo: {} | Rate: {:.1}Hz | Depth: {:.2} | SPACE=toggle ↑↓=rate ←→=depth Q=quit",
        status,
        state.get_rate(),
        state.get_depth()
    )?;

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
    let frequency = 440.0; // A4 note

    // Setup audio
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::anyhow!("No output device available"))?;

    let config = device.default_output_config()?;

    let state = Arc::new(Mutex::new(AudioState::new(frequency)));

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
    {
        let state = state.lock().unwrap();
        draw_ui(&state)?;
    }

    // Event loop
    loop {
        if event::poll(std::time::Duration::from_millis(50))?
            && let Event::Key(KeyEvent { code, .. }) = event::read()?
        {
            match code {
                // Quit on Q or ESC
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break,

                // Spacebar - toggle tremolo
                KeyCode::Char(' ') => {
                    state.lock().unwrap().toggle_tremolo();
                }

                // Up arrow - increase rate
                KeyCode::Up => {
                    state.lock().unwrap().adjust_rate(0.5);
                }

                // Down arrow - decrease rate
                KeyCode::Down => {
                    state.lock().unwrap().adjust_rate(-0.5);
                }

                // Right arrow - increase depth
                KeyCode::Right => {
                    state.lock().unwrap().adjust_depth(0.05);
                }

                // Left arrow - decrease depth
                KeyCode::Left => {
                    state.lock().unwrap().adjust_depth(-0.05);
                }

                _ => {}
            }

            // Update UI after key press
            let state_guard = state.lock().unwrap();
            draw_ui(&state_guard)?;
        }
    }

    // Cleanup terminal
    cleanup_terminal();

    Ok(())
}
