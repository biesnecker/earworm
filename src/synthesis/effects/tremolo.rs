//! Tremolo effect (amplitude modulation).

use crate::core::{AudioSignal, Param, Signal};

/// Tremolo effect that modulates the amplitude of an audio signal.
///
/// Tremolo creates a rhythmic variation in volume by multiplying the input signal
/// with a modulating waveform (typically a sine wave LFO). This creates the classic
/// "trembling" or pulsing sound effect.
///
/// # Examples
///
/// ```
/// use earworm::{SineOscillator, Tremolo, AudioSignalExt};
///
/// // Create a 440 Hz tone with 5 Hz tremolo
/// let osc = SineOscillator::<44100>::new(440.0);
/// let lfo = SineOscillator::<44100>::new(5.0);
/// let mut tremolo = Tremolo::new(osc, lfo, 0.5);
/// ```
pub struct Tremolo<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> {
    pub(crate) source: S,
    modulator: Param,
    depth: Param,
}

impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> Tremolo<SAMPLE_RATE, S> {
    /// Creates a new tremolo effect.
    ///
    /// # Arguments
    ///
    /// * `source` - Input audio signal to modulate
    /// * `modulator` - Modulation source (typically an LFO, often 3-10 Hz for tremolo)
    /// * `depth` - Modulation depth (0.0 = no effect, 1.0 = full tremolo)
    ///
    /// The modulator output is expected to be in the range [-1, 1]. It will be scaled
    /// and offset based on depth to create amplitude modulation:
    /// - At depth 0.0: output = input (no modulation)
    /// - At depth 1.0: output varies from 0 to input (full tremolo)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, Tremolo};
    ///
    /// let audio = SineOscillator::<44100>::new(440.0);
    /// let lfo = SineOscillator::<44100>::new(6.0);
    /// let mut tremolo = Tremolo::new(audio, lfo, 0.8);
    /// ```
    pub fn new(source: S, modulator: impl Into<Param>, depth: impl Into<Param>) -> Self {
        Self {
            source,
            modulator: modulator.into(),
            depth: depth.into(),
        }
    }

    /// Creates a tremolo effect with a fixed rate (uses internal sine LFO).
    ///
    /// This is a convenience method for common use cases where you just want
    /// simple tremolo with a sine wave modulator.
    ///
    /// # Arguments
    ///
    /// * `source` - Input audio signal
    /// * `rate` - Tremolo rate in Hz (typically 3-10 Hz)
    /// * `depth` - Modulation depth (0.0-1.0)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, Tremolo};
    ///
    /// let audio = SineOscillator::<44100>::new(440.0);
    /// let mut tremolo = Tremolo::with_rate(audio, 5.0, 0.5);
    /// ```
    pub fn with_rate(source: S, rate: f64, depth: impl Into<Param>) -> Self {
        let lfo = crate::synthesis::oscillators::SineOscillator::<SAMPLE_RATE>::new(rate);
        Self::new(source, lfo, depth)
    }
}

impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> Signal for Tremolo<SAMPLE_RATE, S> {
    fn next_sample(&mut self) -> f64 {
        let input = self.source.next_sample();
        let depth = self.depth.value().clamp(0.0, 1.0);

        // Get modulator value (expected in range [-1, 1])
        let mod_value = self.modulator.value();

        // Convert modulator from [-1, 1] to a gain multiplier
        // depth=0: gain always 1.0 (no effect)
        // depth=1: gain varies from 0.0 to 1.0 (full tremolo)
        // Formula: gain = 1.0 - depth * (1.0 - (mod_value + 1.0) / 2.0)
        //        = 1.0 - depth * (1.0 - 0.5 - mod_value/2.0)
        //        = 1.0 - depth * (0.5 - mod_value/2.0)
        //        = 1.0 - depth/2.0 + depth*mod_value/2.0
        //        = 1.0 - depth/2.0 * (1.0 - mod_value)
        // Actually, let's use a simpler formula:
        // gain = 1.0 - depth/2.0 + depth/2.0 * mod_value
        //      = 1.0 + depth/2.0 * (mod_value - 1.0)
        // When mod_value = 1: gain = 1.0
        // When mod_value = -1: gain = 1.0 - depth
        // When mod_value = 0: gain = 1.0 - depth/2.0
        let gain = 1.0 + depth / 2.0 * (mod_value - 1.0);

        input * gain
    }
}

impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> AudioSignal<SAMPLE_RATE>
    for Tremolo<SAMPLE_RATE, S>
{
}
