//! Distortion effect with drive and dry/wet mix.

use crate::core::{AudioSignal, Param, Signal};

/// Distortion effect that applies gain and clipping to create harmonic distortion.
///
/// The distortion effect works by:
/// 1. Amplifying the input signal by the drive amount (pre-gain)
/// 2. Applying soft clipping using a tanh function to add harmonics
/// 3. Mixing the distorted signal with the dry signal based on the mix parameter
///
/// # Examples
///
/// ```
/// use earworm::{SineOscillator, Distortion};
///
/// // Create a 440 Hz tone with moderate distortion
/// let osc = SineOscillator::<44100>::new(440.0);
/// let mut distortion = Distortion::new(osc, 5.0, 0.7);
/// ```
pub struct Distortion<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> {
    source: S,
    drive: Param, // Pre-gain before clipping (1.0 = unity, higher = more distortion)
    mix: Param,   // Dry/wet mix (0.0 = all dry, 1.0 = all wet)
}

impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> Distortion<SAMPLE_RATE, S> {
    /// Creates a new distortion effect.
    ///
    /// # Arguments
    ///
    /// * `source` - Input signal to distort
    /// * `drive` - Drive amount (pre-gain before clipping). 1.0 = no distortion,
    ///   2-5 = light distortion, 5-20 = heavy distortion, 20+ = extreme fuzz
    /// * `mix` - Dry/wet mix (0.0 = all dry/original, 1.0 = all wet/distorted)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, Distortion};
    ///
    /// let audio = SineOscillator::<44100>::new(440.0);
    ///
    /// // Light overdrive
    /// let mut light = Distortion::new(audio, 3.0, 0.5);
    ///
    /// // Heavy distortion
    /// let audio = SineOscillator::<44100>::new(440.0);
    /// let mut heavy = Distortion::new(audio, 15.0, 0.8);
    /// ```
    pub fn new(source: S, drive: impl Into<Param>, mix: impl Into<Param>) -> Self {
        Self {
            source,
            drive: drive.into(),
            mix: mix.into(),
        }
    }

    /// Creates a light overdrive effect (subtle warmth and harmonics).
    ///
    /// Typical drive: 2-3, mix: 0.5-0.7
    pub fn overdrive(source: S) -> Self {
        Self::new(source, 3.0, 0.6)
    }

    /// Creates a moderate distortion effect (classic rock/blues tone).
    ///
    /// Typical drive: 5-8, mix: 0.7-0.85
    pub fn classic(source: S) -> Self {
        Self::new(source, 7.0, 0.8)
    }

    /// Creates a heavy distortion/fuzz effect (aggressive, saturated tone).
    ///
    /// Typical drive: 15-30, mix: 0.85-1.0
    pub fn fuzz(source: S) -> Self {
        Self::new(source, 20.0, 0.9)
    }
}

impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> Signal for Distortion<SAMPLE_RATE, S> {
    fn next_sample(&mut self) -> f64 {
        let dry = self.source.next_sample();

        // Get current parameter values
        let drive = self.drive.value().max(0.0);
        let mix = self.mix.value().clamp(0.0, 1.0);

        // Apply drive (pre-gain)
        let driven = dry * drive;

        // Apply soft clipping using tanh
        // tanh provides smooth saturation with natural-sounding harmonics
        // At low drive (1-3): subtle compression and warmth
        // At medium drive (5-10): clear distortion with preserved dynamics
        // At high drive (15+): heavy saturation and fuzz
        let wet = driven.tanh();

        // Compensate for gain from tanh (approximately)
        // tanh approaches ±1, so we scale to maintain reasonable output levels
        let wet = wet * 0.7;

        // Mix dry and wet signals
        dry * (1.0 - mix) + wet * mix
    }
}

impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> AudioSignal<SAMPLE_RATE>
    for Distortion<SAMPLE_RATE, S>
{
}
