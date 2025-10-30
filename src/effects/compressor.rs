//! Compressor effect for dynamic range control.

use crate::signals::{AudioSignal, Param, Signal};

/// Compressor effect for controlling dynamic range.
///
/// A compressor reduces the dynamic range of audio by applying gain reduction
/// when the input signal exceeds a threshold. Unlike a limiter (which has an
/// infinite ratio), a compressor allows configurable ratio, attack, and release
/// times for more musical and transparent dynamic control.
///
/// The compressor uses RMS (root mean square) level detection for a more
/// natural, musical response compared to peak detection.
///
/// # Parameters
///
/// - **Threshold**: Level above which compression starts (typically -20 to 0 dB, or 0.1 to 1.0 linear)
/// - **Ratio**: Amount of compression (1:1 = no compression, 4:1 = moderate, 10:1 = heavy, ∞:1 = limiting)
/// - **Attack**: How quickly compression engages (0.001-0.1s typical)
/// - **Release**: How quickly compression disengages (0.05-1.0s typical)
/// - **Knee**: Smoothness of compression onset (0 = hard knee, >0 = soft knee)
///
/// # Examples
///
/// ```
/// use earworm::{SineOscillator, Compressor};
///
/// // Create a basic compressor
/// let osc = SineOscillator::<44100>::new(440.0);
/// let mut comp = Compressor::new(osc, 0.5, 4.0, 0.01, 0.1, 0.0);
/// ```
pub struct Compressor<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> {
    source: S,
    threshold: Param,     // threshold level (linear, 0.0-1.0)
    ratio: Param,         // compression ratio (1.0 = no compression, higher = more compression)
    attack: Param,        // attack time in seconds
    release: Param,       // release time in seconds
    knee: Param,          // knee width in dB (0 = hard knee)
    current_gain: f64,    // current gain reduction multiplier
    rms_buffer: Vec<f64>, // circular buffer for RMS calculation
    rms_index: usize,     // current position in RMS buffer
}

impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> Compressor<SAMPLE_RATE, S> {
    /// Creates a new compressor effect.
    ///
    /// # Arguments
    ///
    /// * `source` - Input audio signal
    /// * `threshold` - Threshold level (0.0-1.0 linear, typically 0.3-0.7)
    /// * `ratio` - Compression ratio (1.0 = no compression, 4.0 = 4:1, 20.0 = 20:1)
    /// * `attack` - Attack time in seconds (how quickly compression engages, typically 0.001-0.1)
    /// * `release` - Release time in seconds (how quickly compression releases, typically 0.05-1.0)
    /// * `knee` - Knee width in dB (0 = hard knee, 6-12 = soft knee)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, Compressor, SignalExt};
    ///
    /// let audio = SineOscillator::<44100>::new(440.0).gain(1.5);
    /// // Compress with 4:1 ratio, 10ms attack, 100ms release
    /// let mut comp = Compressor::new(audio, 0.5, 4.0, 0.01, 0.1, 0.0);
    /// ```
    pub fn new(
        source: S,
        threshold: impl Into<Param>,
        ratio: impl Into<Param>,
        attack: impl Into<Param>,
        release: impl Into<Param>,
        knee: impl Into<Param>,
    ) -> Self {
        // Use 10ms RMS window
        let rms_window_size = ((SAMPLE_RATE as f64) * 0.01) as usize;

        Self {
            source,
            threshold: threshold.into(),
            ratio: ratio.into(),
            attack: attack.into(),
            release: release.into(),
            knee: knee.into(),
            current_gain: 1.0,
            rms_buffer: vec![0.0; rms_window_size],
            rms_index: 0,
        }
    }

    /// Creates a gentle "vocal" compressor preset.
    ///
    /// Settings: threshold 0.5, ratio 3:1, attack 5ms, release 100ms, soft knee 6dB
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, Compressor};
    ///
    /// let audio = SineOscillator::<44100>::new(440.0);
    /// let mut comp = Compressor::vocal(audio);
    /// ```
    pub fn vocal(source: S) -> Self {
        Self::new(source, 0.5, 3.0, 0.005, 0.1, 6.0)
    }

    /// Creates a "punch" compressor for drums/percussion.
    ///
    /// Settings: threshold 0.6, ratio 4:1, attack 30ms, release 150ms, hard knee
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, Compressor};
    ///
    /// let audio = SineOscillator::<44100>::new(440.0);
    /// let mut comp = Compressor::punch(audio);
    /// ```
    pub fn punch(source: S) -> Self {
        Self::new(source, 0.6, 4.0, 0.03, 0.15, 0.0)
    }

    /// Creates a "glue" compressor for bus/master compression.
    ///
    /// Settings: threshold 0.7, ratio 2:1, attack 10ms, release 300ms, soft knee 12dB
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, Compressor};
    ///
    /// let audio = SineOscillator::<44100>::new(440.0);
    /// let mut comp = Compressor::glue(audio);
    /// ```
    pub fn glue(source: S) -> Self {
        Self::new(source, 0.7, 2.0, 0.01, 0.3, 12.0)
    }

    /// Converts linear amplitude to decibels.
    fn lin_to_db(linear: f64) -> f64 {
        20.0 * linear.max(0.0001).log10()
    }

    /// Converts decibels to linear amplitude.
    fn db_to_lin(db: f64) -> f64 {
        10.0_f64.powf(db / 20.0)
    }

    /// Calculates RMS level from the circular buffer.
    fn calculate_rms(&self) -> f64 {
        let sum: f64 = self.rms_buffer.iter().map(|x| x * x).sum();
        (sum / self.rms_buffer.len() as f64).sqrt()
    }

    /// Gets the current gain reduction multiplier (0.0-1.0).
    /// 1.0 means no reduction, 0.5 means -6dB reduction, etc.
    pub fn current_gain(&self) -> f64 {
        self.current_gain
    }
}

impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> Signal for Compressor<SAMPLE_RATE, S> {
    fn next_sample(&mut self) -> f64 {
        let input = self.source.next_sample();

        // Update RMS buffer
        self.rms_buffer[self.rms_index] = input.abs();
        self.rms_index = (self.rms_index + 1) % self.rms_buffer.len();

        // Get current RMS level
        let rms_level = self.calculate_rms();

        // Get parameter values
        let threshold = self.threshold.value().max(0.0001);
        let ratio = self.ratio.value().max(1.0);
        let attack_time = self.attack.value().max(0.0001);
        let release_time = self.release.value().max(0.0001);
        let knee_db = self.knee.value().max(0.0);

        // Convert to dB
        let input_db = Self::lin_to_db(rms_level);
        let threshold_db = Self::lin_to_db(threshold);

        // Calculate gain reduction in dB
        let mut gain_reduction_db = 0.0;

        if knee_db > 0.0 {
            // Soft knee compression
            let knee_start = threshold_db - knee_db / 2.0;
            let knee_end = threshold_db + knee_db / 2.0;

            if input_db > knee_end {
                // Above knee - full compression
                let over_db = input_db - threshold_db;
                gain_reduction_db = over_db - (over_db / ratio);
            } else if input_db > knee_start {
                // In knee region - gradual compression
                let knee_input = input_db - knee_start;
                let knee_ratio = knee_input / knee_db;
                let over_db = input_db - threshold_db;
                gain_reduction_db = knee_ratio * (over_db - (over_db / ratio));
            }
            // Below knee - no compression
        } else {
            // Hard knee compression
            if input_db > threshold_db {
                let over_db = input_db - threshold_db;
                gain_reduction_db = over_db - (over_db / ratio);
            }
        }

        // Convert gain reduction to linear
        let target_gain = Self::db_to_lin(-gain_reduction_db);

        // Apply attack/release smoothing
        let time_constant = if target_gain < self.current_gain {
            // Attack: gain is decreasing (more compression)
            attack_time
        } else {
            // Release: gain is increasing (less compression)
            release_time
        };

        let coeff = 1.0 - (-1.0 / (time_constant * SAMPLE_RATE as f64)).exp();
        self.current_gain += (target_gain - self.current_gain) * coeff;

        // Apply compression
        input * self.current_gain
    }
}

impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> AudioSignal<SAMPLE_RATE>
    for Compressor<SAMPLE_RATE, S>
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConstantSignal;

    #[test]
    fn test_no_compression_below_threshold() {
        let source = ConstantSignal::<44100>(0.3);
        let mut comp = Compressor::new(source, 0.5, 4.0, 0.01, 0.1, 0.0);

        // Let RMS buffer fill
        for _ in 0..100 {
            comp.next_sample();
        }

        // Sample should be relatively unchanged (within tolerance for RMS smoothing)
        let output = comp.next_sample();
        assert!((output - 0.3).abs() < 0.05, "Output: {}", output);
    }

    #[test]
    fn test_compression_above_threshold() {
        let source = ConstantSignal::<44100>(0.8);
        let mut comp = Compressor::new(source, 0.5, 4.0, 0.01, 0.1, 0.0);

        // Let compression settle
        for _ in 0..1000 {
            comp.next_sample();
        }

        // Output should be compressed (less than input)
        let output = comp.next_sample();
        assert!(
            output < 0.8,
            "Output {} should be less than input 0.8",
            output
        );
        assert!(output > 0.0, "Output should be positive");
    }

    #[test]
    fn test_ratio_1_is_unity() {
        let source = ConstantSignal::<44100>(0.8);
        let mut comp = Compressor::new(source, 0.5, 1.0, 0.01, 0.1, 0.0);

        // With ratio 1.0, no compression should occur
        for _ in 0..1000 {
            comp.next_sample();
        }

        let output = comp.next_sample();
        assert!(
            (output - 0.8).abs() < 0.05,
            "With ratio 1:1, output should equal input"
        );
    }

    #[test]
    fn test_audio_signal_trait() {
        let source = ConstantSignal::<44100>(0.5);
        let comp = Compressor::new(source, 0.5, 4.0, 0.01, 0.1, 0.0);

        // Just verify it implements AudioSignal
        fn assert_audio_signal<T: AudioSignal<44100>>(_: T) {}
        assert_audio_signal(comp);
    }
}
