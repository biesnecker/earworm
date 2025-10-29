//! Core signal processing trait.
//!
//! This module provides the fundamental `Signal` trait that represents
//! any audio signal source or processor that can generate samples.

/// Common interface for all signal sources and processors.
///
/// This trait defines the core functionality for anything that can generate
/// audio samples: oscillators, filters, LFOs, envelopes, noise generators, etc.
///
/// The trait provides two fundamental operations:
/// - Single sample generation via `next_sample()`
/// - Batch processing via `process()`
pub trait Signal {
    /// Generates the next sample from the signal.
    ///
    /// # Returns
    ///
    /// A sample value, typically between -1.0 and 1.0 for audio signals
    fn next_sample(&mut self) -> f64;

    /// Generates multiple samples into a buffer.
    ///
    /// Default implementation calls `next_sample()` for each element.
    /// Implementors may override this for more efficient batch processing.
    ///
    /// # Arguments
    ///
    /// * `buffer` - Mutable slice to fill with samples
    fn process(&mut self, buffer: &mut [f64]) {
        for sample in buffer.iter_mut() {
            *sample = self.next_sample();
        }
    }
}

/// Implementation of `Signal` for `f64` representing a constant signal value.
///
/// This allows using constant values anywhere a `Signal` is expected,
/// which is useful for DC offsets, fixed gain values, or testing.
///
/// # Examples
///
/// ```
/// use earworm::Signal;
///
/// let mut constant = 0.5_f64;
/// assert_eq!(constant.next_sample(), 0.5);
/// assert_eq!(constant.next_sample(), 0.5);
///
/// let mut buffer = vec![0.0; 4];
/// constant.process(&mut buffer);
/// assert_eq!(buffer, vec![0.5, 0.5, 0.5, 0.5]);
/// ```
impl Signal for f64 {
    fn next_sample(&mut self) -> f64 {
        *self
    }

    fn process(&mut self, buffer: &mut [f64]) {
        // Optimized implementation for constant values
        buffer.fill(*self);
    }
}
