//! Audio signal trait for sample-rate-aware signals.

use crate::Signal;

/// Common interface for anything that can be played as audio.
///
/// This trait extends `Signal` to add the sample rate at the type level, which is essential
/// for anything that generates audio samples. The sample rate is encoded as a const generic
/// parameter, ensuring that signals with different sample rates cannot be accidentally mixed.
///
/// # Type Parameters
///
/// * `SAMPLE_RATE` - Sample rate in Hz (e.g., 44100 for CD quality, 48000 for pro audio)
///
/// # Examples
///
/// ```
/// use earworm::{AudioSignal, SineOscillator};
///
/// // Sample rate is in the type
/// let osc: SineOscillator<44100> = SineOscillator::new(440.0);
/// assert_eq!(osc.sample_rate(), 44100.0);
/// ```
pub trait AudioSignal<const SAMPLE_RATE: u32>: Signal {
    /// Gets the sample rate at which this audio is being generated.
    ///
    /// # Returns
    ///
    /// Sample rate in Hz (e.g., 44100.0 for CD quality)
    fn sample_rate(&self) -> f64 {
        SAMPLE_RATE as f64
    }
}
