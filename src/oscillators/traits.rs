//! Core trait definitions for oscillators.

use crate::Signal;

/// Common interface for anything that can be played as audio.
///
/// This trait extends `Signal` to add the sample rate, which is essential
/// for anything that generates audio samples. The sample rate is read-only
/// as it's typically set during construction and shouldn't change during playback.
pub trait AudioSignal: Signal {
    /// Gets the sample rate at which this audio is being generated.
    ///
    /// # Returns
    ///
    /// Sample rate in Hz (e.g., 44100.0 for CD quality)
    fn sample_rate(&self) -> f64;
}

/// Common interface for all oscillators.
///
/// This trait extends `AudioSignal` to add oscillator-specific functionality:
/// frequency control and state management. All oscillators are audio signals,
/// but provide additional capabilities for controlling their frequency.
pub trait Oscillator: AudioSignal {
    /// Sets the frequency of the oscillator.
    ///
    /// # Arguments
    ///
    /// * `frequency` - New frequency in Hz
    fn set_frequency(&mut self, frequency: f64);

    /// Gets the current frequency of the oscillator.
    ///
    /// # Returns
    ///
    /// Current frequency in Hz
    fn frequency(&self) -> f64;

    /// Resets the oscillator to its initial state.
    fn reset(&mut self);
}
