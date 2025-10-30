//! Deadmau5-style pulsing filter effect.
//!
//! Creates the iconic "pulsing" sound by rapidly dropping a low-pass filter's
//! cutoff frequency from high to near-zero on each beat (8th notes at 120 BPM).
//!
//! Press Q or ESC to quit.

mod common;

use anyhow::Result;
use common::{ExampleAudioState, KeyAction, KeyboardConfig, is_quit_key, run_interactive_example};
use crossterm::{ExecutableCommand, event::KeyEvent};
use earworm::{
    AudioSignalExt, BiquadFilter, SawtoothOscillator, Signal, SignalExt, SquareOscillator,
};
use std::io::{Write, stdout};

const SAMPLE_RATE: u32 = 44100;

/// The pulsing filter signal that creates the deadmau5 effect
struct DeadmauFilter {
    filter: BiquadFilter<SAMPLE_RATE, SawtoothOscillator<SAMPLE_RATE>>,
}

impl DeadmauFilter {
    fn new() -> Self {
        // Base frequency for the sawtooth (nice thick sound for this effect)
        let base_freq = 110.0; // A2

        // Create a sawtooth oscillator for rich harmonic content
        let osc = SawtoothOscillator::new(base_freq);

        // At 120 BPM, 8th notes occur at 4 Hz (120 BPM / 60 * 2 beats per half note / 4 eighth notes)
        let pulse_rate = 4.0; // Hz

        // Create a square wave LFO for the sharp on/off pulsing effect
        let lfo = SquareOscillator::<SAMPLE_RATE>::new(pulse_rate);

        // Map the square wave (-1 to 1) to cutoff frequency
        // When LFO is high (1): cutoff at ~4000Hz (open filter)
        // When LFO is low (-1): cutoff at ~50Hz (closed filter, dark)
        // This creates the dramatic "drop" effect
        let modulated_cutoff = lfo
            .gain(1975.0) // Scale: 2000Hz range
            .offset(2025.0); // Offset: centered at 2025Hz (50Hz to 4000Hz)

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

impl ExampleAudioState for DeadmauFilter {
    fn next_sample(&mut self) -> f64 {
        Signal::next_sample(self)
    }
}

fn draw_ui() -> Result<()> {
    let mut stdout = stdout();
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

fn main() -> Result<()> {
    run_interactive_example(
        DeadmauFilter::new(),
        KeyboardConfig::default(),
        |_state| draw_ui(),
        |_state, key_event: &KeyEvent| {
            if is_quit_key(key_event.code) {
                Ok(KeyAction::Exit)
            } else {
                Ok(KeyAction::Continue)
            }
        },
    )
}
