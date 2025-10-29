//! Core trait definitions for oscillators.

use crate::Signal;

/// Common interface for all oscillators.
///
/// This trait extends `Signal` to add oscillator-specific functionality:
/// frequency control and state management. All oscillators are signals,
/// but provide additional capabilities for controlling their frequency.
pub trait Oscillator: Signal {
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
