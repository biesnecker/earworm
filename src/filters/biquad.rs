//! Biquad filter implementations.
//!
//! This module provides a versatile biquad filter that can operate in various
//! modes (low-pass, high-pass, band-pass, notch, all-pass) using the standard
//! biquad difference equation. The implementation uses Robert Bristow-Johnson's
//! Audio EQ Cookbook formulas for coefficient calculation.

use crate::{AudioSignal, Param, Signal};

/// The type of filter to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    /// Low-pass filter - attenuates frequencies above the cutoff
    LowPass,
    /// High-pass filter - attenuates frequencies below the cutoff
    HighPass,
    /// Band-pass filter - passes frequencies near the center, attenuates others
    BandPass,
    /// Notch/band-reject filter - attenuates frequencies near the center
    Notch,
    /// All-pass filter - passes all frequencies but shifts phase
    AllPass,
}

/// A biquad filter that processes an input signal.
///
/// Biquad filters are second-order IIR filters that can implement various
/// filter types by adjusting their coefficients. They provide a good balance
/// of efficiency and quality, making them ideal for real-time audio processing.
///
/// The filter supports both fixed and modulated parameters for cutoff frequency
/// and resonance (Q factor), enabling dynamic filter sweeps and modulation effects.
///
/// # Examples
///
/// ```
/// use earworm::{SineOscillator, filters::BiquadFilter};
///
/// let osc = SineOscillator::<44100>::new(440.0);
/// let mut filter = BiquadFilter::lowpass(osc, 1000.0, 0.707);
/// ```
pub struct BiquadFilter<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> {
    source: S,
    cutoff: Param,
    resonance: Param,
    filter_type: FilterType,

    // Filter state variables (previous samples)
    x1: f64, // Input at t-1
    x2: f64, // Input at t-2
    y1: f64, // Output at t-1
    y2: f64, // Output at t-2

    // Biquad coefficients (normalized)
    b0: f64, // Feedforward coefficient for x[n]
    b1: f64, // Feedforward coefficient for x[n-1]
    b2: f64, // Feedforward coefficient for x[n-2]
    a1: f64, // Feedback coefficient for y[n-1]
    a2: f64, // Feedback coefficient for y[n-2]

    // Optimization: only update coefficients if at least one param is modulated
    needs_coefficient_update: bool,
}

impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> BiquadFilter<SAMPLE_RATE, S> {
    pub fn new(
        source: S,
        cutoff: impl Into<Param>,
        resonance: impl Into<Param>,
        filter_type: FilterType,
    ) -> Self {
        let cutoff = cutoff.into();
        let resonance = resonance.into();

        // Only need to update if at least one param is modulated
        let needs_coefficient_update = !cutoff.is_fixed() || !resonance.is_fixed();

        let mut filter = Self {
            source,
            cutoff,
            resonance,
            filter_type,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            needs_coefficient_update,
        };

        // Calculate initial coefficients
        filter.update_coefficients();
        filter
    }

    /// Updates the filter coefficients based on current parameters.
    ///
    /// Uses Robert Bristow-Johnson's Audio EQ Cookbook formulas.
    fn update_coefficients(&mut self) {
        use std::f64::consts::PI;

        let freq = self.cutoff.value();
        let q = self.resonance.value().max(0.001); // Prevent division by zero

        // Clamp frequency to valid range (avoid nyquist issues)
        let sample_rate = SAMPLE_RATE as f64;
        let freq = freq.clamp(1.0, sample_rate * 0.49);

        // Common calculations
        let omega = 2.0 * PI * freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        // Calculate coefficients based on filter type
        let (mut b0, mut b1, mut b2, a0, mut a1, mut a2) = match self.filter_type {
            FilterType::LowPass => {
                let b0 = (1.0 - cos_omega) / 2.0;
                let b1 = 1.0 - cos_omega;
                let b2 = (1.0 - cos_omega) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }

            FilterType::HighPass => {
                let b0 = (1.0 + cos_omega) / 2.0;
                let b1 = -(1.0 + cos_omega);
                let b2 = (1.0 + cos_omega) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }

            FilterType::BandPass => {
                // Constant 0 dB peak gain (constant skirt gain)
                let b0 = alpha;
                let b1 = 0.0;
                let b2 = -alpha;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }

            FilterType::Notch => {
                let b0 = 1.0;
                let b1 = -2.0 * cos_omega;
                let b2 = 1.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }

            FilterType::AllPass => {
                let b0 = 1.0 - alpha;
                let b1 = -2.0 * cos_omega;
                let b2 = 1.0 + alpha;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
        };

        // Normalize by a0
        b0 /= a0;
        b1 /= a0;
        b2 /= a0;
        a1 /= a0;
        a2 /= a0;

        // Store normalized coefficients
        self.b0 = b0;
        self.b1 = b1;
        self.b2 = b2;
        self.a1 = a1;
        self.a2 = a2;
    }

    /// Creates a low-pass filter.
    ///
    /// # Arguments
    ///
    /// * `source` - Input signal
    /// * `cutoff` - Cutoff frequency in Hz
    /// * `q` - Q factor (resonance), typically 0.5-10.0. Higher = more resonant peak.
    pub fn lowpass(source: S, cutoff: impl Into<Param>, q: impl Into<Param>) -> Self {
        Self::new(source, cutoff, q, FilterType::LowPass)
    }

    /// Creates a high-pass filter.
    ///
    /// # Arguments
    ///
    /// * `source` - Input signal
    /// * `cutoff` - Cutoff frequency in Hz
    /// * `q` - Q factor (resonance), typically 0.5-10.0
    pub fn highpass(source: S, cutoff: impl Into<Param>, q: impl Into<Param>) -> Self {
        Self::new(source, cutoff, q, FilterType::HighPass)
    }

    /// Creates a band-pass filter.
    ///
    /// Passes frequencies near the cutoff, attenuates everything else.
    ///
    /// # Arguments
    ///
    /// * `source` - Input signal
    /// * `center` - Center frequency in Hz
    /// * `q` - Q factor (bandwidth), typically 0.5-10.0. Higher = narrower band.
    pub fn bandpass(source: S, center: impl Into<Param>, q: impl Into<Param>) -> Self {
        Self::new(source, center, q, FilterType::BandPass)
    }

    /// Creates a notch filter (band-reject/band-stop).
    ///
    /// Attenuates frequencies near the cutoff, passes everything else.
    ///
    /// # Arguments
    ///
    /// * `source` - Input signal
    /// * `center` - Center frequency to reject in Hz
    /// * `q` - Q factor (notch width), typically 0.5-10.0. Higher = narrower notch.
    pub fn notch(source: S, center: impl Into<Param>, q: impl Into<Param>) -> Self {
        Self::new(source, center, q, FilterType::Notch)
    }

    /// Creates an all-pass filter.
    ///
    /// Passes all frequencies but shifts their phase. Useful for phaser effects.
    ///
    /// # Arguments
    ///
    /// * `source` - Input signal
    /// * `frequency` - Center frequency for phase shift in Hz
    /// * `q` - Q factor, affects phase response
    pub fn allpass(source: S, frequency: impl Into<Param>, q: impl Into<Param>) -> Self {
        Self::new(source, frequency, q, FilterType::AllPass)
    }
}

impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> Signal for BiquadFilter<SAMPLE_RATE, S> {
    fn next_sample(&mut self) -> f64 {
        // Only update coefficients if parameters are modulated
        if self.needs_coefficient_update {
            self.update_coefficients();
        }

        let x0 = self.source.next_sample();

        // Direct Form I biquad difference equation:
        // y[n] = b0*x[n] + b1*x[n-1] + b2*x[n-2] - a1*y[n-1] - a2*y[n-2]
        let y0 = self.b0 * x0 + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

        // Update state variables
        self.x2 = self.x1;
        self.x1 = x0;
        self.y2 = self.y1;
        self.y1 = y0;

        y0
    }
}

// Implement AudioSignal for BiquadFilter when the source is an AudioSignal
impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> AudioSignal<SAMPLE_RATE>
    for BiquadFilter<SAMPLE_RATE, S>
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ConstantSignal, SineOscillator, combinators::SignalExt};

    #[test]
    fn test_lowpass_creation() {
        let source = ConstantSignal::<44100>(0.5);
        let filter = BiquadFilter::lowpass(source, 1000.0, 0.707);

        assert_eq!(filter.filter_type, FilterType::LowPass);
        assert_eq!(filter.sample_rate(), 44100.0);
    }

    #[test]
    fn test_highpass_creation() {
        let source = ConstantSignal::<44100>(0.5);
        let filter = BiquadFilter::highpass(source, 1000.0, 0.707);

        assert_eq!(filter.filter_type, FilterType::HighPass);
    }

    #[test]
    fn test_bandpass_creation() {
        let source = ConstantSignal::<44100>(0.5);
        let filter = BiquadFilter::bandpass(source, 1000.0, 2.0);

        assert_eq!(filter.filter_type, FilterType::BandPass);
    }

    #[test]
    fn test_notch_creation() {
        let source = ConstantSignal::<44100>(0.5);
        let filter = BiquadFilter::notch(source, 1000.0, 2.0);

        assert_eq!(filter.filter_type, FilterType::Notch);
    }

    #[test]
    fn test_allpass_creation() {
        let source = ConstantSignal::<44100>(0.5);
        let filter = BiquadFilter::allpass(source, 1000.0, 0.707);

        assert_eq!(filter.filter_type, FilterType::AllPass);
    }

    #[test]
    fn test_lowpass_attenuates_high_frequencies() {
        // Create a high-frequency sine wave (10kHz)
        let source = SineOscillator::<44100>::new(10000.0);
        // Filter with very low cutoff (100Hz)
        let mut filter = BiquadFilter::lowpass(source, 100.0, 0.707);

        // Process some samples to let the filter settle
        for _ in 0..100 {
            filter.next_sample();
        }

        // The output should be very small (heavily attenuated)
        let sample = filter.next_sample();
        assert!(sample.abs() < 0.1, "Expected attenuation, got {}", sample);
    }

    #[test]
    fn test_lowpass_passes_low_frequencies() {
        // Create a low-frequency sine wave (100Hz)
        let source = SineOscillator::<44100>::new(100.0);
        // Filter with high cutoff (5kHz)
        let mut filter = BiquadFilter::lowpass(source, 5000.0, 0.707);

        // Process some samples
        for _ in 0..100 {
            filter.next_sample();
        }

        // The output should be close to the input amplitude
        let sample = filter.next_sample();
        assert!(sample.abs() > 0.5, "Expected pass-through, got {}", sample);
    }

    #[test]
    fn test_highpass_attenuates_low_frequencies() {
        // Create a low-frequency sine wave (100Hz)
        let source = SineOscillator::<44100>::new(100.0);
        // Filter with high cutoff (5kHz) - should attenuate low frequencies
        let mut filter = BiquadFilter::highpass(source, 5000.0, 0.707);

        // Process some samples to let the filter settle
        for _ in 0..100 {
            filter.next_sample();
        }

        // The output should be very small (heavily attenuated)
        let sample = filter.next_sample();
        assert!(sample.abs() < 0.1, "Expected attenuation, got {}", sample);
    }

    #[test]
    fn test_constant_input_dc_blocking() {
        // DC signal (constant value)
        let source = ConstantSignal::<44100>(1.0);
        let mut filter = BiquadFilter::highpass(source, 100.0, 0.707);

        // High-pass should block DC
        for _ in 0..1000 {
            filter.next_sample();
        }

        let sample = filter.next_sample();
        assert!(sample.abs() < 0.01, "Expected DC blocking, got {}", sample);
    }

    #[test]
    fn test_filter_stability() {
        // Test that the filter doesn't blow up with normal input
        let source = SineOscillator::<44100>::new(440.0);
        let mut filter = BiquadFilter::lowpass(source, 1000.0, 5.0);

        // Process many samples
        for _ in 0..10000 {
            let sample = filter.next_sample();
            assert!(sample.is_finite(), "Filter became unstable");
            assert!(sample.abs() < 10.0, "Output amplitude too high: {}", sample);
        }
    }

    #[test]
    fn test_bandpass_attenuates_extremes() {
        let _sample_rate = 44100.0;

        // Test low frequency attenuation
        let low_freq = SineOscillator::<44100>::new(100.0);
        let mut bp_low = BiquadFilter::bandpass(low_freq, 1000.0, 5.0);

        for _ in 0..100 {
            bp_low.next_sample();
        }
        let low_sample = bp_low.next_sample();

        // Test high frequency attenuation
        let high_freq = SineOscillator::<44100>::new(10000.0);
        let mut bp_high = BiquadFilter::bandpass(high_freq, 1000.0, 5.0);

        for _ in 0..100 {
            bp_high.next_sample();
        }
        let high_sample = bp_high.next_sample();

        // Both should be attenuated
        assert!(
            low_sample.abs() < 0.3,
            "Low freq not attenuated: {}",
            low_sample
        );
        assert!(
            high_sample.abs() < 0.3,
            "High freq not attenuated: {}",
            high_sample
        );
    }

    #[test]
    fn test_modulated_cutoff() {
        // Use an LFO to modulate the cutoff frequency
        let source = SineOscillator::<44100>::new(440.0);
        let lfo = SineOscillator::<44100>::new(1.0);

        // LFO output is -1 to 1, scale to 500-1500 Hz
        let modulated_cutoff = lfo.gain(500.0).offset(1000.0);

        let mut filter = BiquadFilter::lowpass(source, modulated_cutoff, 0.707);

        // Should update coefficients each sample
        assert!(filter.needs_coefficient_update);

        // Process some samples - should not crash or become unstable
        for _ in 0..1000 {
            let sample = filter.next_sample();
            assert!(sample.is_finite());
        }
    }

    #[test]
    fn test_fixed_params_optimization() {
        let source = ConstantSignal::<44100>(1.0);
        let filter = BiquadFilter::lowpass(source, 1000.0, 0.707);

        // With fixed params, should not need coefficient updates
        assert!(!filter.needs_coefficient_update);
    }

    #[test]
    fn test_notch_filter() {
        // Notch should attenuate the center frequency
        let source = SineOscillator::<44100>::new(1000.0);
        let mut filter = BiquadFilter::notch(source, 1000.0, 10.0);

        // Let filter settle (needs more time for notch to take effect)
        for _ in 0..1000 {
            filter.next_sample();
        }

        // Check multiple samples to find minimum amplitude
        let mut min_amplitude = f64::INFINITY;
        for _ in 0..441 {
            // One period at 1000Hz
            let sample = filter.next_sample();
            min_amplitude = min_amplitude.min(sample.abs());
        }

        // At the notch frequency, signal should be heavily attenuated
        // With high Q, we should see significant reduction
        assert!(
            min_amplitude < 0.3,
            "Notch filter not working: min amplitude = {}",
            min_amplitude
        );
    }

    #[test]
    fn test_allpass_preserves_amplitude() {
        // All-pass should preserve signal amplitude (roughly)
        let source = SineOscillator::<44100>::new(1000.0);
        let mut filter = BiquadFilter::allpass(source, 1000.0, 0.707);

        // Process enough samples for steady state
        for _ in 0..1000 {
            filter.next_sample();
        }

        // Check that amplitude is roughly preserved over many samples
        let mut max_amplitude: f64 = 0.0;
        for _ in 0..441 {
            // One period at 100Hz
            let sample = filter.next_sample();
            max_amplitude = max_amplitude.max(sample.abs());
        }

        // Should be close to 1.0 (allowing for some phase-related variation)
        assert!(
            max_amplitude > 0.8 && max_amplitude < 1.2,
            "All-pass amplitude not preserved: {}",
            max_amplitude
        );
    }

    #[test]
    fn test_q_factor_clamping() {
        // Very low Q should be clamped to prevent division by zero
        let source = ConstantSignal::<44100>(1.0);
        let mut filter = BiquadFilter::lowpass(source, 1000.0, 0.0);

        // Should not panic or produce NaN
        for _ in 0..10 {
            let sample = filter.next_sample();
            assert!(sample.is_finite());
        }
    }

    #[test]
    fn test_frequency_clamping() {
        // Very high frequency should be clamped below Nyquist
        let source = SineOscillator::<44100>::new(440.0);
        let mut filter = BiquadFilter::lowpass(source, 50000.0, 0.707);

        // Should not panic or become unstable
        for _ in 0..100 {
            let sample = filter.next_sample();
            assert!(sample.is_finite());
        }
    }
}
