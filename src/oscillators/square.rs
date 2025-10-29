//! Square wave oscillator implementation.

use super::{AudioSignal, Oscillator};
use crate::Signal;

/// A square wave oscillator for audio synthesis.
///
/// This oscillator generates a continuous square wave at a specified frequency.
/// The waveform alternates between -1.0 and 1.0 with a configurable duty cycle.
/// A duty cycle of 0.5 (50%) produces a symmetric square wave.
/// It maintains phase continuity across calls to `next_sample()`.
pub struct SquareOscillator {
    /// Current phase of the oscillator (0.0 to 1.0)
    phase: f64,
    /// Phase increment per sample (frequency / sample_rate)
    phase_increment: f64,
    /// Sample rate in Hz
    sample_rate: f64,
    /// Duty cycle (0.0 to 1.0) - fraction of cycle where output is high
    duty_cycle: f64,
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
        Self::new_with_duty_cycle(frequency, sample_rate, 0.5)
    }

    /// Creates a new square oscillator with a custom duty cycle.
    ///
    /// # Arguments
    ///
    /// * `frequency` - Frequency of the square wave in Hz
    /// * `sample_rate` - Sample rate in Hz (e.g., 44100.0 for CD quality)
    /// * `duty_cycle` - Duty cycle between 0.0 and 1.0 (0.5 = 50% duty cycle)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{Signal, SquareOscillator};
    ///
    /// // Create a 440 Hz pulse wave with 25% duty cycle
    /// let mut osc = SquareOscillator::new_with_duty_cycle(440.0, 44100.0, 0.25);
    /// let sample = osc.next_sample();
    /// ```
    pub fn new_with_duty_cycle(frequency: f64, sample_rate: f64, duty_cycle: f64) -> Self {
        let phase_increment = frequency / sample_rate;
        let duty_cycle = duty_cycle.clamp(0.0, 1.0);
        Self {
            phase: 0.0,
            phase_increment,
            sample_rate,
            duty_cycle,
        }
    }

    /// Sets the duty cycle of the square wave.
    ///
    /// # Arguments
    ///
    /// * `duty_cycle` - Duty cycle between 0.0 and 1.0 (0.5 = 50% duty cycle)
    pub fn set_duty_cycle(&mut self, duty_cycle: f64) {
        self.duty_cycle = duty_cycle.clamp(0.0, 1.0);
    }

    /// Gets the current duty cycle.
    ///
    /// # Returns
    ///
    /// Current duty cycle between 0.0 and 1.0
    pub fn duty_cycle(&self) -> f64 {
        self.duty_cycle
    }
}

impl Signal for SquareOscillator {
    fn next_sample(&mut self) -> f64 {
        // Generate square wave sample
        // Output is 1.0 when phase < duty_cycle, -1.0 otherwise
        let sample = if self.phase < self.duty_cycle {
            1.0
        } else {
            -1.0
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

impl AudioSignal for SquareOscillator {
    fn sample_rate(&self) -> f64 {
        self.sample_rate
    }
}

impl Oscillator for SquareOscillator {
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
        let osc = SquareOscillator::new(440.0, 44100.0);
        assert_eq!(osc.frequency(), 440.0);
        assert_eq!(osc.duty_cycle(), 0.5);
    }

    #[test]
    fn test_oscillator_creation_with_duty_cycle() {
        let osc = SquareOscillator::new_with_duty_cycle(440.0, 44100.0, 0.25);
        assert_eq!(osc.frequency(), 440.0);
        assert_eq!(osc.duty_cycle(), 0.25);
    }

    #[test]
    fn test_frequency_change() {
        let mut osc = SquareOscillator::new(440.0, 44100.0);
        osc.set_frequency(880.0);
        assert_eq!(osc.frequency(), 880.0);
    }

    #[test]
    fn test_duty_cycle_change() {
        let mut osc = SquareOscillator::new(440.0, 44100.0);
        osc.set_duty_cycle(0.75);
        assert_eq!(osc.duty_cycle(), 0.75);
    }

    #[test]
    fn test_duty_cycle_clamping() {
        let mut osc = SquareOscillator::new(440.0, 44100.0);
        osc.set_duty_cycle(1.5);
        assert_eq!(osc.duty_cycle(), 1.0);
        osc.set_duty_cycle(-0.5);
        assert_eq!(osc.duty_cycle(), 0.0);
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
    fn test_waveform_shape_25_percent() {
        let mut osc = SquareOscillator::new_with_duty_cycle(1.0, 100.0, 0.25);

        // At phase 0.0, should be 1.0 (high)
        let s1 = osc.next_sample();
        assert_eq!(s1, 1.0);

        // At phase < 0.25, should still be 1.0
        for _ in 0..19 {
            osc.next_sample();
        }
        let s2 = osc.next_sample();
        assert_eq!(s2, 1.0);

        // At phase >= 0.25, should be -1.0
        for _ in 0..5 {
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
        // Phase should still be in valid range
        assert!(osc.phase >= 0.0 && osc.phase < 1.0);
    }

    #[test]
    fn test_reset() {
        let mut osc = SquareOscillator::new(440.0, 44100.0);
        // Advance the oscillator
        for _ in 0..100 {
            osc.next_sample();
        }
        osc.reset();
        assert_eq!(osc.phase, 0.0);
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
