mod common;

use earworm::SawtoothOscillator;

fn main() -> Result<(), anyhow::Error> {
    // Create oscillator at 440 Hz (A4 note)
    let oscillator = SawtoothOscillator::new(440.0, 44100.0);

    // Play it for 5 seconds
    common::play_oscillator(oscillator, "sawtooth", 5)
}
