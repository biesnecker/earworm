//! Pink noise generator implementation.

use crate::{AudioSignal, Signal};
use rand::Rng;

/// A pink noise generator.
///
/// Pink noise (also called 1/f noise) has equal power per octave, meaning
/// it has more energy at lower frequencies than white noise. This
/// implementation uses the Voss-McCartney algorithm with 16 generators.
pub struct PinkNoise<const SAMPLE_RATE: u32, R: Rng = rand::rngs::ThreadRng> {
    /// Random number generator
    rng: R,
    /// Array of random values for the Voss algorithm
    generators: [f64; 16],
    /// Current sample counter (used to determine which generators to update)
    counter: u32,
}

impl<const SAMPLE_RATE: u32> Default for PinkNoise<SAMPLE_RATE, rand::rngs::ThreadRng> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const SAMPLE_RATE: u32> PinkNoise<SAMPLE_RATE, rand::rngs::ThreadRng> {
    /// Creates a new pink noise generator with the default ThreadRng.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{Signal, PinkNoise};
    ///
    /// let mut noise = PinkNoise::<44100>::new();
    /// let sample = noise.next_sample();
    /// ```
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let generators = [0.0; 16].map(|_| rng.gen_range(-1.0..=1.0));

        Self {
            rng,
            generators,
            counter: 0,
        }
    }
}

impl<const SAMPLE_RATE: u32, R: Rng> PinkNoise<SAMPLE_RATE, R> {
    /// Creates a new pink noise generator with a custom RNG.
    ///
    /// # Arguments
    ///
    /// * `rng` - Random number generator to use
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{Signal, PinkNoise};
    /// use rand::SeedableRng;
    ///
    /// let rng = rand::rngs::StdRng::seed_from_u64(42);
    /// let mut noise = PinkNoise::<44100, _>::with_rng(rng);
    /// let sample = noise.next_sample();
    /// ```
    pub fn with_rng(mut rng: R) -> Self {
        let generators = [0.0; 16].map(|_| rng.gen_range(-1.0..=1.0));

        Self {
            rng,
            generators,
            counter: 0,
        }
    }
}

impl<const SAMPLE_RATE: u32, R: Rng> Signal for PinkNoise<SAMPLE_RATE, R> {
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

impl<const SAMPLE_RATE: u32, R: Rng> AudioSignal<SAMPLE_RATE> for PinkNoise<SAMPLE_RATE, R> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let noise = PinkNoise::<44100>::new();
        assert_eq!(noise.sample_rate(), 44100.0);
    }

    #[test]
    fn test_sample_range() {
        let mut noise = PinkNoise::<44100>::new();
        // Generate many samples and verify all are in reasonable range
        for _ in 0..10000 {
            let sample = noise.next_sample();
            // Pink noise can occasionally go slightly outside [-1, 1] due to summing
            assert!((-1.5..=1.5).contains(&sample));
        }
    }

    #[test]
    fn test_randomness() {
        let mut noise = PinkNoise::<44100>::new();
        // Generate samples and verify they're not all identical
        let samples: Vec<f64> = (0..100).map(|_| noise.next_sample()).collect();
        let first = samples[0];
        let all_same = samples.iter().all(|&s| s == first);
        assert!(!all_same, "Pink noise should produce varying samples");
    }

    #[test]
    fn test_process_buffer() {
        let mut noise = PinkNoise::<44100>::new();
        let mut buffer = vec![0.0; 128];
        noise.process(&mut buffer);

        // Verify all samples are valid
        for sample in buffer {
            assert!((-1.5..=1.5).contains(&sample));
        }
    }

    #[test]
    fn test_counter_wrapping() {
        let mut noise = PinkNoise::<44100>::new();
        noise.counter = u32::MAX - 10;

        // Generate samples through the wraparound
        for _ in 0..20 {
            let sample = noise.next_sample();
            assert!((-1.5..=1.5).contains(&sample));
        }
    }
}
