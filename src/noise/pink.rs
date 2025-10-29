//! Pink noise generator implementation.

use crate::{AudioSignal, Signal};
use rand::Rng;

/// A pink noise generator.
///
/// Pink noise (also called 1/f noise) has equal power per octave, meaning
/// it has more energy at lower frequencies than white noise. This
/// implementation uses the Voss-McCartney algorithm with 16 generators.
pub struct PinkNoise<R: Rng = rand::rngs::ThreadRng> {
    /// Sample rate in Hz
    sample_rate: f64,
    /// Random number generator
    rng: R,
    /// Array of random values for the Voss algorithm
    generators: [f64; 16],
    /// Current sample counter (used to determine which generators to update)
    counter: u32,
}

impl PinkNoise<rand::rngs::ThreadRng> {
    /// Creates a new pink noise generator with the default ThreadRng.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - Sample rate in Hz (e.g., 44100.0 for CD quality)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{Signal, PinkNoise};
    ///
    /// let mut noise = PinkNoise::new(44100.0);
    /// let sample = noise.next_sample();
    /// ```
    pub fn new(sample_rate: f64) -> Self {
        let mut rng = rand::thread_rng();
        let generators = [0.0; 16].map(|_| rng.gen_range(-1.0..=1.0));

        Self {
            sample_rate,
            rng,
            generators,
            counter: 0,
        }
    }
}

impl<R: Rng> PinkNoise<R> {
    /// Creates a new pink noise generator with a custom RNG.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - Sample rate in Hz (e.g., 44100.0 for CD quality)
    /// * `rng` - Random number generator to use
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{Signal, PinkNoise};
    /// use rand::SeedableRng;
    ///
    /// let rng = rand::rngs::StdRng::seed_from_u64(42);
    /// let mut noise = PinkNoise::with_rng(44100.0, rng);
    /// let sample = noise.next_sample();
    /// ```
    pub fn with_rng(sample_rate: f64, mut rng: R) -> Self {
        let generators = [0.0; 16].map(|_| rng.gen_range(-1.0..=1.0));

        Self {
            sample_rate,
            rng,
            generators,
            counter: 0,
        }
    }
}

impl<R: Rng> Signal for PinkNoise<R> {
    fn next_sample(&mut self) -> f64 {
        // Voss-McCartney algorithm: update generators based on counter's trailing zeros
        let mut bit = 1;
        for i in 0..16 {
            if self.counter & bit != 0 {
                break;
            }
            self.generators[i] = self.rng.gen_range(-1.0..=1.0);
            bit <<= 1;
        }

        self.counter = self.counter.wrapping_add(1);

        // Sum all generators and normalize
        let sum: f64 = self.generators.iter().sum();
        // Divide by number of generators and scale to approximate [-1.0, 1.0] range
        sum / 16.0
    }
}

impl<R: Rng> AudioSignal for PinkNoise<R> {
    fn sample_rate(&self) -> f64 {
        self.sample_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let noise = PinkNoise::new(44100.0);
        assert_eq!(noise.sample_rate(), 44100.0);
    }

    #[test]
    fn test_sample_range() {
        let mut noise = PinkNoise::new(44100.0);
        // Generate many samples and verify all are in reasonable range
        for _ in 0..10000 {
            let sample = noise.next_sample();
            // Pink noise can occasionally go slightly outside [-1, 1] due to summing
            assert!(sample >= -1.5 && sample <= 1.5);
        }
    }

    #[test]
    fn test_randomness() {
        let mut noise = PinkNoise::new(44100.0);
        // Generate samples and verify they're not all identical
        let samples: Vec<f64> = (0..100).map(|_| noise.next_sample()).collect();
        let first = samples[0];
        let all_same = samples.iter().all(|&s| s == first);
        assert!(!all_same, "Pink noise should produce varying samples");
    }

    #[test]
    fn test_process_buffer() {
        let mut noise = PinkNoise::new(44100.0);
        let mut buffer = vec![0.0; 128];
        noise.process(&mut buffer);

        // Verify all samples are valid
        for sample in buffer {
            assert!(sample >= -1.5 && sample <= 1.5);
        }
    }

    #[test]
    fn test_counter_wrapping() {
        let mut noise = PinkNoise::new(44100.0);
        noise.counter = u32::MAX - 10;

        // Generate samples through the wraparound
        for _ in 0..20 {
            let sample = noise.next_sample();
            assert!(sample >= -1.5 && sample <= 1.5);
        }
    }
}
