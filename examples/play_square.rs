mod common;

use earworm::SquareOscillator;

fn main() -> Result<(), anyhow::Error> {
    // Create oscillator at 440 Hz (A4 note) with 50% duty cycle
    let oscillator = SquareOscillator::new(440.0, 44100.0);

    // Play it for 5 seconds
    common::play_oscillator(oscillator, "square", 5)
}
