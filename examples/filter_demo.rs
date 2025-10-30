use earworm::{filters::BiquadFilter, Signal, SignalExt, SineOscillator};

fn main() {
    let sample_rate = 44100.0;

    println!("Biquad Filter Examples\n");

    // Example 1: Low-pass filter
    println!("1. Low-pass filter (cutoff: 1kHz, Q: 0.707)");
    let osc = SineOscillator::new(440.0, sample_rate);
    let mut lowpass = BiquadFilter::lowpass(osc, 1000.0, 0.707, sample_rate);

    print!("   Samples: ");
    for _ in 0..10 {
        print!("{:.3} ", lowpass.next_sample());
    }
    println!("\n");

    // Example 2: High-pass filter
    println!("2. High-pass filter (cutoff: 1kHz, Q: 0.707)");
    let osc = SineOscillator::new(440.0, sample_rate);
    let mut highpass = BiquadFilter::highpass(osc, 1000.0, 0.707, sample_rate);

    print!("   Samples: ");
    for _ in 0..10 {
        print!("{:.3} ", highpass.next_sample());
    }
    println!("\n");

    // Example 3: Band-pass filter
    println!("3. Band-pass filter (center: 1kHz, Q: 5.0)");
    let osc = SineOscillator::new(1000.0, sample_rate);
    let mut bandpass = BiquadFilter::bandpass(osc, 1000.0, 5.0, sample_rate);

    // Let it settle first
    for _ in 0..100 {
        bandpass.next_sample();
    }

    print!("   Samples: ");
    for _ in 0..10 {
        print!("{:.3} ", bandpass.next_sample());
    }
    println!("\n");

    // Example 4: Resonant low-pass (high Q)
    println!("4. Resonant low-pass filter (cutoff: 800Hz, Q: 10.0)");
    let osc = SineOscillator::new(800.0, sample_rate);
    let mut resonant = BiquadFilter::lowpass(osc, 800.0, 10.0, sample_rate);

    // Let it settle
    for _ in 0..100 {
        resonant.next_sample();
    }

    print!("   Samples: ");
    for _ in 0..10 {
        print!("{:.3} ", resonant.next_sample());
    }
    println!("\n");

    // Example 5: Modulated filter (filter sweep)
    println!("5. Filter sweep with LFO modulation");
    let osc = SineOscillator::new(440.0, sample_rate);
    let lfo = SineOscillator::new(0.5, sample_rate);

    // LFO sweeps cutoff from 500Hz to 2000Hz
    let modulated_cutoff = lfo.gain(750.0).offset(1250.0);

    let mut swept_filter = BiquadFilter::lowpass(osc, modulated_cutoff, 2.0, sample_rate);

    print!("   First 10 samples: ");
    for _ in 0..10 {
        print!("{:.3} ", swept_filter.next_sample());
    }
    println!();

    // Skip ahead to show the sweep
    for _ in 0..22000 {
        swept_filter.next_sample();
    }

    print!("   After sweep:      ");
    for _ in 0..10 {
        print!("{:.3} ", swept_filter.next_sample());
    }
    println!("\n");

    // Example 6: Chain multiple filters
    println!("6. Chained filters (bandpass -> lowpass)");
    let osc = SineOscillator::new(880.0, sample_rate);
    let bandpass = BiquadFilter::bandpass(osc, 1000.0, 3.0, sample_rate);
    let mut chain = BiquadFilter::lowpass(bandpass, 1200.0, 0.707, sample_rate);

    // Let it settle
    for _ in 0..200 {
        chain.next_sample();
    }

    print!("   Samples: ");
    for _ in 0..10 {
        print!("{:.3} ", chain.next_sample());
    }
    println!("\n");

    // Example 7: Notch filter
    println!("7. Notch filter removing 1kHz (Q: 10.0)");
    let osc = SineOscillator::new(1000.0, sample_rate);
    let mut notch = BiquadFilter::notch(osc, 1000.0, 10.0, sample_rate);

    // Let it settle
    for _ in 0..1000 {
        notch.next_sample();
    }

    print!("   Samples: ");
    for _ in 0..10 {
        print!("{:.3} ", notch.next_sample());
    }
    println!("\n");

    println!("Filter demonstration complete!");
}
