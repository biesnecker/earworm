//! Signal combinators for composing and transforming audio signals.
//!
//! This module provides building blocks for combining and manipulating signals,
//! including mathematical operations (addition, multiplication), gain control,
//! offsetting, and mixing multiple signals together.

use crate::{Param, Signal};

/// Multiplies two signals together (amplitude modulation / ring modulation).
///
/// This combinator performs sample-by-sample multiplication of two signals,
/// which creates amplitude modulation effects. When one signal is an LFO,
/// this creates tremolo. When both signals are in the audio range, this
/// creates ring modulation with complex harmonic content.
///
/// # Examples
///
/// ```
/// use earworm::{SineOscillator, combinators::Multiply};
///
/// let carrier = SineOscillator::new(440.0, 44100.0);
/// let modulator = SineOscillator::new(2.0, 44100.0);
/// let mut ring_mod = Multiply { a: carrier, b: modulator };
/// ```
pub struct Multiply<A: Signal, B: Signal> {
    pub a: A,
    pub b: B,
}

impl<A: Signal, B: Signal> Signal for Multiply<A, B> {
    fn next_sample(&mut self) -> f64 {
        self.a.next_sample() * self.b.next_sample()
    }
}

/// Adds two signals together (mixing).
///
/// This combinator performs sample-by-sample addition of two signals.
/// Note that when mixing multiple signals, you may need to reduce the
/// gain to prevent clipping.
///
/// # Examples
///
/// ```
/// use earworm::{SineOscillator, combinators::Add};
///
/// let osc1 = SineOscillator::new(440.0, 44100.0);
/// let osc2 = SineOscillator::new(880.0, 44100.0);
/// let mut mixed = Add { a: osc1, b: osc2 };
/// ```
pub struct Add<A: Signal, B: Signal> {
    pub a: A,
    pub b: B,
}

impl<A: Signal, B: Signal> Signal for Add<A, B> {
    fn next_sample(&mut self) -> f64 {
        self.a.next_sample() + self.b.next_sample()
    }
}

/// Scales a signal by a factor (gain/attenuation).
///
/// This combinator multiplies the input signal by a gain factor,
/// which can be either fixed or modulated. Values greater than 1.0
/// amplify the signal, while values between 0.0 and 1.0 attenuate it.
///
/// # Examples
///
/// ```
/// use earworm::{SineOscillator, combinators::Gain};
///
/// let osc = SineOscillator::new(440.0, 44100.0);
/// let mut quieter = Gain { source: osc, gain: 0.5.into() };
/// ```
pub struct Gain<S: Signal> {
    pub source: S,
    pub gain: Param,
}

impl<S: Signal> Signal for Gain<S> {
    fn next_sample(&mut self) -> f64 {
        self.source.next_sample() * self.gain.value()
    }
}

/// Adds an offset to a signal (DC offset).
///
/// This combinator adds a constant or modulated offset to the input signal.
/// This is useful for shifting signals into different ranges or adding
/// vibrato/pitch modulation when used with oscillator frequency parameters.
///
/// # Examples
///
/// ```
/// use earworm::{SineOscillator, combinators::Offset};
///
/// let osc = SineOscillator::new(440.0, 44100.0);
/// // Shift the signal from [-1, 1] to [0, 2]
/// let mut shifted = Offset { source: osc, offset: 1.0.into() };
/// ```
pub struct Offset<S: Signal> {
    pub source: S,
    pub offset: Param,
}

impl<S: Signal> Signal for Offset<S> {
    fn next_sample(&mut self) -> f64 {
        self.source.next_sample() + self.offset.value()
    }
}

/// Mixes multiple signals together with optional weights.
///
/// This combinator allows mixing any number of signals together,
/// with each signal having its own weight (gain) factor. This is
/// more efficient than chaining multiple `Add` combinators when
/// you need to mix many signals.
///
/// # Examples
///
/// ```
/// use earworm::{SineOscillator, combinators::Mix};
///
/// let osc1 = SineOscillator::new(440.0, 44100.0);
/// let osc2 = SineOscillator::new(880.0, 44100.0);
/// let mut mixer = Mix {
///     sources: vec![
///         (Box::new(osc1), 0.5),
///         (Box::new(osc2), 0.5),
///     ],
/// };
/// ```
pub struct Mix {
    pub sources: Vec<(Box<dyn Signal + Send>, f64)>,
}

impl Signal for Mix {
    fn next_sample(&mut self) -> f64 {
        self.sources
            .iter_mut()
            .map(|(source, weight)| source.next_sample() * *weight)
            .sum()
    }
}

/// Clips/clamps a signal to a range (hard clipping distortion).
///
/// This combinator limits the signal amplitude to stay within a specified range,
/// creating hard clipping distortion when the signal exceeds the bounds. This is
/// useful for overdrive effects and preventing signal overflow.
///
/// # Examples
///
/// ```
/// use earworm::{SineOscillator, combinators::Clamp};
///
/// let osc = SineOscillator::new(440.0, 44100.0);
/// let mut clipped = Clamp { source: osc, min: -0.5, max: 0.5 };
/// ```
pub struct Clamp<S: Signal> {
    pub source: S,
    pub min: f64,
    pub max: f64,
}

impl<S: Signal> Signal for Clamp<S> {
    fn next_sample(&mut self) -> f64 {
        self.source.next_sample().clamp(self.min, self.max)
    }
}

/// Applies a function to each sample.
///
/// This combinator allows applying arbitrary transformations to a signal
/// by providing a function that processes each sample. This is useful for
/// custom waveshaping, distortion, or other sample-by-sample processing.
///
/// # Examples
///
/// ```
/// use earworm::{SineOscillator, combinators::Map};
///
/// let osc = SineOscillator::new(440.0, 44100.0);
/// // Apply a simple waveshaping function
/// let mut shaped = Map { source: osc, func: |x| x * x * x };
/// ```
pub struct Map<S: Signal, F>
where
    F: FnMut(f64) -> f64,
{
    pub source: S,
    pub func: F,
}

impl<S: Signal, F> Signal for Map<S, F>
where
    F: FnMut(f64) -> f64,
{
    fn next_sample(&mut self) -> f64 {
        let sample = self.source.next_sample();
        (self.func)(sample)
    }
}

/// Inverts/negates a signal.
///
/// This combinator multiplies the signal by -1, flipping it around the zero axis.
/// This can be used for phase inversion or creating complementary signals.
///
/// # Examples
///
/// ```
/// use earworm::{SineOscillator, combinators::Invert};
///
/// let osc = SineOscillator::new(440.0, 44100.0);
/// let mut inverted = Invert { source: osc };
/// ```
pub struct Invert<S: Signal> {
    pub source: S,
}

impl<S: Signal> Signal for Invert<S> {
    fn next_sample(&mut self) -> f64 {
        -self.source.next_sample()
    }
}

/// Crossfades between two signals (0.0 = all A, 1.0 = all B).
///
/// This combinator performs a linear crossfade between two signals based on
/// a mix parameter. When mix is 0.0, only signal A is heard. When mix is 1.0,
/// only signal B is heard. Values in between blend the two signals proportionally.
///
/// # Examples
///
/// ```
/// use earworm::{SineOscillator, combinators::Crossfade};
///
/// let osc1 = SineOscillator::new(440.0, 44100.0);
/// let osc2 = SineOscillator::new(880.0, 44100.0);
/// let mut crossfade = Crossfade { a: osc1, b: osc2, mix: 0.5.into() };
/// ```
pub struct Crossfade<A: Signal, B: Signal> {
    pub a: A,
    pub b: B,
    pub mix: Param,
}

impl<A: Signal, B: Signal> Signal for Crossfade<A, B> {
    fn next_sample(&mut self) -> f64 {
        let mix = self.mix.value().clamp(0.0, 1.0);
        let sample_a = self.a.next_sample();
        let sample_b = self.b.next_sample();
        sample_a * (1.0 - mix) + sample_b * mix
    }
}

/// Takes the minimum of two signals.
///
/// This combinator outputs the minimum value of two signals at each sample.
/// This can create interesting modulation effects and is useful for
/// creating hard sync-like behaviors.
///
/// # Examples
///
/// ```
/// use earworm::{SineOscillator, combinators::Min};
///
/// let osc1 = SineOscillator::new(440.0, 44100.0);
/// let osc2 = SineOscillator::new(880.0, 44100.0);
/// let mut min_signal = Min { a: osc1, b: osc2 };
/// ```
pub struct Min<A: Signal, B: Signal> {
    pub a: A,
    pub b: B,
}

impl<A: Signal, B: Signal> Signal for Min<A, B> {
    fn next_sample(&mut self) -> f64 {
        self.a.next_sample().min(self.b.next_sample())
    }
}

/// Takes the maximum of two signals.
///
/// This combinator outputs the maximum value of two signals at each sample.
/// This can create interesting modulation effects and is useful for
/// various waveshaping techniques.
///
/// # Examples
///
/// ```
/// use earworm::{SineOscillator, combinators::Max};
///
/// let osc1 = SineOscillator::new(440.0, 44100.0);
/// let osc2 = SineOscillator::new(880.0, 44100.0);
/// let mut max_signal = Max { a: osc1, b: osc2 };
/// ```
pub struct Max<A: Signal, B: Signal> {
    pub a: A,
    pub b: B,
}

impl<A: Signal, B: Signal> Signal for Max<A, B> {
    fn next_sample(&mut self) -> f64 {
        self.a.next_sample().max(self.b.next_sample())
    }
}

/// Absolute value (rectification).
///
/// This combinator takes the absolute value of the signal, effectively
/// folding negative values to positive. This creates full-wave rectification,
/// which adds harmonic content to the signal.
///
/// # Examples
///
/// ```
/// use earworm::{SineOscillator, combinators::Abs};
///
/// let osc = SineOscillator::new(440.0, 44100.0);
/// let mut rectified = Abs { source: osc };
/// ```
pub struct Abs<S: Signal> {
    pub source: S,
}

impl<S: Signal> Signal for Abs<S> {
    fn next_sample(&mut self) -> f64 {
        self.source.next_sample().abs()
    }
}

/// Only passes signal through if it exceeds a threshold (noise gate).
///
/// This combinator implements a noise gate that silences the signal when
/// its amplitude is below a threshold. This is useful for removing noise
/// or creating gated effects.
///
/// # Examples
///
/// ```
/// use earworm::{SineOscillator, combinators::Gate};
///
/// let osc = SineOscillator::new(440.0, 44100.0);
/// let mut gated = Gate { source: osc, threshold: 0.1.into() };
/// ```
pub struct Gate<S: Signal> {
    pub source: S,
    pub threshold: Param,
}

impl<S: Signal> Signal for Gate<S> {
    fn next_sample(&mut self) -> f64 {
        let sample = self.source.next_sample();
        if sample.abs() > self.threshold.value() {
            sample
        } else {
            0.0
        }
    }
}

/// Extension trait providing convenient combinator methods on any Signal.
///
/// This trait is automatically implemented for all types that implement `Signal`,
/// providing a fluent API for chaining signal operations together.
///
/// # Examples
///
/// ```
/// use earworm::{SineOscillator, combinators::SignalExt};
///
/// let osc1 = SineOscillator::new(440.0, 44100.0);
/// let osc2 = SineOscillator::new(2.0, 44100.0);
///
/// // Chain operations together
/// let mut signal = osc1
///     .multiply(osc2)  // Ring modulation
///     .gain(0.5)       // Reduce volume
///     .clamp(-0.8, 0.8) // Clip the signal
///     .offset(0.1);    // Add DC offset
/// ```
pub trait SignalExt: Signal + Sized {
    /// Multiplies this signal with another signal (ring modulation).
    fn multiply<S: Signal>(self, other: S) -> Multiply<Self, S> {
        Multiply { a: self, b: other }
    }

    /// Adds this signal to another signal (mixing).
    fn add<S: Signal>(self, other: S) -> Add<Self, S> {
        Add { a: self, b: other }
    }

    /// Applies a gain factor to this signal.
    fn gain(self, gain: impl Into<Param>) -> Gain<Self> {
        Gain {
            source: self,
            gain: gain.into(),
        }
    }

    /// Adds an offset to this signal.
    fn offset(self, offset: impl Into<Param>) -> Offset<Self> {
        Offset {
            source: self,
            offset: offset.into(),
        }
    }

    /// Clips/clamps this signal to a range.
    fn clamp(self, min: f64, max: f64) -> Clamp<Self> {
        Clamp {
            source: self,
            min,
            max,
        }
    }

    /// Applies a function to each sample of this signal.
    fn map<F>(self, func: F) -> Map<Self, F>
    where
        F: FnMut(f64) -> f64,
    {
        Map { source: self, func }
    }

    /// Inverts/negates this signal.
    fn invert(self) -> Invert<Self> {
        Invert { source: self }
    }

    /// Crossfades this signal with another signal.
    fn crossfade<S: Signal>(self, other: S, mix: impl Into<Param>) -> Crossfade<Self, S> {
        Crossfade {
            a: self,
            b: other,
            mix: mix.into(),
        }
    }

    /// Takes the minimum of this signal and another signal.
    fn min<S: Signal>(self, other: S) -> Min<Self, S> {
        Min { a: self, b: other }
    }

    /// Takes the maximum of this signal and another signal.
    fn max<S: Signal>(self, other: S) -> Max<Self, S> {
        Max { a: self, b: other }
    }

    /// Takes the absolute value of this signal.
    fn abs(self) -> Abs<Self> {
        Abs { source: self }
    }

    /// Applies a noise gate to this signal.
    fn gate(self, threshold: impl Into<Param>) -> Gate<Self> {
        Gate {
            source: self,
            threshold: threshold.into(),
        }
    }
}

// Blanket implementation for all Signal types
impl<T: Signal> SignalExt for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConstantSignal;

    #[test]
    fn test_multiply() {
        let a = ConstantSignal(2.0);
        let b = ConstantSignal(3.0);
        let mut mult = Multiply { a, b };
        assert_eq!(mult.next_sample(), 6.0);
    }

    #[test]
    fn test_add() {
        let a = ConstantSignal(2.0);
        let b = ConstantSignal(3.0);
        let mut add = Add { a, b };
        assert_eq!(add.next_sample(), 5.0);
    }

    #[test]
    fn test_gain() {
        let source = ConstantSignal(2.0);
        let mut gain = Gain {
            source,
            gain: 0.5.into(),
        };
        assert_eq!(gain.next_sample(), 1.0);
    }

    #[test]
    fn test_offset() {
        let source = ConstantSignal(2.0);
        let mut offset = Offset {
            source,
            offset: 3.0.into(),
        };
        assert_eq!(offset.next_sample(), 5.0);
    }

    #[test]
    fn test_mix() {
        let osc1 = ConstantSignal(1.0);
        let osc2 = ConstantSignal(2.0);
        let osc3 = ConstantSignal(3.0);

        let mut mixer = Mix {
            sources: vec![
                (Box::new(osc1), 1.0),
                (Box::new(osc2), 0.5),
                (Box::new(osc3), 0.25),
            ],
        };

        // 1.0*1.0 + 2.0*0.5 + 3.0*0.25 = 1.0 + 1.0 + 0.75 = 2.75
        assert_eq!(mixer.next_sample(), 2.75);
    }

    #[test]
    fn test_signal_ext_chaining() {
        let a = ConstantSignal(2.0);
        let b = ConstantSignal(3.0);

        let mut signal = a.multiply(b).gain(0.5).offset(1.0);

        // (2.0 * 3.0) * 0.5 + 1.0 = 6.0 * 0.5 + 1.0 = 3.0 + 1.0 = 4.0
        assert_eq!(signal.next_sample(), 4.0);
    }

    #[test]
    fn test_signal_ext_add() {
        let a = ConstantSignal(2.0);
        let b = ConstantSignal(3.0);

        let mut signal = a.add(b);
        assert_eq!(signal.next_sample(), 5.0);
    }

    #[test]
    fn test_clamp() {
        let source = ConstantSignal(2.0);
        let mut clamped = Clamp {
            source,
            min: -1.0,
            max: 1.0,
        };
        assert_eq!(clamped.next_sample(), 1.0);

        let source2 = ConstantSignal(-2.0);
        let mut clamped2 = Clamp {
            source: source2,
            min: -1.0,
            max: 1.0,
        };
        assert_eq!(clamped2.next_sample(), -1.0);
    }

    #[test]
    fn test_map() {
        let source = ConstantSignal(2.0);
        let mut mapped = Map {
            source,
            func: |x| x * 2.0,
        };
        assert_eq!(mapped.next_sample(), 4.0);
    }

    #[test]
    fn test_invert() {
        let source = ConstantSignal(2.0);
        let mut inverted = Invert { source };
        assert_eq!(inverted.next_sample(), -2.0);
    }

    #[test]
    fn test_crossfade() {
        let a = ConstantSignal(1.0);
        let b = ConstantSignal(3.0);
        let mut crossfade = Crossfade {
            a,
            b,
            mix: 0.5.into(),
        };
        // 1.0 * 0.5 + 3.0 * 0.5 = 2.0
        assert_eq!(crossfade.next_sample(), 2.0);

        let a2 = ConstantSignal(1.0);
        let b2 = ConstantSignal(3.0);
        let mut crossfade2 = Crossfade {
            a: a2,
            b: b2,
            mix: 0.0.into(),
        };
        assert_eq!(crossfade2.next_sample(), 1.0);

        let a3 = ConstantSignal(1.0);
        let b3 = ConstantSignal(3.0);
        let mut crossfade3 = Crossfade {
            a: a3,
            b: b3,
            mix: 1.0.into(),
        };
        assert_eq!(crossfade3.next_sample(), 3.0);
    }

    #[test]
    fn test_min() {
        let a = ConstantSignal(2.0);
        let b = ConstantSignal(3.0);
        let mut min_signal = Min { a, b };
        assert_eq!(min_signal.next_sample(), 2.0);
    }

    #[test]
    fn test_max() {
        let a = ConstantSignal(2.0);
        let b = ConstantSignal(3.0);
        let mut max_signal = Max { a, b };
        assert_eq!(max_signal.next_sample(), 3.0);
    }

    #[test]
    fn test_abs() {
        let source = ConstantSignal(-2.0);
        let mut abs_signal = Abs { source };
        assert_eq!(abs_signal.next_sample(), 2.0);
    }

    #[test]
    fn test_gate() {
        let source = ConstantSignal(0.05);
        let mut gated = Gate {
            source,
            threshold: 0.1.into(),
        };
        assert_eq!(gated.next_sample(), 0.0);

        let source2 = ConstantSignal(0.2);
        let mut gated2 = Gate {
            source: source2,
            threshold: 0.1.into(),
        };
        assert_eq!(gated2.next_sample(), 0.2);
    }

    #[test]
    fn test_signal_ext_new_combinators() {
        let source = ConstantSignal(2.0);
        let mut clamped = source.clamp(-1.0, 1.0);
        assert_eq!(clamped.next_sample(), 1.0);

        let source2 = ConstantSignal(2.0);
        let mut inverted = source2.invert();
        assert_eq!(inverted.next_sample(), -2.0);

        let source3 = ConstantSignal(-2.0);
        let mut abs_signal = source3.abs();
        assert_eq!(abs_signal.next_sample(), 2.0);
    }

    #[test]
    fn test_complex_chain_with_new_combinators() {
        let a = ConstantSignal(2.0);
        let b = ConstantSignal(1.0);

        // (2.0 + 1.0) * 0.5 = 1.5, clamped to [0.0, 1.0] = 1.0
        let mut signal = a.add(b).gain(0.5).clamp(0.0, 1.0);
        assert_eq!(signal.next_sample(), 1.0);
    }

}
