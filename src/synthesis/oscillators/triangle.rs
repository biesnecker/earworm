//! Triangle wave oscillator implementation.

use super::Oscillator;
use crate::core::Pitched;
use crate::{AudioSignal, Signal};

/// A triangle wave oscillator for audio synthesis.
///
/// This oscillator generates a continuous triangle wave at a specified frequency.
/// The waveform rises linearly from -1.0 to 1.0, then falls linearly back to -1.0.
/// It maintains phase continuity across calls to `next_sample()`.
///
/// # Type Parameters
///
/// * `SAMPLE_RATE` - Sample rate in Hz (e.g., 44100 for CD quality)
pub struct TriangleOscillator<const SAMPLE_RATE: u32> {
    /// Current phase of the oscillator (0.0 to 1.0)
    phase: f64,
    /// Phase increment per sample (frequency / sample_rate)
    phase_increment: f64,
}

impl<const SAMPLE_RATE: u32> TriangleOscillator<SAMPLE_RATE> {
    /// Creates a new triangle oscillator.
    ///
    /// # Arguments
    ///
    /// * `frequency` - Frequency of the triangle wave in Hz
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{Signal, TriangleOscillator};
    ///
    /// // Create a 440 Hz (A4 note) oscillator at 44.1 kHz sample rate
    /// let mut osc = TriangleOscillator::<44100>::new(440.0);
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

impl<const SAMPLE_RATE: u32> Signal for TriangleOscillator<SAMPLE_RATE> {
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

impl<const SAMPLE_RATE: u32> AudioSignal<SAMPLE_RATE> for TriangleOscillator<SAMPLE_RATE> {}

impl<const SAMPLE_RATE: u32> Pitched for TriangleOscillator<SAMPLE_RATE> {
    fn set_frequency(&mut self, frequency: f64) {
        self.phase_increment = frequency / SAMPLE_RATE as f64;
    }

    fn frequency(&self) -> f64 {
        self.phase_increment * SAMPLE_RATE as f64
    }
}

impl<const SAMPLE_RATE: u32> Oscillator for TriangleOscillator<SAMPLE_RATE> {
    fn reset(&mut self) {
        self.phase = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oscillator_creation() {
        let osc = TriangleOscillator::<44100>::new(440.0);
        assert!((osc.frequency() - 440.0).abs() < 0.01);
    }

    #[test]
    fn test_sample_generation() {
        let mut osc = TriangleOscillator::<44100>::new(440.0);
        let sample = osc.next_sample();
        // Triangle wave should produce values in range [-1.0, 1.0]
        assert!((-1.0..=1.0).contains(&sample));
    }

    #[test]
    fn test_frequency_change() {
        let mut osc = TriangleOscillator::<44100>::new(440.0);
        osc.set_frequency(880.0);
        assert!((osc.frequency() - 880.0).abs() < 0.01);
    }

    #[test]
    fn test_reset() {
        let mut osc = TriangleOscillator::<44100>::new(440.0);
        // Generate some samples
        for _ in 0..100 {
            osc.next_sample();
        }
        // Reset should bring phase back to 0
        osc.reset();
        // After reset, first sample should be at beginning of waveform
        let sample = osc.next_sample();
        assert!((sample - (-1.0)).abs() < 0.01); // Should start at -1.0
    }

    #[test]
    fn test_waveform_shape() {
        let mut osc = TriangleOscillator::<44100>::new(1.0); // 1 Hz at 44100 Hz sample rate

        // First quarter should rise from -1 to near 0
        osc.reset();
        let first = osc.next_sample();
        for _ in 0..(44100 / 4 - 1) {
            osc.next_sample();
        }
        let quarter = osc.next_sample();
        assert!(first < quarter); // Should be rising

        // Second quarter should continue rising to near 1
        for _ in 0..(44100 / 4) {
            osc.next_sample();
        }
        let half = osc.next_sample();
        assert!(quarter < half); // Should continue rising
        assert!(half > 0.9); // Should be near peak

        // Third quarter should fall back toward 0
        for _ in 0..(44100 / 4) {
            osc.next_sample();
        }
        let three_quarter = osc.next_sample();
        assert!(three_quarter < half); // Should be falling
    }

    #[test]
    fn test_phase_wrapping() {
        let mut osc = TriangleOscillator::<44100>::new(44100.0); // Frequency = sample rate
        // At this frequency, phase should wrap every sample
        osc.next_sample();
        osc.next_sample();
        // Should not panic or produce NaN
        let sample = osc.next_sample();
        assert!(!sample.is_nan());
    }

    #[test]
    fn test_zero_frequency() {
        let mut osc = TriangleOscillator::<44100>::new(0.0);
        let sample1 = osc.next_sample();
        let sample2 = osc.next_sample();
        // With zero frequency, phase doesn't advance, so samples should be identical
        assert_eq!(sample1, sample2);
    }

    #[test]
    fn test_linearity() {
        let mut osc = TriangleOscillator::<44100>::new(1.0); // 1 Hz

        osc.reset();
        let samples: Vec<f64> = (0..100).map(|_| osc.next_sample()).collect();

        // Check that the rising portion is linear
        for i in 1..50 {
            if samples[i - 1] < samples[i] {
                // Rising portion
                let diff1 = samples[i] - samples[i - 1];
                if i < 49 && samples[i] < samples[i + 1] {
                    let diff2 = samples[i + 1] - samples[i];
                    // Differences should be roughly equal (linear)
                    assert!((diff1 - diff2).abs() < 0.01);
                }
            }
        }
    }

    #[test]
    fn test_sample_range() {
        let mut osc = TriangleOscillator::<44100>::new(440.0);
        for _ in 0..1000 {
            let sample = osc.next_sample();
            assert!(
                (-1.0..=1.0).contains(&sample),
                "Sample out of range: {}",
                sample
            );
        }
    }

    #[test]
    fn test_process_buffer() {
        let mut osc = TriangleOscillator::<44100>::new(440.0);
        let mut buffer = [0.0; 128];
        osc.process(&mut buffer);

        // Check that all samples are in valid range
        for &sample in buffer.iter() {
            assert!((-1.0..=1.0).contains(&sample));
        }
    }
}
