//! Core trait definitions for oscillators.

use crate::core::Pitched;

/// Oscillators are pitched signals with additional state control.
///
/// This trait extends `Pitched` to add oscillator-specific functionality
/// like state reset. All oscillators have controllable frequency (via `Pitched`)
/// and can reset their internal state to initial conditions.
pub trait Oscillator: Pitched {
    /// Resets the oscillator to its initial state.
    ///
    /// This typically resets the phase to zero and any other internal
    /// state variables to their initial values.
    fn reset(&mut self);
}
