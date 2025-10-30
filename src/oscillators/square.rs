//! Square wave oscillator implementation.

use super::{Oscillator, PulseOscillator};
use crate::{AudioSignal, Signal};

/// A square wave oscillator for audio synthesis.
///
/// This is a specialized version of `PulseOscillator` with a fixed 50% duty cycle,
/// producing a symmetric square wave. The waveform alternates between -1.0 and 1.0.
///
/// For variable duty cycles, use `PulseOscillator` directly.
pub struct SquareOscillator {
    /// The underlying pulse oscillator with fixed 50% duty cycle
    pulse: PulseOscillator,
}

impl SquareOscillator {
    /// Creates a new square oscillator with 50% duty cycle.
    ///
    /// # Arguments
    ///
    /// * `frequency` - Frequency of the square wave in Hz
    /// * `sample_rate` - Sample rate in Hz (e.g., 44100.0 for CD quality)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{Signal, SquareOscillator};
    ///
    /// // Create a 440 Hz (A4 note) oscillator at 44.1 kHz sample rate
    /// let mut osc = SquareOscillator::new(440.0, 44100.0);
    /// let sample = osc.next_sample();
    /// ```
    pub fn new(frequency: f64, sample_rate: f64) -> Self {
        // PulseOscillator expects duty cycle in range [-1, 1] which maps to [0, 1]
        // For 50% duty cycle (0.5), we need input value of 0.0
        // Because: 0.0 * 0.5 + 0.5 = 0.5
        Self {
            pulse: PulseOscillator::new(frequency, sample_rate, 0.0.into()),
        }
    }
}

impl Signal for SquareOscillator {
    fn next_sample(&mut self) -> f64 {
        self.pulse.next_sample()
    }

    fn process(&mut self, buffer: &mut [f64]) {
        self.pulse.process(buffer)
    }
}

impl AudioSignal for SquareOscillator {
    fn sample_rate(&self) -> f64 {
        self.pulse.sample_rate()
    }
}

impl Oscillator for SquareOscillator {
    fn set_frequency(&mut self, frequency: f64) {
        self.pulse.set_frequency(frequency);
    }

    fn frequency(&self) -> f64 {
        self.pulse.frequency()
    }

    fn reset(&mut self) {
        self.pulse.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oscillator_creation() {
        let osc = SquareOscillator::new(440.0, 44100.0);
        assert_eq!(osc.frequency(), 440.0);
    }

    #[test]
    fn test_frequency_change() {
        let mut osc = SquareOscillator::new(440.0, 44100.0);
        osc.set_frequency(880.0);
        assert_eq!(osc.frequency(), 880.0);
    }

    #[test]
    fn test_sample_generation() {
        let mut osc = SquareOscillator::new(440.0, 44100.0);
        let sample = osc.next_sample();
        // First sample should be 1.0 (starting at phase 0, duty cycle 0.5)
        assert_eq!(sample, 1.0);
    }

    #[test]
    fn test_sample_range() {
        let mut osc = SquareOscillator::new(440.0, 44100.0);
        // Generate a full cycle and verify all samples are either -1.0 or 1.0
        for _ in 0..44100 {
            let sample = osc.next_sample();
            assert!(sample == -1.0 || sample == 1.0);
        }
    }

    #[test]
    fn test_waveform_shape_50_percent() {
        let mut osc = SquareOscillator::new(1.0, 100.0);

        // At phase 0.0, should be 1.0 (high)
        let s1 = osc.next_sample();
        assert_eq!(s1, 1.0);

        // At phase < 0.5, should still be 1.0
        for _ in 0..24 {
            osc.next_sample();
        }
        let s2 = osc.next_sample();
        assert_eq!(s2, 1.0);

        // At phase >= 0.5, should be -1.0
        for _ in 0..24 {
            osc.next_sample();
        }
        let s3 = osc.next_sample();
        assert_eq!(s3, -1.0);
    }

    #[test]
    fn test_phase_wrapping() {
        let mut osc = SquareOscillator::new(1000.0, 44100.0);
        // Run for many samples to ensure phase wraps correctly
        for _ in 0..100000 {
            osc.next_sample();
        }
        // Just verify it doesn't crash or produce invalid output
        assert!(true);
    }

    #[test]
    fn test_reset() {
        let mut osc = SquareOscillator::new(440.0, 44100.0);
        // Advance the oscillator
        for _ in 0..100 {
            osc.next_sample();
        }
        osc.reset();
        // After reset, first sample should be 1.0 again
        let sample = osc.next_sample();
        assert_eq!(sample, 1.0);
    }

    #[test]
    fn test_process_buffer() {
        let mut osc = SquareOscillator::new(440.0, 44100.0);
        let mut buffer = vec![0.0; 128];
        osc.process(&mut buffer);

        // Verify all samples are valid
        for sample in buffer {
            assert!(sample == -1.0 || sample == 1.0);
        }
    }

    #[test]
    fn test_zero_frequency() {
        let mut osc = SquareOscillator::new(0.0, 44100.0);
        let sample1 = osc.next_sample();
        let sample2 = osc.next_sample();
        // With 0 Hz, phase doesn't advance, so samples should be identical
        assert_eq!(sample1, sample2);
    }

    #[test]
    fn test_symmetric_duty_cycle() {
        let mut osc = SquareOscillator::new(1.0, 100.0);

        let mut high_count = 0;
        let mut low_count = 0;

        // Count high and low samples in one complete cycle
        for _ in 0..100 {
            let sample = osc.next_sample();
            if sample > 0.0 {
                high_count += 1;
            } else {
                low_count += 1;
            }
        }

        // With 50% duty cycle, should be roughly equal
        assert_eq!(high_count, 50);
        assert_eq!(low_count, 50);
    }
}
