//! Simple sequencer demonstration with audio output.
//!
//! This example shows the basic usage of the Sequencer to play a pattern.
//! It demonstrates the core concepts and plays the pattern through your speakers.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use earworm::music::{ADSR, Pattern, Sequencer, StealingStrategy, VoiceAllocator};
use earworm::{NoteEvent, Pitch, SawtoothOscillator, Signal};
use std::sync::{Arc, Mutex};

const SAMPLE_RATE: u32 = 44100;
const VOICES: usize = 4;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Earworm Sequencer Demo\n");

    // Create a sequencer at 120 BPM with 16th note steps
    let sequencer = Sequencer::new(120.0, 4, SAMPLE_RATE);

    // Create a simple 16-step pattern
    let mut pattern = Pattern::new(16);
    pattern.set_name("Acid Bassline");

    // Add some notes at various steps
    let notes = [
        (0, Pitch::C, 3),  // Step 0
        (3, Pitch::C, 4),  // Step 3
        (6, Pitch::G, 3),  // Step 6
        (9, Pitch::G, 3),  // Step 9
        (12, Pitch::C, 3), // Step 12
        (15, Pitch::D, 3), // Step 15
    ];

    for (step, pitch, octave) in notes {
        pattern.add_event(step, NoteEvent::from_pitch(pitch, octave, 0.8, Some(0.2)));
    }

    // Create synth with punchy envelope
    let osc = SawtoothOscillator::new(440.0);
    let env = ADSR::new(0.005, 0.1, 0.3, 0.2, SAMPLE_RATE as f64);
    let synth = VoiceAllocator::<SAMPLE_RATE, VOICES, _, _>::new(osc, env)
        .with_strategy(StealingStrategy::Oldest);

    let mut sequencer = sequencer;

    // Load the pattern and start playback
    sequencer.set_pattern(pattern);
    sequencer.play();

    println!("Pattern: {}", sequencer.pattern().unwrap().name().unwrap());
    println!("Tempo: {} BPM", sequencer.tempo());
    println!("Steps per beat: 4 (16th notes)");
    println!("\nPlaying for 2 seconds...\n");

    // Setup audio output
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("No output device available")?;

    let config = device.default_output_config()?;

    // Separate locks to avoid borrow conflicts
    let sequencer = Arc::new(Mutex::new(sequencer));
    let synth = Arc::new(Mutex::new(synth));

    let sequencer_clone = Arc::clone(&sequencer);
    let synth_clone = Arc::clone(&synth);

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(2) {
                // Tick the sequencer - now returns events directly!
                let events_opt = sequencer_clone.lock().unwrap().tick();

                if let Some(events) = events_opt {
                    // Trigger notes (no more copying needed at this level!)
                    let mut synth = synth_clone.lock().unwrap();
                    for event in events {
                        println!(
                            "  → Note: {:?} at velocity {:.2}",
                            event.note, event.velocity
                        );
                        // Convert frequency to MIDI note number
                        let midi_note =
                            (69.0 + 12.0 * (event.note.pitch / 440.0).log2()).round() as u8;
                        synth.note_on(midi_note, event.velocity);
                    }
                }

                // Generate audio sample
                let sample = synth_clone.lock().unwrap().next_sample() * 0.3;

                // Write to both channels
                frame[0] = sample as f32;
                if frame.len() > 1 {
                    frame[1] = sample as f32;
                }
            }
        },
        |err| eprintln!("Audio stream error: {}", err),
        None,
    )?;

    stream.play()?;

    // Play for 2 seconds
    std::thread::sleep(std::time::Duration::from_secs(2));

    println!("\n✓ Playback complete");

    Ok(())
}
