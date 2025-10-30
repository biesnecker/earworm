use earworm::{Signal, SignalExt, SineOscillator, SquareOscillator};

fn main() {
    let sample_rate = 44100.0;
    let frequency = 440.0;

    // Example 1: Clipping/Distortion
    println!("Example 1: Hard clipping distortion");
    let mut distorted = SineOscillator::new(frequency, sample_rate)
        .gain(2.0) // Drive the signal hot
        .clamp(-0.5, 0.5); // Hard clip at ±0.5

    for _ in 0..10 {
        print!("{:.3} ", distorted.next_sample());
    }
    println!("\n");

    // Example 2: Waveshaping with map
    println!("Example 2: Cubic waveshaping");
    let mut waveshaped = SineOscillator::new(frequency, sample_rate)
        .map(|x| x * x * x); // Cubic distortion

    for _ in 0..10 {
        print!("{:.3} ", waveshaped.next_sample());
    }
    println!("\n");

    // Example 3: Crossfading between two oscillators
    println!("Example 3: Crossfading sine and square");
    let sine = SineOscillator::new(frequency, sample_rate);
    let square = SquareOscillator::new(frequency, sample_rate);
    let mut morphing = sine.crossfade(square, 0.5); // 50/50 mix

    for _ in 0..10 {
        print!("{:.3} ", morphing.next_sample());
    }
    println!("\n");

    // Example 4: Ring modulation with min/max
    println!("Example 4: Min of two oscillators");
    let osc1 = SineOscillator::new(440.0, sample_rate);
    let osc2 = SineOscillator::new(880.0, sample_rate);
    let mut min_signal = osc1.min(osc2);

    for _ in 0..10 {
        print!("{:.3} ", min_signal.next_sample());
    }
    println!("\n");

    // Example 5: Full-wave rectification
    println!("Example 5: Rectified sine wave");
    let mut rectified = SineOscillator::new(frequency, sample_rate).abs();

    for _ in 0..10 {
        print!("{:.3} ", rectified.next_sample());
    }
    println!("\n");

    // Example 6: Gated signal
    println!("Example 6: Noise gate effect");
    let mut gated = SineOscillator::new(frequency, sample_rate)
        .gain(0.5) // Quieter signal
        .gate(0.3); // Gate threshold at 0.3

    for _ in 0..10 {
        print!("{:.3} ", gated.next_sample());
    }
    println!("\n");

    // Example 7: Complex chain
    println!("Example 7: Complex processing chain");
    let osc = SineOscillator::new(frequency, sample_rate);
    let modulator = SineOscillator::new(2.0, sample_rate);

    let mut complex = osc
        .multiply(modulator) // Ring modulation
        .gain(1.5) // Boost
        .clamp(-0.8, 0.8) // Soft clip
        .map(|x| x.signum() * x.abs().powf(0.5)) // Compression
        .gain(0.7); // Final level

    for _ in 0..10 {
        print!("{:.3} ", complex.next_sample());
    }
    println!();
}
