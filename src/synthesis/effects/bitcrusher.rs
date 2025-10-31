//! Bitcrusher effect for lo-fi digital degradation.

use crate::core::{AudioSignal, Param, Signal};

/// Bitcrusher effect that reduces sample rate and bit depth.
///
/// Creates lo-fi digital degradation by simulating lower quality audio:
/// - Sample rate reduction creates a "sample and hold" effect
/// - Bit depth reduction creates quantization distortion
pub struct Bitcrusher<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> {
    source: S,
    sample_rate_reduction: Param, // 1.0 = no reduction, 8.0 = 1/8 reduction
    bit_depth: Param,             // bits of resolution (e.g., 8.0 for 8-bit)
    hold_counter: f64,
    held_sample: f64,
}

impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> Bitcrusher<SAMPLE_RATE, S> {
    /// Creates a new bitcrusher effect.
    ///
    /// # Arguments
    ///
    /// * `source` - Input signal
    /// * `sample_rate_reduction` - Sample rate divisor (1.0 = no reduction, 2.0 = half rate, etc.)
    /// * `bit_depth` - Bit depth for quantization (e.g., 8.0 for 8-bit, 4.0 for 4-bit)
    pub fn new(
        source: S,
        sample_rate_reduction: impl Into<Param>,
        bit_depth: impl Into<Param>,
    ) -> Self {
        Self {
            source,
            sample_rate_reduction: sample_rate_reduction.into(),
            bit_depth: bit_depth.into(),
            hold_counter: f64::INFINITY, // Start with infinity to capture first sample
            held_sample: 0.0,
        }
    }
}

impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> Signal for Bitcrusher<SAMPLE_RATE, S> {
    fn next_sample(&mut self) -> f64 {
        let current_sample = self.source.next_sample();

        // Check if we should update the held sample
        if self.hold_counter >= self.sample_rate_reduction.value().max(1.0) {
            self.held_sample = current_sample;
            self.hold_counter = 0.0;
        }

        self.hold_counter += 1.0;

        let levels = 2.0_f64.powf(self.bit_depth.value());
        (self.held_sample * levels).round() / levels
    }
}

impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> AudioSignal<SAMPLE_RATE>
    for Bitcrusher<SAMPLE_RATE, S>
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::combinators::SignalExt;

    // Helper to create a simple test signal
    struct TestSignal<const SAMPLE_RATE: u32> {
        values: Vec<f64>,
        index: usize,
    }

    impl<const SAMPLE_RATE: u32> TestSignal<SAMPLE_RATE> {
        fn new(values: Vec<f64>) -> Self {
            Self { values, index: 0 }
        }
    }

    impl<const SAMPLE_RATE: u32> Signal for TestSignal<SAMPLE_RATE> {
        fn next_sample(&mut self) -> f64 {
            let value = self.values[self.index % self.values.len()];
            self.index += 1;
            value
        }
    }

    impl<const SAMPLE_RATE: u32> AudioSignal<SAMPLE_RATE> for TestSignal<SAMPLE_RATE> {}

    #[test]
    fn test_no_reduction() {
        // With sample_rate_reduction = 1.0, every sample should pass through
        let signal = TestSignal::<44100>::new(vec![0.1, 0.2, 0.3, 0.4]);
        let mut crusher = Bitcrusher::new(signal, 1.0, 16.0);

        assert!((crusher.next_sample() - 0.1).abs() < 0.001);
        assert!((crusher.next_sample() - 0.2).abs() < 0.001);
        assert!((crusher.next_sample() - 0.3).abs() < 0.001);
        assert!((crusher.next_sample() - 0.4).abs() < 0.001);
    }

    #[test]
    fn test_sample_rate_reduction() {
        // With sample_rate_reduction = 2.0, every other sample should be held
        let signal = TestSignal::<44100>::new(vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6]);
        let mut crusher = Bitcrusher::new(signal, 2.0, 16.0);

        // First sample is grabbed and held
        let s1 = crusher.next_sample();
        assert!((s1 - 0.1).abs() < 0.001);
        // Second sample is held (still 0.1)
        let s2 = crusher.next_sample();
        assert!((s2 - 0.1).abs() < 0.001);
        // Third sample is grabbed
        let s3 = crusher.next_sample();
        assert!((s3 - 0.3).abs() < 0.001);
        // Fourth sample is held (still 0.3)
        let s4 = crusher.next_sample();
        assert!((s4 - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_sample_rate_reduction_3x() {
        // With sample_rate_reduction = 3.0, hold for 3 samples
        let signal = TestSignal::<44100>::new(vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7]);
        let mut crusher = Bitcrusher::new(signal, 3.0, 16.0);

        let s1 = crusher.next_sample();
        let s2 = crusher.next_sample();
        let s3 = crusher.next_sample();
        assert!((s1 - 0.1).abs() < 0.001);
        assert!((s2 - 0.1).abs() < 0.001);
        assert!((s3 - 0.1).abs() < 0.001);

        let s4 = crusher.next_sample();
        let s5 = crusher.next_sample();
        let s6 = crusher.next_sample();
        assert!((s4 - 0.4).abs() < 0.001);
        assert!((s5 - 0.4).abs() < 0.001);
        assert!((s6 - 0.4).abs() < 0.001);
    }

    #[test]
    fn test_bit_depth_reduction() {
        // Test quantization with low bit depth
        let signal = TestSignal::<44100>::new(vec![0.5]);
        let mut crusher = Bitcrusher::new(signal, 1.0, 2.0); // 2-bit = 4 levels

        // With 2 bits, we have 4 levels: 0.0, 0.25, 0.5, 0.75, 1.0
        // 0.5 should quantize to 0.5
        let sample = crusher.next_sample();
        assert!((sample - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_bit_depth_quantization_levels() {
        // Test that 8-bit gives 256 levels
        let signal = TestSignal::<44100>::new(vec![0.123456]);
        let mut crusher = Bitcrusher::new(signal, 1.0, 8.0);

        let sample = crusher.next_sample();
        // Should be quantized to one of 256 levels
        let levels = 2.0_f64.powf(8.0);
        let expected = (0.123456 * levels).round() / levels;
        assert!((sample - expected).abs() < 0.0001);
    }

    #[test]
    fn test_severe_bit_crushing() {
        // Test very low bit depth (1 bit = 2 levels: 0 or 1)
        let signal = TestSignal::<44100>::new(vec![0.3, -0.3]);
        let mut crusher = Bitcrusher::new(signal, 1.0, 1.0);

        let sample1 = crusher.next_sample();
        let sample2 = crusher.next_sample();

        // With 1 bit, we only have 2 levels: -1 and 1 (or close to it)
        assert!(sample1.abs() > 0.4); // Should be quantized to near +/- 0.5
        assert!(sample2.abs() > 0.4);
    }

    #[test]
    fn test_combined_effects() {
        // Test both sample rate reduction and bit depth reduction together
        let signal = TestSignal::<44100>::new(vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6]);
        let mut crusher = Bitcrusher::new(signal, 2.0, 4.0); // 2x rate reduction, 4-bit

        let sample1 = crusher.next_sample();
        let sample2 = crusher.next_sample();

        // First two samples should be the same (held)
        assert_eq!(sample1, sample2);

        // Should be quantized to 4-bit precision
        let levels = 2.0_f64.powf(4.0);
        let quantized_check = (sample1 * levels).round() / levels;
        assert!((sample1 - quantized_check).abs() < 0.0001);
    }

    #[test]
    fn test_sample_rate_reduction_clamp_below_one() {
        // Test that sample_rate_reduction < 1.0 behaves like 1.0
        let signal = TestSignal::<44100>::new(vec![0.1, 0.2, 0.3, 0.4]);
        let mut crusher = Bitcrusher::new(signal, 0.5, 16.0); // Invalid value < 1.0

        // Should behave like no reduction (every sample passes through)
        assert!((crusher.next_sample() - 0.1).abs() < 0.001);
        assert!((crusher.next_sample() - 0.2).abs() < 0.001);
        assert!((crusher.next_sample() - 0.3).abs() < 0.001);
        assert!((crusher.next_sample() - 0.4).abs() < 0.001);
    }

    #[test]
    fn test_audio_signal_trait() {
        let signal = TestSignal::<48000>::new(vec![0.5]);
        let crusher = Bitcrusher::new(signal, 2.0, 8.0);

        assert_eq!(crusher.sample_rate(), 48000.0);
    }

    #[test]
    fn test_modulated_parameters() {
        // Test that Param can be used for modulation
        use crate::SineOscillator;

        let signal = TestSignal::<44100>::new(vec![0.5; 100]);
        let lfo = SineOscillator::<44100>::new(1.0);

        // This should compile and run without panicking
        let mut crusher = Bitcrusher::new(signal, Param::modulated(lfo.gain(2.0).offset(3.0)), 8.0);

        // Just verify it runs
        for _ in 0..10 {
            crusher.next_sample();
        }
    }
}
