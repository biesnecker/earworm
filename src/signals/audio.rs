//! Audio signal traits and extensions for sample-rate-aware signals.

use crate::{Param, Signal};

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

/// Extension trait providing convenient filter methods for audio signals.
///
/// This trait is automatically implemented for all types that implement `AudioSignal`,
/// providing easy access to filtering operations without needing to manually pass
/// the sample rate (since `AudioSignal` already provides it).
///
/// # Examples
///
/// ```
/// use earworm::{SineOscillator, AudioSignalExt};
///
/// let osc = SineOscillator::new(440.0, 44100.0);
/// // No need to pass sample_rate - it's automatically obtained from the AudioSignal
/// let mut filtered = osc.lowpass_filter(1000.0, 0.707);
/// ```
pub trait AudioSignalExt<const SAMPLE_RATE: u32>: AudioSignal<SAMPLE_RATE> + Sized {
    /// Applies a low-pass filter to this audio signal.
    ///
    /// The sample rate is automatically obtained from the `AudioSignal` trait.
    ///
    /// # Arguments
    ///
    /// * `cutoff` - Cutoff frequency in Hz (can be fixed or modulated)
    /// * `q` - Q factor/resonance, typically 0.5-10.0 (can be fixed or modulated)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, AudioSignalExt};
    ///
    /// let osc = SineOscillator::new(440.0, 44100.0);
    /// let mut filtered = osc.lowpass_filter(1000.0, 0.707);
    /// ```
    fn lowpass_filter(
        self,
        cutoff: impl Into<Param>,
        q: impl Into<Param>,
    ) -> crate::filters::BiquadFilter<SAMPLE_RATE, Self> {
        crate::filters::BiquadFilter::lowpass(self, cutoff, q)
    }

    /// Applies a high-pass filter to this audio signal.
    ///
    /// The sample rate is automatically obtained from the `AudioSignal` trait.
    ///
    /// # Arguments
    ///
    /// * `cutoff` - Cutoff frequency in Hz (can be fixed or modulated)
    /// * `q` - Q factor/resonance, typically 0.5-10.0 (can be fixed or modulated)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, AudioSignalExt};
    ///
    /// let osc = SineOscillator::new(440.0, 44100.0);
    /// let mut filtered = osc.highpass_filter(100.0, 0.707);
    /// ```
    fn highpass_filter(
        self,
        cutoff: impl Into<Param>,
        q: impl Into<Param>,
    ) -> crate::filters::BiquadFilter<SAMPLE_RATE, Self> {
        crate::filters::BiquadFilter::highpass(self, cutoff, q)
    }

    /// Applies a band-pass filter to this audio signal.
    ///
    /// The sample rate is automatically obtained from the `AudioSignal` trait.
    ///
    /// # Arguments
    ///
    /// * `center` - Center frequency in Hz (can be fixed or modulated)
    /// * `q` - Q factor/bandwidth, typically 0.5-10.0. Higher = narrower band (can be fixed or modulated)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, AudioSignalExt};
    ///
    /// let osc = SineOscillator::new(440.0, 44100.0);
    /// let mut filtered = osc.bandpass_filter(440.0, 5.0);
    /// ```
    fn bandpass_filter(
        self,
        center: impl Into<Param>,
        q: impl Into<Param>,
    ) -> crate::filters::BiquadFilter<SAMPLE_RATE, Self> {
        crate::filters::BiquadFilter::bandpass(self, center, q)
    }

    /// Applies a notch filter (band-reject) to this audio signal.
    ///
    /// The sample rate is automatically obtained from the `AudioSignal` trait.
    ///
    /// # Arguments
    ///
    /// * `center` - Center frequency to reject in Hz (can be fixed or modulated)
    /// * `q` - Q factor/notch width, typically 0.5-10.0. Higher = narrower notch (can be fixed or modulated)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, AudioSignalExt};
    ///
    /// let osc = SineOscillator::new(440.0, 44100.0);
    /// let mut filtered = osc.notch_filter(440.0, 5.0);
    /// ```
    fn notch_filter(
        self,
        center: impl Into<Param>,
        q: impl Into<Param>,
    ) -> crate::filters::BiquadFilter<SAMPLE_RATE, Self> {
        crate::filters::BiquadFilter::notch(self, center, q)
    }

    /// Applies an all-pass filter to this audio signal.
    ///
    /// All-pass filters pass all frequencies but shift their phase.
    /// Useful for phaser effects and creating complementary signals.
    ///
    /// The sample rate is automatically obtained from the `AudioSignal` trait.
    ///
    /// # Arguments
    ///
    /// * `frequency` - Center frequency for phase shift in Hz (can be fixed or modulated)
    /// * `q` - Q factor, affects phase response (can be fixed or modulated)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, AudioSignalExt};
    ///
    /// let osc = SineOscillator::new(440.0, 44100.0);
    /// let mut filtered = osc.allpass_filter(1000.0, 0.707);
    /// ```
    fn allpass_filter(
        self,
        frequency: impl Into<Param>,
        q: impl Into<Param>,
    ) -> crate::filters::BiquadFilter<SAMPLE_RATE, Self> {
        crate::filters::BiquadFilter::allpass(self, frequency, q)
    }
}

// Blanket implementation for all AudioSignal types
impl<T: AudioSignal<SAMPLE_RATE>, const SAMPLE_RATE: u32> AudioSignalExt<SAMPLE_RATE> for T {}
