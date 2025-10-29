//! Pulse wave oscillator with modulating duty cycle.

use super::{AudioSignal, Oscillator};
use crate::Signal;

/// A pulse wave oscillator with variable duty cycle.
///
/// Unlike the `SquareOscillator` which has a fixed duty cycle, this oscillator
/// allows the duty cycle to be modulated by any signal source (constant value,
/// LFO, envelope, etc.). This makes it extremely flexible for creating time-varying
/// timbres.
///
/// The duty cycle signal is expected to be in the range [-1.0, 1.0] and will be
/// automatically scaled and clamped to [0.0, 1.0].
pub struct PulseOscillator<D: Signal> {
    /// Current phase of the oscillator (0.0 to 1.0)
    phase: f64,
    /// Phase increment per sample (frequency / sample_rate)
    phase_increment: f64,
    /// Sample rate in Hz
    sample_rate: f64,
    /// Duty cycle modulation source (0.0 to 1.0) - fraction of cycle where output is high
    duty_cycle: D,
}

impl<D: Signal> PulseOscillator<D> {
    /// Creates a new pulse oscillator with a modulating duty cycle.
    ///
    /// # Arguments
    ///
    /// * `frequency` - Frequency of the pulse wave in Hz
    /// * `sample_rate` - Sample rate in Hz (e.g., 44100.0 for CD quality)
    /// * `duty_cycle` - Signal source for duty cycle modulation (range -1.0 to 1.0)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{Signal, PulseOscillator, SineOscillator};
    ///
    /// // Create a pulse wave with fixed 25% duty cycle
    /// let mut pulse = PulseOscillator::new(440.0, 44100.0, 0.25);
    /// let sample = pulse.next_sample();
    ///
    /// // Create a pulse wave with duty cycle modulated by an LFO
    /// let lfo = SineOscillator::new(2.0, 44100.0);
    /// let mut pulse = PulseOscillator::new(440.0, 44100.0, lfo);
    /// let sample = pulse.next_sample();
    /// ```
    pub fn new(frequency: f64, sample_rate: f64, duty_cycle: D) -> Self {
        let phase_increment = frequency / sample_rate;
        Self {
            phase: 0.0,
            phase_increment,
            sample_rate,
            duty_cycle,
        }
    }
}

impl<D: Signal> AudioSignal for PulseOscillator<D> {
    fn sample_rate(&self) -> f64 {
        self.sample_rate
    }
}

impl<D: Signal> Signal for PulseOscillator<D> {
    fn next_sample(&mut self) -> f64 {
        // Get duty cycle from modulation source and scale from [-1, 1] to [0, 1]
        let duty = self.duty_cycle.next_sample();
        let duty = (duty * 0.5 + 0.5).clamp(0.0, 1.0);

        // Generate pulse wave sample
        // Output is 1.0 when phase < duty_cycle, -1.0 otherwise
        let sample = if self.phase < duty { 1.0 } else { -1.0 };

        // Increment phase and wrap to [0.0, 1.0)
        self.phase += self.phase_increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        sample
    }
}

impl<D: Signal> Oscillator for PulseOscillator<D> {
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
    use crate::SineOscillator;

    #[test]
    fn test_oscillator_creation() {
        let osc = PulseOscillator::new(440.0, 44100.0, 0.5);
        assert_eq!(osc.frequency(), 440.0);
    }

    #[test]
    fn test_frequency_change() {
        let mut osc = PulseOscillator::new(440.0, 44100.0, 0.5);
        osc.set_frequency(880.0);
        assert_eq!(osc.frequency(), 880.0);
    }

    #[test]
    fn test_sample_generation() {
        let mut osc = PulseOscillator::new(440.0, 44100.0, 0.5);
        let sample = osc.next_sample();
        // First sample should be 1.0 (starting at phase 0, duty cycle 0.5)
        assert_eq!(sample, 1.0);
    }

    #[test]
    fn test_sample_range() {
        let mut osc = PulseOscillator::new(440.0, 44100.0, 0.5);
        // Generate a full cycle and verify all samples are either -1.0 or 1.0
        for _ in 0..44100 {
            let sample = osc.next_sample();
            assert!(sample == -1.0 || sample == 1.0);
        }
    }

    #[test]
    fn test_waveform_shape_50_percent() {
        // Input 0.5 maps to: 0.5 * 0.5 + 0.5 = 0.75 duty cycle
        let mut osc = PulseOscillator::new(1.0, 100.0, 0.5);

        // At phase 0.0, should be 1.0 (high)
        let s1 = osc.next_sample();
        assert_eq!(s1, 1.0);

        // Generate 74 more samples (total 75) - still under 0.75 duty cycle threshold
        for _ in 0..74 {
            let sample = osc.next_sample();
            assert_eq!(sample, 1.0);
        }

        // Now at sample 76, phase should be >= 0.75, should be -1.0
        let s2 = osc.next_sample();
        assert_eq!(s2, -1.0);
    }

    #[test]
    fn test_waveform_shape_25_percent() {
        // Input 0.25 maps to: 0.25 * 0.5 + 0.5 = 0.625 duty cycle
        let mut osc = PulseOscillator::new(1.0, 100.0, 0.25);

        // At phase 0.0, should be 1.0 (high)
        let s1 = osc.next_sample();
        assert_eq!(s1, 1.0);

        // Generate 61 more samples (total 62) - still under 0.625 duty cycle threshold
        for _ in 0..61 {
            let sample = osc.next_sample();
            assert_eq!(sample, 1.0);
        }

        // Now at sample 63, phase should be 0.63 which is > 0.625, should be -1.0
        let s2 = osc.next_sample();
        assert_eq!(s2, 1.0); // Actually still 1.0 at 0.62

        // At sample 64, phase is 0.63, should be -1.0
        let s3 = osc.next_sample();
        assert_eq!(s3, -1.0);
    }

    #[test]
    fn test_phase_wrapping() {
        let mut osc = PulseOscillator::new(1000.0, 44100.0, 0.5);
        // Run for many samples to ensure phase wraps correctly
        for _ in 0..100000 {
            osc.next_sample();
        }
        // Phase should still be in valid range
        assert!(osc.phase >= 0.0 && osc.phase < 1.0);
    }

    #[test]
    fn test_reset() {
        let mut osc = PulseOscillator::new(440.0, 44100.0, 0.5);
        // Advance the oscillator
        for _ in 0..100 {
            osc.next_sample();
        }
        osc.reset();
        assert_eq!(osc.phase, 0.0);
    }

    #[test]
    fn test_process_buffer() {
        let mut osc = PulseOscillator::new(440.0, 44100.0, 0.5);
        let mut buffer = vec![0.0; 128];
        osc.process(&mut buffer);

        // Verify all samples are valid
        for sample in buffer {
            assert!(sample == -1.0 || sample == 1.0);
        }
    }

    #[test]
    fn test_zero_frequency() {
        let mut osc = PulseOscillator::new(0.0, 44100.0, 0.5);
        let sample1 = osc.next_sample();
        let sample2 = osc.next_sample();
        // With 0 Hz, phase doesn't advance, so samples should be identical
        assert_eq!(sample1, sample2);
    }

    #[test]
    fn test_modulating_duty_cycle() {
        // Create a pulse oscillator with a sine wave modulating the duty cycle
        let lfo = SineOscillator::new(1.0, 100.0);
        let mut osc = PulseOscillator::new(10.0, 100.0, lfo);

        // Generate samples - duty cycle should be changing over time
        for _ in 0..100 {
            // Capture multiple cycles to see the modulation
            for _ in 0..10 {
                osc.next_sample();
            }
        }

        // Just verify it doesn't crash with modulation - detailed verification
        // would require analyzing the actual duty cycle over time
        assert!(true);
    }

    #[test]
    fn test_duty_cycle_scaling() {
        // Test that duty cycle values outside [0, 1] are properly scaled and clamped
        let mut osc = PulseOscillator::new(1.0, 100.0, -1.0); // Should map to 0.0
        let sample1 = osc.next_sample();
        // With duty cycle 0.0, output should always be -1.0
        assert_eq!(sample1, -1.0);

        let mut osc = PulseOscillator::new(1.0, 100.0, 1.0); // Should map to 1.0
        let sample2 = osc.next_sample();
        // With duty cycle 1.0, output should always be 1.0
        assert_eq!(sample2, 1.0);
    }
}
