//! Interactive TUI for switching between noise types.
//!
//! Press SPACE to cycle through noise types.
//! Press Q or ESC to quit.

use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, StreamConfig};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use earworm::{PinkNoise, Signal, WhiteNoise};
use rand::SeedableRng;
use std::io::{stdout, Write};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NoiseType {
    White,
    Pink,
}

impl NoiseType {
    fn next(self) -> Self {
        match self {
            NoiseType::White => NoiseType::Pink,
            NoiseType::Pink => NoiseType::White,
        }
    }

    fn name(self) -> &'static str {
        match self {
            NoiseType::White => "White Noise",
            NoiseType::Pink => "Pink Noise",
        }
    }
}

enum NoiseGenerator {
    White(WhiteNoise<rand::rngs::StdRng>),
    Pink(PinkNoise<rand::rngs::StdRng>),
}

impl NoiseGenerator {
    fn new(noise_type: NoiseType, sample_rate: f64) -> Self {
        // Create a seeded RNG (StdRng is Send + Sync)
        let rng = rand::rngs::StdRng::from_entropy();
        match noise_type {
            NoiseType::White => NoiseGenerator::White(WhiteNoise::with_rng(sample_rate, rng)),
            NoiseType::Pink => {
                let rng = rand::rngs::StdRng::from_entropy();
                NoiseGenerator::Pink(PinkNoise::with_rng(sample_rate, rng))
            }
        }
    }
}

impl Signal for NoiseGenerator {
    fn next_sample(&mut self) -> f64 {
        match self {
            NoiseGenerator::White(noise) => noise.next_sample(),
            NoiseGenerator::Pink(noise) => noise.next_sample(),
        }
    }
}

struct AudioState {
    generator: NoiseGenerator,
    noise_type: NoiseType,
    sample_rate: f64,
}

impl AudioState {
    fn new(sample_rate: f64) -> Self {
        let noise_type = NoiseType::White;
        Self {
            generator: NoiseGenerator::new(noise_type, sample_rate),
            noise_type,
            sample_rate,
        }
    }

    fn switch_noise_type(&mut self) {
        self.noise_type = self.noise_type.next();
        self.generator = NoiseGenerator::new(self.noise_type, self.sample_rate);
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
                let sample = state.generator.next_sample();
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

fn draw_ui(noise_type: NoiseType) -> Result<()> {
    let mut stdout = stdout();

    // Clear and show simple status
    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;
    stdout.execute(crossterm::cursor::MoveTo(0, 0))?;

    write!(stdout, "Playing: {} | SPACE=switch Q=quit", noise_type.name())?;

    stdout.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    // Setup audio
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::anyhow!("No output device available"))?;

    let config = device.default_output_config()?;
    let sample_rate = config.sample_rate().0 as f64;

    let state = Arc::new(Mutex::new(AudioState::new(sample_rate)));

    // Start audio stream
    let _stream = match config.sample_format() {
        SampleFormat::F32 => run_audio_stream::<f32>(&device, &config.into(), state.clone())?,
        SampleFormat::I16 => run_audio_stream::<i16>(&device, &config.into(), state.clone())?,
        SampleFormat::U16 => run_audio_stream::<u16>(&device, &config.into(), state.clone())?,
        sample_format => {
            return Err(anyhow::anyhow!(
                "Unsupported sample format: {}",
                sample_format
            ))
        }
    };

    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(crossterm::cursor::Hide)?;

    // Draw initial UI
    draw_ui(state.lock().unwrap().noise_type)?;

    // Event loop
    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break,
                    KeyCode::Char(' ') => {
                        let mut state = state.lock().unwrap();
                        state.switch_noise_type();
                        let noise_type = state.noise_type;
                        drop(state);
                        draw_ui(noise_type)?;
                    }
                    _ => {}
                }
            }
        }
    }

    // Cleanup terminal
    stdout().execute(crossterm::cursor::Show)?;
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
