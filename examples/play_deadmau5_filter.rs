//! Deadmau5-style pulsing filter effect.
//!
//! Creates the iconic "pulsing" sound by rapidly dropping a low-pass filter's
//! cutoff frequency from high to near-zero on each beat (8th notes at 120 BPM).
//!
//! Press Q or ESC to quit.

use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, StreamConfig};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use earworm::{AudioSignalExt, BiquadFilter, Signal, SignalExt, SawtoothOscillator, SquareOscillator};
use std::io::{stdout, Write};
use std::panic;
use std::sync::{Arc, Mutex};

/// The pulsing filter signal that creates the deadmau5 effect
struct DeadmauFilter {
    filter: BiquadFilter<SawtoothOscillator>,
}

impl DeadmauFilter {
    fn new(sample_rate: f64) -> Self {
        // Base frequency for the sawtooth (nice thick sound for this effect)
        let base_freq = 110.0; // A2

        // Create a sawtooth oscillator for rich harmonic content
        let osc = SawtoothOscillator::new(base_freq, sample_rate);

        // At 120 BPM, 8th notes occur at 4 Hz (120 BPM / 60 * 2 beats per half note / 4 eighth notes)
        let pulse_rate = 4.0; // Hz

        // Create a square wave LFO for the sharp on/off pulsing effect
        let lfo = SquareOscillator::new(pulse_rate, sample_rate);

        // Map the square wave (-1 to 1) to cutoff frequency
        // When LFO is high (1): cutoff at ~4000Hz (open filter)
        // When LFO is low (-1): cutoff at ~50Hz (closed filter, dark)
        // This creates the dramatic "drop" effect
        let modulated_cutoff = lfo
            .gain(1975.0)      // Scale: 2000Hz range
            .offset(2025.0);   // Offset: centered at 2025Hz (50Hz to 4000Hz)

        // Use moderate Q for some resonance at the cutoff
        let q = 2.0;

        let filter = osc.lowpass_filter(modulated_cutoff, q);

        Self { filter }
    }
}

impl Signal for DeadmauFilter {
    fn next_sample(&mut self) -> f64 {
        // Scale down to prevent clipping
        self.filter.next_sample() * 0.3
    }
}

fn run_audio_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    state: Arc<Mutex<DeadmauFilter>>,
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

fn draw_ui() -> Result<()> {
    let mut stdout = stdout();

    // Clear and show simple status
    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;
    stdout.execute(crossterm::cursor::MoveTo(0, 0))?;

    write!(
        stdout,
        "Playing: Deadmau5 Filter (4kHz→50Hz @ 4Hz) | Q=quit"
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
    // Setup audio
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::anyhow!("No output device available"))?;

    let config = device.default_output_config()?;
    let sample_rate = config.sample_rate().0 as f64;

    let state = Arc::new(Mutex::new(DeadmauFilter::new(sample_rate)));

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

    // Set up panic hook to restore terminal on panic
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        cleanup_terminal();
        original_hook(panic_info);
    }));

    // Draw UI
    draw_ui()?;

    // Event loop
    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    // Cleanup terminal
    cleanup_terminal();

    Ok(())
}
