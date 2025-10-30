//! Interactive example for demonstrating delay effects.
//!
//! Press SPACE to cycle through delay types.
//! Press Q or ESC to quit.

use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, StreamConfig};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use earworm::{Delay, Gain, Gate, Signal, SignalExt, SineOscillator, SquareOscillator};
use std::io::{stdout, Write};
use std::panic;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DelayType {
    Slapback,
    ShortEcho,
    MediumEcho,
    LongEcho,
    ModulatedDelay,
    NoDry,
}

impl DelayType {
    fn next(self) -> Self {
        match self {
            DelayType::Slapback => DelayType::ShortEcho,
            DelayType::ShortEcho => DelayType::MediumEcho,
            DelayType::MediumEcho => DelayType::LongEcho,
            DelayType::LongEcho => DelayType::ModulatedDelay,
            DelayType::ModulatedDelay => DelayType::NoDry,
            DelayType::NoDry => DelayType::Slapback,
        }
    }

    fn name(self) -> &'static str {
        match self {
            DelayType::Slapback => "Slapback (75ms)",
            DelayType::ShortEcho => "Short Echo (200ms)",
            DelayType::MediumEcho => "Medium Echo (375ms)",
            DelayType::LongEcho => "Long Echo (500ms)",
            DelayType::ModulatedDelay => "Modulated (PWM)",
            DelayType::NoDry => "100% Wet (500ms)",
        }
    }
}

enum DelayWrapper {
    Slapback(Delay<Gate<Gain<SineOscillator>>>),
    ShortEcho(Delay<Gate<Gain<SineOscillator>>>),
    MediumEcho(Delay<Gate<Gain<SineOscillator>>>),
    LongEcho(Delay<Gate<Gain<SineOscillator>>>),
    ModulatedDelay(Delay<Gate<Gain<SineOscillator>>>),
    NoDry(Delay<Gate<Gain<SineOscillator>>>),
}

impl DelayWrapper {
    fn new(delay_type: DelayType, frequency: f64, sample_rate: f64) -> Self {
        // Create a pulsed sine wave so we can hear distinct echoes
        // Gate opens when LFO > 0, creating rhythmic pulses
        let sine = SineOscillator::new(frequency, sample_rate);
        let gained = sine.gain(0.5); // Reduce volume a bit
        let lfo = SquareOscillator::new(2.0, sample_rate); // 2 Hz pulse rate
        let source = gained.gate(lfo);

        match delay_type {
            DelayType::Slapback => {
                DelayWrapper::Slapback(Delay::slapback(source))
            }
            DelayType::ShortEcho => {
                DelayWrapper::ShortEcho(Delay::echo(source, 0.2, 0.5))
            }
            DelayType::MediumEcho => {
                DelayWrapper::MediumEcho(Delay::echo(source, 0.375, 0.6))
            }
            DelayType::LongEcho => {
                DelayWrapper::LongEcho(Delay::echo(source, 0.5, 0.75))
            }
            DelayType::ModulatedDelay => {
                // Create an LFO to modulate the delay time
                let mod_lfo = SineOscillator::new(0.3, sample_rate);
                DelayWrapper::ModulatedDelay(Delay::new(
                    source,
                    0.6,     // max delay time
                    mod_lfo, // modulated delay time
                    0.6,     // feedback
                    0.5,     // mix
                ))
            }
            DelayType::NoDry => {
                DelayWrapper::NoDry(Delay::new(source, 0.5, 0.5, 0.6, 1.0))
            }
        }
    }
}

impl Signal for DelayWrapper {
    fn next_sample(&mut self) -> f64 {
        match self {
            DelayWrapper::Slapback(d) => d.next_sample(),
            DelayWrapper::ShortEcho(d) => d.next_sample(),
            DelayWrapper::MediumEcho(d) => d.next_sample(),
            DelayWrapper::LongEcho(d) => d.next_sample(),
            DelayWrapper::ModulatedDelay(d) => d.next_sample(),
            DelayWrapper::NoDry(d) => d.next_sample(),
        }
    }
}

struct AudioState {
    delay: DelayWrapper,
    delay_type: DelayType,
    frequency: f64,
    sample_rate: f64,
    /// Fade-in counter to avoid clicks when switching
    fade_samples: usize,
}

impl AudioState {
    fn new(frequency: f64, sample_rate: f64) -> Self {
        let delay_type = DelayType::Slapback;
        Self {
            delay: DelayWrapper::new(delay_type, frequency, sample_rate),
            delay_type,
            frequency,
            sample_rate,
            fade_samples: 0,
        }
    }

    fn switch_delay(&mut self) {
        self.delay_type = self.delay_type.next();
        self.delay = DelayWrapper::new(self.delay_type, self.frequency, self.sample_rate);
        // Add a brief fade-in to avoid clicks (10ms at 44100 Hz = ~441 samples)
        self.fade_samples = (self.sample_rate * 0.01) as usize;
    }

    fn next_sample(&mut self) -> f64 {
        let sample = self.delay.next_sample();

        // Apply fade-in if we just switched
        if self.fade_samples > 0 {
            let fade_start = (self.sample_rate * 0.01) as usize;
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
                let value: T = T::from_sample(sample * 0.5);
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

fn draw_ui(delay_type: DelayType, frequency: f64) -> Result<()> {
    let mut stdout = stdout();

    // Clear and show simple status
    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;
    stdout.execute(crossterm::cursor::MoveTo(0, 0))?;

    write!(
        stdout,
        "Playing: {} @ {:.0}Hz | SPACE=switch Q=quit",
        delay_type.name(),
        frequency
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
    draw_ui(state.lock().unwrap().delay_type, frequency)?;

    // Event loop
    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break,
                    KeyCode::Char(' ') => {
                        let mut state = state.lock().unwrap();
                        state.switch_delay();
                        let delay_type = state.delay_type;
                        drop(state);
                        draw_ui(delay_type, frequency)?;
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
