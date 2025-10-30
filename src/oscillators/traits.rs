//! Core trait definitions for oscillators and audio signal extensions.

use crate::{Param, Signal};

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
pub trait AudioSignalExt: AudioSignal + Sized {
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
    ) -> crate::filters::BiquadFilter<Self> {
        let sample_rate = self.sample_rate();
        crate::filters::BiquadFilter::lowpass(self, cutoff, q, sample_rate)
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
    ) -> crate::filters::BiquadFilter<Self> {
        let sample_rate = self.sample_rate();
        crate::filters::BiquadFilter::highpass(self, cutoff, q, sample_rate)
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
    ) -> crate::filters::BiquadFilter<Self> {
        let sample_rate = self.sample_rate();
        crate::filters::BiquadFilter::bandpass(self, center, q, sample_rate)
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
    ) -> crate::filters::BiquadFilter<Self> {
        let sample_rate = self.sample_rate();
        crate::filters::BiquadFilter::notch(self, center, q, sample_rate)
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
    ) -> crate::filters::BiquadFilter<Self> {
        let sample_rate = self.sample_rate();
        crate::filters::BiquadFilter::allpass(self, frequency, q, sample_rate)
    }
}

// Blanket implementation for all AudioSignal types
impl<T: AudioSignal> AudioSignalExt for T {}
