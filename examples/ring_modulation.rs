//! Ring modulation example using signal combinators.
//!
//! This example demonstrates ring modulation by multiplying a carrier wave
//! with a modulator wave to create interesting harmonic effects. It also
//! showcases amplitude modulation (tremolo) with an LFO.
//!
//! Press SPACE to cycle through different modulation effects.
//! Press Q or ESC to quit.

use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, StreamConfig};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use earworm::{Signal, SignalExt, SineOscillator};
use std::io::{stdout, Write};
use std::sync::{Arc, Mutex};
use std::panic;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModulationType {
    /// No modulation - just the carrier
    None,
    /// Slow tremolo (amplitude modulation with LFO)
    Tremolo,
    /// Ring modulation with a low frequency
    RingLow,
    /// Ring modulation with harmonic frequency
    RingHarmonic,
    /// Ring modulation with inharmonic frequency
    RingInharmonic,
}

impl ModulationType {
    fn next(self) -> Self {
        match self {
            ModulationType::None => ModulationType::Tremolo,
            ModulationType::Tremolo => ModulationType::RingLow,
            ModulationType::RingLow => ModulationType::RingHarmonic,
            ModulationType::RingHarmonic => ModulationType::RingInharmonic,
            ModulationType::RingInharmonic => ModulationType::None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            ModulationType::None => "No Modulation",
            ModulationType::Tremolo => "Tremolo (6 Hz LFO)",
            ModulationType::RingLow => "Ring Mod (30 Hz)",
            ModulationType::RingHarmonic => "Ring Mod (660 Hz - 3:2 ratio)",
            ModulationType::RingInharmonic => "Ring Mod (573 Hz - inharmonic)",
        }
    }
}

struct AudioState {
    carrier_freq: f64,
    sample_rate: f64,
    mod_type: ModulationType,
    signal: Box<dyn Signal + Send>,
    fade_samples: usize,
}

impl AudioState {
    fn new(carrier_freq: f64, sample_rate: f64) -> Self {
        let mod_type = ModulationType::None;
        let signal = Self::create_signal(mod_type, carrier_freq, sample_rate);
        Self {
            carrier_freq,
            sample_rate,
            mod_type,
            signal,
            fade_samples: 0,
        }
    }

    fn create_signal(
        mod_type: ModulationType,
        carrier_freq: f64,
        sample_rate: f64,
    ) -> Box<dyn Signal + Send> {
        let carrier = SineOscillator::new(carrier_freq, sample_rate);

        match mod_type {
            ModulationType::None => {
                // Just the carrier with some gain reduction
                Box::new(carrier.gain(0.3))
            }
            ModulationType::Tremolo => {
                // Amplitude modulation with a slow LFO
                // LFO output is -1 to 1, so we offset it to 0 to 2, then multiply by 0.5
                // to get 0 to 1 range for smooth tremolo
                let lfo = SineOscillator::new(6.0, sample_rate);
                Box::new(
                    carrier
                        .multiply(lfo.offset(1.0).gain(0.5))
                        .gain(0.3),
                )
            }
            ModulationType::RingLow => {
                // Ring modulation with a low frequency creates a warbling effect
                let modulator = SineOscillator::new(30.0, sample_rate);
                Box::new(carrier.multiply(modulator).gain(0.3))
            }
            ModulationType::RingHarmonic => {
                // Ring modulation with a harmonic frequency (3:2 ratio creates a musical fifth)
                let modulator = SineOscillator::new(carrier_freq * 1.5, sample_rate);
                Box::new(carrier.multiply(modulator).gain(0.3))
            }
            ModulationType::RingInharmonic => {
                // Ring modulation with an inharmonic frequency creates dissonant metallic tones
                let modulator = SineOscillator::new(573.0, sample_rate);
                Box::new(carrier.multiply(modulator).gain(0.3))
            }
        }
    }

    fn switch_modulation(&mut self) {
        self.mod_type = self.mod_type.next();
        self.signal = Self::create_signal(self.mod_type, self.carrier_freq, self.sample_rate);
        // Add a brief fade-in to avoid clicks (2ms)
        self.fade_samples = (self.sample_rate * 0.002) as usize;
    }

    fn next_sample(&mut self) -> f64 {
        let sample = self.signal.next_sample();

        // Apply fade-in if we just switched
        if self.fade_samples > 0 {
            let fade_start = (self.sample_rate * 0.002) as usize;
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

fn draw_ui(mod_type: ModulationType) -> Result<()> {
    let mut stdout = stdout();

    // Clear and show simple status
    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;
    stdout.execute(crossterm::cursor::MoveTo(0, 0))?;

    write!(stdout, "Playing: {} | SPACE=switch Q=quit", mod_type.name())?;

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
    let sample_rate = config.sample_rate().0 as f64;

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
    draw_ui(state.lock().unwrap().mod_type)?;

    // Event loop
    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break,
                    KeyCode::Char(' ') => {
                        let mut state = state.lock().unwrap();
                        state.switch_modulation();
                        let mod_type = state.mod_type;
                        drop(state);
                        draw_ui(mod_type)?;
                    }
                    _ => {}
                }
            }
        }
    }

    // Cleanup terminal
    cleanup_terminal();

    Ok(())
}
