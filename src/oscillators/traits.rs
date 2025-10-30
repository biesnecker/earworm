//! Core trait definitions for oscillators.

use crate::AudioSignal;

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
