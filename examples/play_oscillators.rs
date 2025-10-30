//! Interactive example for switching between oscillator types.
//!
//! Press SPACE to cycle through oscillator types.
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
    PulseOscillator, SawtoothOscillator, Signal, SineOscillator, SquareOscillator,
    TriangleOscillator,
};
use std::io::{Write, stdout};
use std::sync::{Arc, Mutex};
use std::panic;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OscillatorType {
    Sine,
    Triangle,
    Sawtooth,
    Square,
    Pulse,
    PulseLFO,
}

impl OscillatorType {
    fn next(self) -> Self {
        match self {
            OscillatorType::Sine => OscillatorType::Triangle,
            OscillatorType::Triangle => OscillatorType::Sawtooth,
            OscillatorType::Sawtooth => OscillatorType::Square,
            OscillatorType::Square => OscillatorType::Pulse,
            OscillatorType::Pulse => OscillatorType::PulseLFO,
            OscillatorType::PulseLFO => OscillatorType::Sine,
        }
    }

    fn name(self) -> &'static str {
        match self {
            OscillatorType::Sine => "Sine",
            OscillatorType::Triangle => "Triangle",
            OscillatorType::Sawtooth => "Sawtooth",
            OscillatorType::Square => "Square",
            OscillatorType::Pulse => "Pulse (25%)",
            OscillatorType::PulseLFO => "Pulse (PWM)",
        }
    }
}

enum OscillatorWrapper {
    Sine(SineOscillator),
    Triangle(TriangleOscillator),
    Sawtooth(SawtoothOscillator),
    Square(SquareOscillator),
    Pulse(PulseOscillator),
    PulseLFO(PulseOscillator),
}

impl OscillatorWrapper {
    fn new(osc_type: OscillatorType, frequency: f64, sample_rate: f64) -> Self {
        match osc_type {
            OscillatorType::Sine => {
                OscillatorWrapper::Sine(SineOscillator::new(frequency, sample_rate))
            }
            OscillatorType::Triangle => {
                OscillatorWrapper::Triangle(TriangleOscillator::new(frequency, sample_rate))
            }
            OscillatorType::Sawtooth => {
                OscillatorWrapper::Sawtooth(SawtoothOscillator::new(frequency, sample_rate))
            }
            OscillatorType::Square => {
                OscillatorWrapper::Square(SquareOscillator::new(frequency, sample_rate))
            }
            OscillatorType::Pulse => {
                // Use 0.25 for a 25% duty cycle (maps to 0.625 internally)
                OscillatorWrapper::Pulse(PulseOscillator::new(
                    frequency,
                    sample_rate,
                    0.25.into(),
                ))
            }
            OscillatorType::PulseLFO => {
                // Create an LFO at 0.5 Hz to modulate the pulse width
                let lfo = SineOscillator::new(0.5, sample_rate);
                OscillatorWrapper::PulseLFO(PulseOscillator::new(
                    frequency,
                    sample_rate,
                    lfo.into(),
                ))
            }
        }
    }
}

impl Signal for OscillatorWrapper {
    fn next_sample(&mut self) -> f64 {
        match self {
            OscillatorWrapper::Sine(osc) => osc.next_sample(),
            OscillatorWrapper::Triangle(osc) => osc.next_sample(),
            OscillatorWrapper::Sawtooth(osc) => osc.next_sample(),
            OscillatorWrapper::Square(osc) => osc.next_sample(),
            OscillatorWrapper::Pulse(osc) => osc.next_sample(),
            OscillatorWrapper::PulseLFO(osc) => osc.next_sample(),
        }
    }
}

struct AudioState {
    oscillator: OscillatorWrapper,
    osc_type: OscillatorType,
    frequency: f64,
    sample_rate: f64,
    /// Fade-in counter to avoid clicks when switching
    fade_samples: usize,
}

impl AudioState {
    fn new(frequency: f64, sample_rate: f64) -> Self {
        let osc_type = OscillatorType::Sine;
        Self {
            oscillator: OscillatorWrapper::new(osc_type, frequency, sample_rate),
            osc_type,
            frequency,
            sample_rate,
            fade_samples: 0,
        }
    }

    fn switch_oscillator(&mut self) {
        self.osc_type = self.osc_type.next();
        self.oscillator = OscillatorWrapper::new(self.osc_type, self.frequency, self.sample_rate);
        // Add a brief fade-in to avoid clicks (2ms at 44100 Hz = ~88 samples)
        self.fade_samples = (self.sample_rate * 0.002) as usize;
    }

    fn next_sample(&mut self) -> f64 {
        let sample = self.oscillator.next_sample();

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

fn draw_ui(osc_type: OscillatorType, frequency: f64) -> Result<()> {
    let mut stdout = stdout();

    // Clear and show simple status
    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;
    stdout.execute(crossterm::cursor::MoveTo(0, 0))?;

    write!(
        stdout,
        "Playing: {} @ {:.0}Hz | SPACE=switch Q=quit",
        osc_type.name(),
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
    draw_ui(state.lock().unwrap().osc_type, frequency)?;

    // Event loop
    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break,
                    KeyCode::Char(' ') => {
                        let mut state = state.lock().unwrap();
                        state.switch_oscillator();
                        let osc_type = state.osc_type;
                        drop(state);
                        draw_ui(osc_type, frequency)?;
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
