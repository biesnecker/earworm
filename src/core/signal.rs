//! Core signal processing trait and parameter types.
//!
//! This module provides the fundamental `Signal` trait that represents
//! any audio signal source or processor that can generate samples, as well
//! as the `Param` type for parameters that can be either fixed or modulated.

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

/// Minimal trait for anything with a controllable pitch.
///
/// This trait provides basic frequency control for any signal that has a
/// tunable pitch or frequency. It can be implemented by oscillators, filters,
/// effects, or other signal processors that have a frequency parameter.
///
/// # Examples
///
/// ```
/// use earworm::{Pitched, SineOscillator};
///
/// // Oscillators implement Pitched, so you need to import the trait
/// // to use its methods
/// let mut osc = SineOscillator::<44100>::new(440.0);
/// assert_eq!(osc.frequency(), 440.0);
///
/// osc.set_frequency(880.0);
/// assert_eq!(osc.frequency(), 880.0);
/// ```
pub trait Pitched {
    /// Sets the frequency of the signal.
    ///
    /// # Arguments
    ///
    /// * `freq` - New frequency in Hz
    fn set_frequency(&mut self, freq: f64);

    /// Gets the current frequency of the signal.
    ///
    /// # Returns
    ///
    /// Current frequency in Hz
    fn frequency(&self) -> f64;
}

/// A constant signal that always returns the same value.
///
/// This is a lightweight wrapper around `f64` that implements `Signal`,
/// useful for creating fixed parameters that can be converted into `Param`.
///
/// # Examples
///
/// ```
/// use earworm::{ConstantSignal, Param};
///
/// let constant = ConstantSignal::<44100>(0.5);
/// let param: Param = constant.into();
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConstantSignal<const SAMPLE_RATE: u32>(pub f64);

impl<const SAMPLE_RATE: u32> Signal for ConstantSignal<SAMPLE_RATE> {
    fn next_sample(&mut self) -> f64 {
        self.0
    }

    fn process(&mut self, buffer: &mut [f64]) {
        buffer.fill(self.0);
    }
}

impl<const SAMPLE_RATE: u32> From<f64> for ConstantSignal<SAMPLE_RATE> {
    fn from(value: f64) -> Self {
        ConstantSignal::<SAMPLE_RATE>(value)
    }
}

impl<const SAMPLE_RATE: u32> crate::AudioSignal<SAMPLE_RATE> for ConstantSignal<SAMPLE_RATE> {}

/// A parameter that can be either a fixed value or modulated by a signal.
///
/// This type is used throughout the library for parameters that can be
/// controlled either statically (with a fixed value) or dynamically (by
/// another signal source like an LFO or envelope).
///
/// Using `Param` instead of generics simplifies type signatures and allows
/// for heterogeneous collections of modulatable parameters, at the cost of
/// a small performance overhead from dynamic dispatch.
///
/// # Examples
///
/// ```
/// use earworm::{Param, SineOscillator};
///
/// // Fixed parameter - f64 converts to ConstantSignal, then to Param
/// let mut fixed_param: Param = 0.5.into();
/// assert_eq!(fixed_param.value(), 0.5);
///
/// // Modulated parameter using Into
/// let lfo = SineOscillator::<44100>::new(2.0);
/// let mut modulated_param: Param = lfo.into();
/// let value = modulated_param.value(); // Gets next sample from LFO
///
/// // Or explicitly with Param::modulated()
/// let lfo2 = SineOscillator::<44100>::new(2.0);
/// let mut modulated_param2 = Param::modulated(lfo2);
/// ```
pub enum Param {
    /// A fixed, constant value
    Fixed(f64),
    /// A value modulated by a signal source
    Signal(Box<dyn Signal + Send>),
}

impl Param {
    /// Gets the current value of the parameter.
    ///
    /// For fixed parameters, this returns the constant value.
    /// For modulated parameters, this advances the signal and returns the next sample.
    ///
    /// # Returns
    ///
    /// The current parameter value
    pub fn value(&mut self) -> f64 {
        match self {
            Param::Fixed(v) => *v,
            Param::Signal(s) => s.next_sample(),
        }
    }

    /// Creates a fixed parameter with the given value.
    ///
    /// # Arguments
    ///
    /// * `value` - The constant value for this parameter
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::Param;
    ///
    /// let param = Param::fixed(0.75);
    /// ```
    pub fn fixed(value: f64) -> Self {
        Param::Fixed(value)
    }

    /// Creates a modulated parameter controlled by a signal source.
    ///
    /// # Arguments
    ///
    /// * `signal` - Any type implementing `Signal + Send` to control this parameter
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{Param, SineOscillator};
    ///
    /// let lfo = SineOscillator::<44100>::new(1.0);
    /// let param = Param::modulated(lfo);
    /// ```
    pub fn modulated(signal: impl Signal + Send + 'static) -> Self {
        Param::Signal(Box::new(signal))
    }

    /// Returns true if this parameter is fixed (non-modulated).
    pub fn is_fixed(&self) -> bool {
        matches!(self, Param::Fixed(_))
    }
}

impl From<f64> for Param {
    fn from(value: f64) -> Self {
        Param::Fixed(value)
    }
}

impl<S: Signal + Send + 'static> From<S> for Param {
    fn from(signal: S) -> Self {
        Param::Signal(Box::new(signal))
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f64_to_constant_signal() {
        let constant: ConstantSignal<44100> = 0.5.into();
        assert_eq!(constant.0, 0.5);
    }

    #[test]
    fn test_f64_to_param() {
        let param: Param = 0.5.into();
        match param {
            Param::Fixed(v) => assert_eq!(v, 0.5),
            Param::Signal(_) => panic!("Expected Fixed, got Signal"),
        }
    }

    #[test]
    fn test_signal_to_param() {
        use crate::SineOscillator;
        let lfo = SineOscillator::<44100>::new(1.0);
        let param: Param = lfo.into();
        match param {
            Param::Fixed(_) => panic!("Expected Signal, got Fixed"),
            Param::Signal(_) => {} // Success
        }
    }

    #[test]
    fn test_constant_signal_to_param() {
        // ConstantSignal converts to Param::Signal (can't specialize without nightly)
        // For efficient Param::Fixed, use f64.into() directly
        let constant = ConstantSignal::<44100>(0.75);
        let param: Param = constant.into();
        match param {
            Param::Signal(_) => {} // Success - ConstantSignal is a Signal
            Param::Fixed(_) => panic!("Unexpected Fixed variant"),
        }
    }
}
