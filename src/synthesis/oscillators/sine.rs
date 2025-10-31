//! Sine wave oscillator implementation.

use super::Oscillator;
use crate::core::Pitched;
use crate::{AudioSignal, Signal};
use std::f64::consts::PI;

/// A simple sine wave oscillator for audio synthesis.
///
/// This oscillator generates a continuous sine wave at a specified frequency.
/// It maintains phase continuity across calls to `next_sample()`.
///
/// # Type Parameters
///
/// * `SAMPLE_RATE` - Sample rate in Hz (e.g., 44100 for CD quality)
pub struct SineOscillator<const SAMPLE_RATE: u32> {
    /// Current phase of the oscillator (0.0 to 1.0)
    phase: f64,
    /// Phase increment per sample (frequency / sample_rate)
    phase_increment: f64,
}

impl<const SAMPLE_RATE: u32> SineOscillator<SAMPLE_RATE> {
    /// Creates a new sine oscillator.
    ///
    /// # Arguments
    ///
    /// * `frequency` - Frequency of the sine wave in Hz
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{Signal, SineOscillator};
    ///
    /// // Create a 440 Hz (A4 note) oscillator at 44.1 kHz sample rate
    /// let mut osc = SineOscillator::<44100>::new(440.0);
    /// let sample = osc.next_sample();
    /// ```
    pub fn new(frequency: f64) -> Self {
        let phase_increment = frequency / SAMPLE_RATE as f64;
        Self {
            phase: 0.0,
            phase_increment,
        }
    }
}

impl<const SAMPLE_RATE: u32> Signal for SineOscillator<SAMPLE_RATE> {
    fn next_sample(&mut self) -> f64 {
        // Generate sine wave sample
        let sample = (self.phase * 2.0 * PI).sin();

        // Increment phase and wrap to [0.0, 1.0)
        self.phase += self.phase_increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        sample
    }

    // Uses default implementation of process() from the trait
}

impl<const SAMPLE_RATE: u32> AudioSignal<SAMPLE_RATE> for SineOscillator<SAMPLE_RATE> {}

impl<const SAMPLE_RATE: u32> Pitched for SineOscillator<SAMPLE_RATE> {
    fn set_frequency(&mut self, frequency: f64) {
        self.phase_increment = frequency / SAMPLE_RATE as f64;
    }

    fn frequency(&self) -> f64 {
        self.phase_increment * SAMPLE_RATE as f64
    }
}

impl<const SAMPLE_RATE: u32> Oscillator for SineOscillator<SAMPLE_RATE> {
    fn reset(&mut self) {
        self.phase = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oscillator_creation() {
        let osc = SineOscillator::<44100>::new(440.0);
        assert_eq!(osc.frequency(), 440.0);
    }

    #[test]
    fn test_frequency_change() {
        let mut osc = SineOscillator::<44100>::new(440.0);
        osc.set_frequency(880.0);
        assert_eq!(osc.frequency(), 880.0);
    }

    #[test]
    fn test_sample_generation() {
        let mut osc = SineOscillator::<44100>::new(440.0);
        let sample = osc.next_sample();
        // First sample should be close to 0 (starting at phase 0)
        assert!(sample.abs() < 0.1);
    }

    #[test]
    fn test_sample_range() {
        let mut osc = SineOscillator::<44100>::new(440.0);
        // Generate a full cycle and verify all samples are in [-1.0, 1.0]
        for _ in 0..44100 {
            let sample = osc.next_sample();
            assert!((-1.0..=1.0).contains(&sample));
        }
    }

    #[test]
    fn test_phase_wrapping() {
        let mut osc = SineOscillator::<44100>::new(1000.0);
        // Run for many samples to ensure phase wraps correctly
        for _ in 0..100000 {
            osc.next_sample();
        }
        // Phase should still be in valid range
        assert!(osc.phase >= 0.0 && osc.phase < 1.0);
    }

    #[test]
    fn test_reset() {
        let mut osc = SineOscillator::<44100>::new(440.0);
        // Advance the oscillator
        for _ in 0..100 {
            osc.next_sample();
        }
        osc.reset();
        assert_eq!(osc.phase, 0.0);
    }

    #[test]
    fn test_process_buffer() {
        let mut osc = SineOscillator::<44100>::new(440.0);
        let mut buffer = vec![0.0; 128];
        osc.process(&mut buffer);

        // Verify all samples are valid
        for sample in buffer {
            assert!((-1.0..=1.0).contains(&sample));
        }
    }

    #[test]
    fn test_zero_frequency() {
        let mut osc = SineOscillator::<44100>::new(0.0);
        let sample1 = osc.next_sample();
        let sample2 = osc.next_sample();
        // With 0 Hz, phase doesn't advance, so samples should be identical
        assert_eq!(sample1, sample2);
    }
}
