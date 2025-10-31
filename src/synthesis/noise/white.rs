//! White noise generator implementation.

use crate::{AudioSignal, Signal};
use rand::Rng;

/// A white noise generator.
///
/// White noise has equal power across all frequencies. Each sample is
/// a random value uniformly distributed between -1.0 and 1.0.
pub struct WhiteNoise<const SAMPLE_RATE: u32, R: Rng = rand::rngs::ThreadRng> {
    /// Random number generator
    rng: R,
}

impl<const SAMPLE_RATE: u32> Default for WhiteNoise<SAMPLE_RATE, rand::rngs::ThreadRng> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const SAMPLE_RATE: u32> WhiteNoise<SAMPLE_RATE, rand::rngs::ThreadRng> {
    /// Creates a new white noise generator with the default ThreadRng.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{Signal, WhiteNoise};
    ///
    /// let mut noise = WhiteNoise::<44100>::new();
    /// let sample = noise.next_sample();
    /// ```
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
        }
    }
}

impl<const SAMPLE_RATE: u32, R: Rng> WhiteNoise<SAMPLE_RATE, R> {
    /// Creates a new white noise generator with a custom RNG.
    ///
    /// # Arguments
    ///
    /// * `rng` - Random number generator to use
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{Signal, WhiteNoise};
    /// use rand::SeedableRng;
    ///
    /// let rng = rand::rngs::StdRng::seed_from_u64(42);
    /// let mut noise = WhiteNoise::<44100, _>::with_rng(rng);
    /// let sample = noise.next_sample();
    /// ```
    pub fn with_rng(rng: R) -> Self {
        Self { rng }
    }
}

impl<const SAMPLE_RATE: u32, R: Rng> Signal for WhiteNoise<SAMPLE_RATE, R> {
    fn next_sample(&mut self) -> f64 {
        // Generate random value in range [-1.0, 1.0]
        self.rng.gen_range(-1.0..=1.0)
    }
}

impl<const SAMPLE_RATE: u32, R: Rng> AudioSignal<SAMPLE_RATE> for WhiteNoise<SAMPLE_RATE, R> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let noise = WhiteNoise::<44100>::new();
        assert_eq!(noise.sample_rate(), 44100.0);
    }

    #[test]
    fn test_sample_range() {
        let mut noise = WhiteNoise::<44100>::new();
        // Generate many samples and verify all are in [-1.0, 1.0]
        for _ in 0..10000 {
            let sample = noise.next_sample();
            assert!((-1.0..=1.0).contains(&sample));
        }
    }

    #[test]
    fn test_randomness() {
        let mut noise = WhiteNoise::<44100>::new();
        // Generate samples and verify they're not all identical
        let samples: Vec<f64> = (0..100).map(|_| noise.next_sample()).collect();
        let first = samples[0];
        let all_same = samples.iter().all(|&s| s == first);
        assert!(!all_same, "White noise should produce varying samples");
    }

    #[test]
    fn test_process_buffer() {
        let mut noise = WhiteNoise::<44100>::new();
        let mut buffer = vec![0.0; 128];
        noise.process(&mut buffer);

        // Verify all samples are valid
        for sample in buffer {
            assert!((-1.0..=1.0).contains(&sample));
        }
    }
}
