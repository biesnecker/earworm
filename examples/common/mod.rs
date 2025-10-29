//! Common utilities for audio playback examples.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use earworm::Oscillator;
use std::sync::{Arc, Mutex};

/// Plays an oscillator for a specified duration.
///
/// # Arguments
///
/// * `oscillator` - The oscillator to play
/// * `waveform_name` - Name of the waveform for display (e.g., "sine", "triangle")
/// * `duration_secs` - How long to play the oscillator in seconds
///
/// # Returns
///
/// Result indicating success or error during playback
pub fn play_oscillator<O>(
    oscillator: O,
    waveform_name: &str,
    duration_secs: u64,
) -> Result<(), anyhow::Error>
where
    O: Oscillator + Send + 'static,
{
    // Set up the audio output device
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");

    let config = device.default_output_config()?;
    println!("Output device: {}", device.name()?);
    println!("Default output config: {:?}", config);

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32, O>(&device, &config.into(), oscillator, waveform_name, duration_secs)?,
        cpal::SampleFormat::I16 => run::<i16, O>(&device, &config.into(), oscillator, waveform_name, duration_secs)?,
        cpal::SampleFormat::U16 => run::<u16, O>(&device, &config.into(), oscillator, waveform_name, duration_secs)?,
        sample_format => panic!("Unsupported sample format '{sample_format}'"),
    }

    Ok(())
}

fn run<T, O>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    oscillator: O,
    waveform_name: &str,
    duration_secs: u64,
) -> Result<(), anyhow::Error>
where
    T: cpal::SizedSample + cpal::FromSample<f64>,
    O: Oscillator + Send + 'static,
{
    let sample_rate = config.sample_rate.0 as f64;
    let channels = config.channels as usize;
    let frequency = oscillator.frequency();

    let oscillator = Arc::new(Mutex::new(oscillator));

    println!(
        "Playing {} Hz {} wave at {} Hz sample rate with {} channel(s)...",
        frequency, waveform_name, sample_rate, channels
    );
    println!("Press Ctrl+C to stop.");

    let osc = oscillator.clone();
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            let mut osc = osc.lock().unwrap();
            for frame in data.chunks_mut(channels) {
                let sample = osc.next_sample();
                let value: T = cpal::Sample::from_sample(sample);
                for channel_sample in frame.iter_mut() {
                    *channel_sample = value;
                }
            }
        },
        err_fn,
        None,
    )?;

    stream.play()?;

    // Keep the program running
    std::thread::sleep(std::time::Duration::from_secs(duration_secs));

    println!("Done!");
    Ok(())
}
