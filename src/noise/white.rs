//! White noise generator implementation.

use crate::{AudioSignal, Signal};
use rand::Rng;

/// A white noise generator.
///
/// White noise has equal power across all frequencies. Each sample is
/// a random value uniformly distributed between -1.0 and 1.0.
pub struct WhiteNoise<R: Rng = rand::rngs::ThreadRng> {
    /// Sample rate in Hz
    sample_rate: f64,
    /// Random number generator
    rng: R,
}

impl WhiteNoise<rand::rngs::ThreadRng> {
    /// Creates a new white noise generator with the default ThreadRng.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - Sample rate in Hz (e.g., 44100.0 for CD quality)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{Signal, WhiteNoise};
    ///
    /// let mut noise = WhiteNoise::new(44100.0);
    /// let sample = noise.next_sample();
    /// ```
    pub fn new(sample_rate: f64) -> Self {
        Self {
            sample_rate,
            rng: rand::thread_rng(),
        }
    }
}

impl<R: Rng> WhiteNoise<R> {
    /// Creates a new white noise generator with a custom RNG.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - Sample rate in Hz (e.g., 44100.0 for CD quality)
    /// * `rng` - Random number generator to use
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{Signal, WhiteNoise};
    /// use rand::SeedableRng;
    ///
    /// let rng = rand::rngs::StdRng::seed_from_u64(42);
    /// let mut noise = WhiteNoise::with_rng(44100.0, rng);
    /// let sample = noise.next_sample();
    /// ```
    pub fn with_rng(sample_rate: f64, rng: R) -> Self {
        Self { sample_rate, rng }
    }
}

impl<R: Rng> Signal for WhiteNoise<R> {
    fn next_sample(&mut self) -> f64 {
        // Generate random value in range [-1.0, 1.0]
        self.rng.gen_range(-1.0..=1.0)
    }
}

impl<R: Rng> AudioSignal for WhiteNoise<R> {
    fn sample_rate(&self) -> f64 {
        self.sample_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let noise = WhiteNoise::new(44100.0);
        assert_eq!(noise.sample_rate(), 44100.0);
    }

    #[test]
    fn test_sample_range() {
        let mut noise = WhiteNoise::new(44100.0);
        // Generate many samples and verify all are in [-1.0, 1.0]
        for _ in 0..10000 {
            let sample = noise.next_sample();
            assert!(sample >= -1.0 && sample <= 1.0);
        }
    }

    #[test]
    fn test_randomness() {
        let mut noise = WhiteNoise::new(44100.0);
        // Generate samples and verify they're not all identical
        let samples: Vec<f64> = (0..100).map(|_| noise.next_sample()).collect();
        let first = samples[0];
        let all_same = samples.iter().all(|&s| s == first);
        assert!(!all_same, "White noise should produce varying samples");
    }

    #[test]
    fn test_process_buffer() {
        let mut noise = WhiteNoise::new(44100.0);
        let mut buffer = vec![0.0; 128];
        noise.process(&mut buffer);

        // Verify all samples are valid
        for sample in buffer {
            assert!(sample >= -1.0 && sample <= 1.0);
        }
    }
}
