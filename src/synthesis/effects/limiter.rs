//! Limiter effect for preventing clipping.

use crate::core::{AudioSignal, Param, Signal};

/// Limiter effect that prevents audio from exceeding a threshold.
///
/// A limiter is a type of dynamics processor that applies gain reduction when
/// the input signal exceeds a threshold, preventing clipping and maintaining
/// headroom. Unlike a compressor, a limiter uses a very high ratio (effectively
/// infinite) and fast attack to "brick wall" limit the output.
///
/// The limiter tracks the peak amplitude and smoothly reduces gain when needed,
/// using the release parameter to determine how quickly gain returns to unity
/// after the signal drops below the threshold.
///
/// # Examples
///
/// ```
/// use earworm::{SineOscillator, Limiter};
///
/// // Create a loud oscillator and limit it to prevent clipping
/// let osc = SineOscillator::<44100>::new(440.0);
/// let mut limiter = Limiter::new(osc, 0.9, 0.1);
/// ```
pub struct Limiter<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> {
    source: S,
    threshold: Param,
    release: Param, // release time in seconds
    current_gain: f64,
}

impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> Limiter<SAMPLE_RATE, S> {
    /// Creates a new limiter effect.
    ///
    /// # Arguments
    ///
    /// * `source` - Input audio signal
    /// * `threshold` - Maximum allowed amplitude (0.0-1.0, typically 0.8-0.95)
    /// * `release` - Release time in seconds (how quickly gain returns to unity)
    ///
    /// The attack time is intentionally very fast (instant) to prevent any samples
    /// from exceeding the threshold. The release time controls how quickly the
    /// gain reduction is released after the signal drops below threshold.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, Limiter, SignalExt};
    ///
    /// let audio = SineOscillator::<44100>::new(440.0).gain(2.0);
    /// // Limit to 0.9 with 100ms release time
    /// let mut limiter = Limiter::new(audio, 0.9, 0.1);
    /// ```
    pub fn new(source: S, threshold: impl Into<Param>, release: impl Into<Param>) -> Self {
        Self {
            source,
            threshold: threshold.into(),
            release: release.into(),
            current_gain: 1.0,
        }
    }

    /// Creates a "safety" limiter with conservative settings.
    ///
    /// Uses a threshold of 0.95 and release time of 50ms, suitable for
    /// preventing clipping at the output stage without audible artifacts.
    ///
    /// # Arguments
    ///
    /// * `source` - Input audio signal
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, Limiter};
    ///
    /// let audio = SineOscillator::<44100>::new(440.0);
    /// let mut limiter = Limiter::safety(audio);
    /// ```
    pub fn safety(source: S) -> Self {
        Self::new(source, 0.95, 0.05)
    }

    /// Creates a "brick wall" limiter with fast release.
    ///
    /// Uses a threshold of 0.9 and very fast release time of 10ms,
    /// suitable for aggressive limiting and maximizing loudness.
    ///
    /// # Arguments
    ///
    /// * `source` - Input audio signal
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, Limiter};
    ///
    /// let audio = SineOscillator::<44100>::new(440.0);
    /// let mut limiter = Limiter::brick_wall(audio);
    /// ```
    pub fn brick_wall(source: S) -> Self {
        Self::new(source, 0.9, 0.01)
    }

    /// Gets the current gain reduction multiplier (0.0-1.0).
    /// 1.0 means no reduction, 0.5 means -6dB reduction, etc.
    pub fn current_gain(&self) -> f64 {
        self.current_gain
    }
}

impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> Signal for Limiter<SAMPLE_RATE, S> {
    fn next_sample(&mut self) -> f64 {
        let input = self.source.next_sample();
        let threshold = self.threshold.value().max(0.0);
        let release_time = self.release.value().max(0.0001); // Minimum 0.1ms to avoid instability

        // Calculate the absolute amplitude of the input
        let input_level = input.abs();

        // Determine target gain
        let target_gain = if input_level > threshold {
            // Need to reduce gain to prevent exceeding threshold
            threshold / input_level.max(0.0001) // Avoid division by zero
        } else {
            // No limiting needed, return to unity gain
            1.0
        };

        // Instant attack (take the lower gain immediately)
        // Smooth release (gradually return to higher gain)
        if target_gain < self.current_gain {
            // Attack: instant
            self.current_gain = target_gain;
        } else {
            // Release: smooth exponential approach to target gain
            // Calculate release coefficient based on release time
            // Time constant tau = release_time, coefficient = 1 - exp(-1/(tau * sample_rate))
            let release_coeff = 1.0 - (-1.0 / (release_time * SAMPLE_RATE as f64)).exp();
            self.current_gain += (target_gain - self.current_gain) * release_coeff;
        }

        // Apply gain reduction
        input * self.current_gain
    }
}

impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> AudioSignal<SAMPLE_RATE>
    for Limiter<SAMPLE_RATE, S>
{
}
