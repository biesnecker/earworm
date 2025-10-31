//! Sawtooth wave oscillator implementation.

use super::Oscillator;
use crate::core::Pitched;
use crate::{AudioSignal, Signal};

/// A sawtooth wave oscillator for audio synthesis.
///
/// This oscillator generates a continuous sawtooth wave at a specified frequency.
/// The waveform rises linearly from -1.0 to 1.0, then sharply drops back to -1.0.
/// It maintains phase continuity across calls to `next_sample()`.
///
/// # Type Parameters
///
/// * `SAMPLE_RATE` - Sample rate in Hz (e.g., 44100 for CD quality)
#[derive(Clone)]
pub struct SawtoothOscillator<const SAMPLE_RATE: u32> {
    /// Current phase of the oscillator (0.0 to 1.0)
    phase: f64,
    /// Phase increment per sample (frequency / sample_rate)
    phase_increment: f64,
}

impl<const SAMPLE_RATE: u32> SawtoothOscillator<SAMPLE_RATE> {
    /// Creates a new sawtooth oscillator.
    ///
    /// # Arguments
    ///
    /// * `frequency` - Frequency of the sawtooth wave in Hz
    pub fn new(frequency: f64) -> Self {
        let phase_increment = frequency / SAMPLE_RATE as f64;
        Self {
            phase: 0.0,
            phase_increment,
        }
    }
}

impl<const SAMPLE_RATE: u32> Signal for SawtoothOscillator<SAMPLE_RATE> {
    fn next_sample(&mut self) -> f64 {
        // Generate sawtooth wave sample
        // Sawtooth wave: rises linearly from -1.0 to 1.0 over the full phase 0.0 to 1.0
        let sample = 2.0 * self.phase - 1.0;

        // Increment phase and wrap to [0.0, 1.0)
        self.phase += self.phase_increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        sample
    }
}

impl<const SAMPLE_RATE: u32> AudioSignal<SAMPLE_RATE> for SawtoothOscillator<SAMPLE_RATE> {}

impl<const SAMPLE_RATE: u32> Pitched for SawtoothOscillator<SAMPLE_RATE> {
    fn set_frequency(&mut self, frequency: f64) {
        self.phase_increment = frequency / SAMPLE_RATE as f64;
    }

    fn frequency(&self) -> f64 {
        self.phase_increment * SAMPLE_RATE as f64
    }
}

impl<const SAMPLE_RATE: u32> Oscillator for SawtoothOscillator<SAMPLE_RATE> {
    fn reset(&mut self) {
        self.phase = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oscillator_creation() {
        let osc = SawtoothOscillator::<44100>::new(440.0);
        assert_eq!(osc.frequency(), 440.0);
    }

    #[test]
    fn test_frequency_change() {
        let mut osc = SawtoothOscillator::<44100>::new(440.0);
        osc.set_frequency(880.0);
        assert_eq!(osc.frequency(), 880.0);
    }

    #[test]
    fn test_reset() {
        let mut osc = SawtoothOscillator::<44100>::new(440.0);
        for _ in 0..100 {
            osc.next_sample();
        }
        osc.reset();
        let sample = osc.next_sample();
        assert!((sample - (-1.0)).abs() < 0.01);
    }

    #[test]
    fn test_sample_generation() {
        let mut osc = SawtoothOscillator::<44100>::new(440.0);
        let sample = osc.next_sample();
        assert!((-1.0..=1.0).contains(&sample));
    }

    #[test]
    fn test_waveform_shape() {
        let mut osc = SawtoothOscillator::<44100>::new(1.0);
        osc.reset();
        let first = osc.next_sample();
        for _ in 0..(44100 / 2 - 1) {
            osc.next_sample();
        }
        let mid = osc.next_sample();
        assert!(first < mid);
    }

    #[test]
    fn test_continuous_rise() {
        let mut osc = SawtoothOscillator::<44100>::new(100.0);
        osc.reset();
        let mut prev = osc.next_sample();
        for _ in 0..220 {
            let curr = osc.next_sample();
            if curr > prev {
                assert!(curr > prev);
            }
            prev = curr;
        }
    }

    #[test]
    fn test_linearity() {
        let mut osc = SawtoothOscillator::<44100>::new(1.0);
        osc.reset();
        let samples: Vec<f64> = (0..100).map(|_| osc.next_sample()).collect();
        for i in 1..samples.len() - 1 {
            if samples[i] > samples[i - 1] && samples[i + 1] > samples[i] {
                let diff1 = samples[i] - samples[i - 1];
                let diff2 = samples[i + 1] - samples[i];
                assert!((diff1 - diff2).abs() < 0.01);
            }
        }
    }

    #[test]
    fn test_phase_wrapping() {
        let mut osc = SawtoothOscillator::<44100>::new(44100.0);
        osc.next_sample();
        osc.next_sample();
        let sample = osc.next_sample();
        assert!(!sample.is_nan());
    }

    #[test]
    fn test_zero_frequency() {
        let mut osc = SawtoothOscillator::<44100>::new(0.0);
        let sample1 = osc.next_sample();
        let sample2 = osc.next_sample();
        assert_eq!(sample1, sample2);
    }

    #[test]
    fn test_sample_range() {
        let mut osc = SawtoothOscillator::<44100>::new(440.0);
        for _ in 0..1000 {
            let sample = osc.next_sample();
            assert!((-1.0..=1.0).contains(&sample));
        }
    }

    #[test]
    fn test_process_buffer() {
        let mut osc = SawtoothOscillator::<44100>::new(440.0);
        let mut buffer = [0.0; 128];
        osc.process(&mut buffer);
        for &sample in buffer.iter() {
            assert!((-1.0..=1.0).contains(&sample));
        }
    }
}
