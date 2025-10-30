//! Triangle wave oscillator implementation.

use super::Oscillator;
use crate::{AudioSignal, Signal};

/// A triangle wave oscillator for audio synthesis.
///
/// This oscillator generates a continuous triangle wave at a specified frequency.
/// The waveform rises linearly from -1.0 to 1.0, then falls linearly back to -1.0.
/// It maintains phase continuity across calls to `next_sample()`.
pub struct TriangleOscillator {
    /// Current phase of the oscillator (0.0 to 1.0)
    phase: f64,
    /// Phase increment per sample (frequency / sample_rate)
    phase_increment: f64,
    /// Sample rate in Hz
    sample_rate: f64,
}

impl TriangleOscillator {
    /// Creates a new triangle oscillator.
    ///
    /// # Arguments
    ///
    /// * `frequency` - Frequency of the triangle wave in Hz
    /// * `sample_rate` - Sample rate in Hz (e.g., 44100.0 for CD quality)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{Signal, TriangleOscillator};
    ///
    /// // Create a 440 Hz (A4 note) oscillator at 44.1 kHz sample rate
    /// let mut osc = TriangleOscillator::new(440.0, 44100.0);
    /// let sample = osc.next_sample();
    /// ```
    pub fn new(frequency: f64, sample_rate: f64) -> Self {
        let phase_increment = frequency / sample_rate;
        Self {
            phase: 0.0,
            phase_increment,
            sample_rate,
        }
    }
}

impl Signal for TriangleOscillator {
    fn next_sample(&mut self) -> f64 {
        // Generate triangle wave sample
        // Triangle wave: rises from -1 to 1 in first half, falls from 1 to -1 in second half
        let sample = if self.phase < 0.5 {
            // Rising: -1.0 to 1.0 over phase 0.0 to 0.5
            4.0 * self.phase - 1.0
        } else {
            // Falling: 1.0 to -1.0 over phase 0.5 to 1.0
            3.0 - 4.0 * self.phase
        };

        // Increment phase and wrap to [0.0, 1.0)
        self.phase += self.phase_increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        sample
    }

    // Uses default implementation of process() from the trait
}

impl AudioSignal for TriangleOscillator {
    fn sample_rate(&self) -> f64 {
        self.sample_rate
    }
}

impl Oscillator for TriangleOscillator {
    fn set_frequency(&mut self, frequency: f64) {
        self.phase_increment = frequency / self.sample_rate;
    }

    fn frequency(&self) -> f64 {
        self.phase_increment * self.sample_rate
    }

    fn reset(&mut self) {
        self.phase = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oscillator_creation() {
        let osc = TriangleOscillator::new(440.0, 44100.0);
        assert_eq!(osc.frequency(), 440.0);
    }

    #[test]
    fn test_frequency_change() {
        let mut osc = TriangleOscillator::new(440.0, 44100.0);
        osc.set_frequency(880.0);
        assert_eq!(osc.frequency(), 880.0);
    }

    #[test]
    fn test_sample_generation() {
        let mut osc = TriangleOscillator::new(440.0, 44100.0);
        let sample = osc.next_sample();
        // First sample should be -1.0 (starting at phase 0)
        assert!((sample + 1.0).abs() < 0.01);
    }

    #[test]
    fn test_sample_range() {
        let mut osc = TriangleOscillator::new(440.0, 44100.0);
        // Generate a full cycle and verify all samples are in [-1.0, 1.0]
        for _ in 0..44100 {
            let sample = osc.next_sample();
            assert!(sample >= -1.0 && sample <= 1.0);
        }
    }

    #[test]
    fn test_waveform_shape() {
        let mut osc = TriangleOscillator::new(1.0, 100.0);

        // At phase 0.0, should be -1.0
        let s1 = osc.next_sample();
        assert!((s1 + 1.0).abs() < 0.1);

        // Skip to roughly phase 0.25 (should be around 0.0)
        for _ in 0..23 {
            osc.next_sample();
        }
        let s2 = osc.next_sample();
        assert!(s2.abs() < 0.1);

        // At phase 0.5, should be at peak (1.0)
        for _ in 0..24 {
            osc.next_sample();
        }
        let s3 = osc.next_sample();
        assert!((s3 - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_phase_wrapping() {
        let mut osc = TriangleOscillator::new(1000.0, 44100.0);
        // Run for many samples to ensure phase wraps correctly
        for _ in 0..100000 {
            osc.next_sample();
        }
        // Phase should still be in valid range
        assert!(osc.phase >= 0.0 && osc.phase < 1.0);
    }

    #[test]
    fn test_reset() {
        let mut osc = TriangleOscillator::new(440.0, 44100.0);
        // Advance the oscillator
        for _ in 0..100 {
            osc.next_sample();
        }
        osc.reset();
        assert_eq!(osc.phase, 0.0);
    }

    #[test]
    fn test_process_buffer() {
        let mut osc = TriangleOscillator::new(440.0, 44100.0);
        let mut buffer = vec![0.0; 128];
        osc.process(&mut buffer);

        // Verify all samples are valid
        for sample in buffer {
            assert!(sample >= -1.0 && sample <= 1.0);
        }
    }

    #[test]
    fn test_zero_frequency() {
        let mut osc = TriangleOscillator::new(0.0, 44100.0);
        let sample1 = osc.next_sample();
        let sample2 = osc.next_sample();
        // With 0 Hz, phase doesn't advance, so samples should be identical
        assert_eq!(sample1, sample2);
    }

    #[test]
    fn test_linearity() {
        let mut osc = TriangleOscillator::new(1.0, 1000.0);

        // Test rising edge linearity
        let s1 = osc.next_sample();
        let s2 = osc.next_sample();
        let s3 = osc.next_sample();

        let diff1 = s2 - s1;
        let diff2 = s3 - s2;

        // Differences should be equal (linear ramp)
        assert!((diff1 - diff2).abs() < 0.0001);
    }
}
