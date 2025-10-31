//! Extension trait for audio signals that adds convenient synthesis methods.
//!
//! This trait is only available when the `synth` feature is enabled.

use crate::core::{AudioSignal, Param};
use crate::synthesis::effects::{
    Bitcrusher, Compressor, Delay, Distortion, Limiter, Tremolo, Vibrato,
};
use crate::synthesis::filters::BiquadFilter;

/// Extension trait providing convenient filter and effect methods for audio signals.
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
/// let osc = SineOscillator::<44100>::new(440.0);
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
    /// let osc = SineOscillator::<44100>::new(440.0);
    /// let mut filtered = osc.lowpass_filter(1000.0, 0.707);
    /// ```
    fn lowpass_filter(
        self,
        cutoff: impl Into<Param>,
        q: impl Into<Param>,
    ) -> BiquadFilter<SAMPLE_RATE, Self> {
        BiquadFilter::lowpass(self, cutoff, q)
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
    /// let osc = SineOscillator::<44100>::new(440.0);
    /// let mut filtered = osc.highpass_filter(100.0, 0.707);
    /// ```
    fn highpass_filter(
        self,
        cutoff: impl Into<Param>,
        q: impl Into<Param>,
    ) -> BiquadFilter<SAMPLE_RATE, Self> {
        BiquadFilter::highpass(self, cutoff, q)
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
    /// let osc = SineOscillator::<44100>::new(440.0);
    /// let mut filtered = osc.bandpass_filter(440.0, 5.0);
    /// ```
    fn bandpass_filter(
        self,
        center: impl Into<Param>,
        q: impl Into<Param>,
    ) -> BiquadFilter<SAMPLE_RATE, Self> {
        BiquadFilter::bandpass(self, center, q)
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
    /// let osc = SineOscillator::<44100>::new(440.0);
    /// let mut filtered = osc.notch_filter(440.0, 5.0);
    /// ```
    fn notch_filter(
        self,
        center: impl Into<Param>,
        q: impl Into<Param>,
    ) -> BiquadFilter<SAMPLE_RATE, Self> {
        BiquadFilter::notch(self, center, q)
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
    /// let osc = SineOscillator::<44100>::new(440.0);
    /// let mut filtered = osc.allpass_filter(1000.0, 0.707);
    /// ```
    fn allpass_filter(
        self,
        frequency: impl Into<Param>,
        q: impl Into<Param>,
    ) -> BiquadFilter<SAMPLE_RATE, Self> {
        BiquadFilter::allpass(self, frequency, q)
    }

    // ===== Effects =====

    /// Applies a tremolo effect (amplitude modulation) to this audio signal.
    ///
    /// Tremolo creates a rhythmic variation in volume using an LFO to modulate amplitude.
    ///
    /// # Arguments
    ///
    /// * `rate` - Tremolo rate in Hz (typically 3-10 Hz for classic tremolo)
    /// * `depth` - Modulation depth, 0.0-1.0 (0.0 = no effect, 1.0 = full tremolo)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, AudioSignalExt};
    ///
    /// let osc = SineOscillator::<44100>::new(440.0);
    /// let mut tremolo = osc.tremolo(5.0, 0.5);
    /// ```
    fn tremolo(self, rate: f64, depth: impl Into<Param>) -> Tremolo<SAMPLE_RATE, Self> {
        Tremolo::with_rate(self, rate, depth)
    }

    /// Applies a vibrato effect (pitch modulation) to this audio signal.
    ///
    /// Vibrato modulates the pitch up and down using a variable delay line.
    ///
    /// # Arguments
    ///
    /// * `rate` - Vibrato rate in Hz (typically 2-8 Hz)
    /// * `depth` - Pitch deviation in cents (100 cents = 1 semitone, typically 10-50)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, AudioSignalExt};
    ///
    /// let osc = SineOscillator::<44100>::new(440.0);
    /// let mut vibrato = osc.vibrato(5.0, 20.0);
    /// ```
    fn vibrato(
        self,
        rate: impl Into<Param>,
        depth: impl Into<Param>,
    ) -> Vibrato<SAMPLE_RATE, Self> {
        Vibrato::new(self, rate, depth)
    }

    /// Applies a delay effect to this audio signal.
    ///
    /// Creates echoes by feeding back delayed copies of the signal.
    ///
    /// # Arguments
    ///
    /// * `max_delay_time` - Maximum delay time in seconds (determines buffer size)
    /// * `delay_time` - Delay time in seconds (can be fixed or modulated)
    /// * `feedback` - Feedback amount, 0.0-1.0 (0.0 = single echo, higher = more repeats)
    /// * `mix` - Dry/wet mix, 0.0-1.0 (0.0 = dry only, 1.0 = wet only)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, AudioSignalExt};
    ///
    /// let osc = SineOscillator::<44100>::new(440.0);
    /// // 500ms max delay, starting at 300ms, with feedback and 50% mix
    /// let mut delayed = osc.delay(0.5, 0.3, 0.3, 0.5);
    /// ```
    fn delay(
        self,
        max_delay_time: f64,
        delay_time: impl Into<Param>,
        feedback: impl Into<Param>,
        mix: impl Into<Param>,
    ) -> Delay<SAMPLE_RATE, Self> {
        Delay::new(self, max_delay_time, delay_time, feedback, mix)
    }

    /// Applies distortion to this audio signal.
    ///
    /// Uses waveshaping to add harmonic content and grit.
    ///
    /// # Arguments
    ///
    /// * `drive` - Distortion amount, 1.0+ (1.0 = no distortion, higher = more distortion)
    /// * `mix` - Dry/wet mix, 0.0-1.0 (0.0 = dry only, 1.0 = wet only)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, AudioSignalExt};
    ///
    /// let osc = SineOscillator::<44100>::new(440.0);
    /// let mut distorted = osc.distortion(5.0, 0.7);
    /// ```
    fn distortion(
        self,
        drive: impl Into<Param>,
        mix: impl Into<Param>,
    ) -> Distortion<SAMPLE_RATE, Self> {
        Distortion::new(self, drive, mix)
    }

    // ===== Dynamics Processing =====

    /// Applies a compressor to control the dynamic range of this audio signal.
    ///
    /// Reduces loud parts and can bring up quiet parts for more consistent levels.
    ///
    /// # Arguments
    ///
    /// * `threshold` - Level above which compression starts (0.0-1.0, typically 0.3-0.7)
    /// * `ratio` - Compression ratio (1.0 = no compression, 4.0 = 4:1, higher = more compression)
    /// * `attack` - Attack time in seconds (how quickly compression engages, typically 0.001-0.1)
    /// * `release` - Release time in seconds (how quickly compression releases, typically 0.05-1.0)
    /// * `knee` - Soft knee width in dB (0 = hard knee, 6-12 = soft knee)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, AudioSignalExt};
    ///
    /// let osc = SineOscillator::<44100>::new(440.0);
    /// let mut compressed = osc.compressor(0.5, 4.0, 0.01, 0.1, 0.0);
    /// ```
    fn compressor(
        self,
        threshold: impl Into<Param>,
        ratio: impl Into<Param>,
        attack: impl Into<Param>,
        release: impl Into<Param>,
        knee: impl Into<Param>,
    ) -> Compressor<SAMPLE_RATE, Self> {
        Compressor::new(self, threshold, ratio, attack, release, knee)
    }

    /// Applies a limiter to prevent clipping of this audio signal.
    ///
    /// A limiter is like a compressor with infinite ratio and instant attack,
    /// preventing any samples from exceeding the threshold.
    ///
    /// # Arguments
    ///
    /// * `threshold` - Maximum allowed amplitude (0.0-1.0, typically 0.8-0.95)
    /// * `release` - Release time in seconds (how quickly gain returns to unity)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, AudioSignalExt};
    ///
    /// let osc = SineOscillator::<44100>::new(440.0);
    /// let mut limited = osc.limiter(0.9, 0.05);
    /// ```
    fn limiter(
        self,
        threshold: impl Into<Param>,
        release: impl Into<Param>,
    ) -> Limiter<SAMPLE_RATE, Self> {
        Limiter::new(self, threshold, release)
    }

    // ===== Lo-Fi / Degradation =====

    /// Applies bitcrusher effect to this audio signal.
    ///
    /// Reduces bit depth and sample rate for lo-fi, digital distortion effects.
    ///
    /// # Arguments
    ///
    /// * `bit_depth` - Number of bits (1-16, lower = more distortion)
    /// * `sample_rate_reduction` - Sample rate divisor (1.0 = no reduction, higher = more aliasing)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, AudioSignalExt};
    ///
    /// let osc = SineOscillator::<44100>::new(440.0);
    /// let mut crushed = osc.bitcrusher(8.0, 4.0);
    /// ```
    fn bitcrusher(
        self,
        bit_depth: impl Into<Param>,
        sample_rate_reduction: impl Into<Param>,
    ) -> Bitcrusher<SAMPLE_RATE, Self> {
        Bitcrusher::new(self, bit_depth, sample_rate_reduction)
    }
}

// Blanket implementation for all AudioSignal types
impl<T: AudioSignal<SAMPLE_RATE>, const SAMPLE_RATE: u32> AudioSignalExt<SAMPLE_RATE> for T {}
