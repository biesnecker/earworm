//! AHD (Attack, Hold, Decay) envelope generator.

use super::envelope::{Envelope, EnvelopeState};
use crate::synthesis::envelopes::Curve;

/// AHD (Attack, Hold, Decay) envelope generator.
///
/// Generates a control signal that follows an AHD envelope shape:
/// - **Attack**: ramps from 0 to peak level (1.0)
/// - **Hold**: sustains at peak level for a specified time
/// - **Decay**: ramps from peak to 0
///
/// This is useful for sounds that need a sustained peak before decaying,
/// like bell sounds or percussive hits with resonance. Unlike ADSR, there is
/// no sustain phase - the envelope always completes its cycle.
///
/// # Examples
///
/// ```
/// use earworm::music::{AHD, Envelope};
/// use earworm::Curve;
///
/// // Create an AHD with 0.01s attack, 0.05s hold, 0.3s decay
/// let mut env = AHD::new(0.01, 0.05, 0.3, 44100.0)
///     .with_decay_curve(Curve::Exponential(3.0));
///
/// // Trigger the envelope
/// env.trigger(0.8);
///
/// // Generate samples - will attack, hold, then decay to completion
/// while env.is_active() {
///     let level = env.next_sample();
///     // Use level to control amplitude
/// }
/// ```
#[derive(Clone)]
pub struct AHD {
    state: EnvelopeState,
    phase_position: f64, // samples elapsed in current phase
    current_level: f64,  // current output level

    // Time parameters (in seconds)
    attack_time: f64,
    hold_time: f64,
    decay_time: f64,

    // Curves for each phase
    attack_curve: Curve,
    decay_curve: Curve,

    sample_rate: f64,
}

impl AHD {
    /// Creates a new AHD envelope with linear curves.
    ///
    /// # Arguments
    ///
    /// * `attack_time` - Attack time in seconds (0 or positive)
    /// * `hold_time` - Hold time in seconds (0 or positive)
    /// * `decay_time` - Decay time in seconds (0 or positive)
    /// * `sample_rate` - Sample rate in Hz
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::AHD;
    ///
    /// // Bell-like envelope: 10ms attack, 50ms hold, 500ms decay
    /// let env = AHD::new(0.01, 0.05, 0.5, 44100.0);
    /// ```
    pub fn new(attack_time: f64, hold_time: f64, decay_time: f64, sample_rate: f64) -> Self {
        Self {
            state: EnvelopeState::Idle,
            phase_position: 0.0,
            current_level: 0.0,
            attack_time: attack_time.max(0.0),
            hold_time: hold_time.max(0.0),
            decay_time: decay_time.max(0.0),
            attack_curve: Curve::Linear,
            decay_curve: Curve::Linear,
            sample_rate,
        }
    }

    /// Sets the attack curve.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::AHD;
    /// use earworm::Curve;
    ///
    /// let env = AHD::new(0.01, 0.05, 0.3, 44100.0)
    ///     .with_attack_curve(Curve::Exponential(2.0));
    /// ```
    pub fn with_attack_curve(mut self, curve: Curve) -> Self {
        self.attack_curve = curve;
        self
    }

    /// Sets the decay curve.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::AHD;
    /// use earworm::Curve;
    ///
    /// let env = AHD::new(0.01, 0.05, 0.3, 44100.0)
    ///     .with_decay_curve(Curve::Exponential(3.0));
    /// ```
    pub fn with_decay_curve(mut self, curve: Curve) -> Self {
        self.decay_curve = curve;
        self
    }

    /// Resets the envelope to idle state.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::{AHD, Envelope};
    ///
    /// let mut env = AHD::new(0.01, 0.05, 0.3, 44100.0);
    /// env.trigger(0.8);
    /// env.reset();
    /// assert!(!env.is_active());
    /// ```
    pub fn reset(&mut self) {
        self.state = EnvelopeState::Idle;
        self.phase_position = 0.0;
        self.current_level = 0.0;
    }
}

impl Envelope for AHD {
    fn trigger(&mut self, _velocity: f64) {
        self.state = EnvelopeState::Attack;
        self.phase_position = 0.0;
    }

    fn release(&mut self) {
        // AHD doesn't support release - it always completes its cycle
        // But we can skip to decay phase if in attack or sustain
        if matches!(self.state, EnvelopeState::Attack | EnvelopeState::Sustain) {
            self.state = EnvelopeState::Decay;
            self.phase_position = 0.0;
        }
    }

    fn is_active(&self) -> bool {
        !matches!(self.state, EnvelopeState::Idle)
    }

    fn level(&self) -> f64 {
        self.current_level
    }

    fn state(&self) -> EnvelopeState {
        self.state
    }

    fn is_releasing(&self) -> bool {
        // For AHD, the Decay phase is the final "releasing" phase
        matches!(self.state, EnvelopeState::Decay)
    }

    fn next_sample(&mut self) -> f64 {
        match self.state {
            EnvelopeState::Idle => 0.0,

            EnvelopeState::Attack => {
                if self.attack_time <= 0.0 {
                    // Skip attack if time is zero, go to hold
                    self.state = EnvelopeState::Sustain;
                    self.phase_position = 0.0;
                    self.current_level = 1.0;
                    return 1.0;
                }

                let progress = self.phase_position / (self.attack_time * self.sample_rate);

                if progress >= 1.0 {
                    // Attack complete, move to hold (sustain)
                    self.state = EnvelopeState::Sustain;
                    self.phase_position = 0.0;
                    self.current_level = 1.0;
                    1.0
                } else {
                    // Still in attack
                    let level = self.attack_curve.apply(progress);
                    self.phase_position += 1.0;
                    self.current_level = level;
                    level
                }
            }

            EnvelopeState::Sustain => {
                // In AHD, sustain is the "hold" phase at peak level
                if self.hold_time <= 0.0 {
                    // Skip hold if time is zero, go to decay
                    self.state = EnvelopeState::Decay;
                    self.phase_position = 0.0;
                    return 1.0;
                }

                let progress = self.phase_position / (self.hold_time * self.sample_rate);

                if progress >= 1.0 {
                    // Hold complete, move to decay
                    self.state = EnvelopeState::Decay;
                    self.phase_position = 0.0;
                    self.current_level = 1.0;
                    1.0
                } else {
                    // Still holding at peak
                    self.phase_position += 1.0;
                    self.current_level = 1.0;
                    1.0
                }
            }

            EnvelopeState::Decay => {
                if self.decay_time <= 0.0 {
                    // Instant decay
                    self.state = EnvelopeState::Idle;
                    self.current_level = 0.0;
                    return 0.0;
                }

                let progress = self.phase_position / (self.decay_time * self.sample_rate);

                if progress >= 1.0 {
                    // Decay complete
                    self.state = EnvelopeState::Idle;
                    self.current_level = 0.0;
                    0.0
                } else {
                    // Still in decay
                    let curve_value = 1.0 - self.decay_curve.apply(progress);
                    let level = curve_value;
                    self.phase_position += 1.0;
                    self.current_level = level;
                    level
                }
            }

            // AHD doesn't use Release
            EnvelopeState::Release => {
                // Shouldn't happen, but treat as decay
                self.state = EnvelopeState::Decay;
                self.phase_position = 0.0;
                self.current_level
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f64 = 44100.0;

    #[test]
    fn test_creation() {
        let env = AHD::new(0.1, 0.05, 0.3, SAMPLE_RATE);
        assert!(!env.is_active());
        assert_eq!(env.level(), 0.0);
        assert_eq!(env.state(), EnvelopeState::Idle);
    }

    #[test]
    fn test_trigger_activates() {
        let mut env = AHD::new(0.1, 0.05, 0.3, SAMPLE_RATE);
        env.trigger(0.8);
        assert!(env.is_active());
        assert_eq!(env.state(), EnvelopeState::Attack);
    }

    #[test]
    fn test_attack_phase() {
        let mut env = AHD::new(0.1, 0.05, 0.3, SAMPLE_RATE);
        env.trigger(0.8);

        // First sample should be very low
        let first = env.next_sample();
        assert!(first < 0.1);

        // Halfway through attack should be around 0.5
        let attack_samples = (0.1 * SAMPLE_RATE) as usize;
        for _ in 0..(attack_samples / 2 - 1) {
            env.next_sample();
        }
        let mid = env.next_sample();
        assert!(mid > 0.4 && mid < 0.6);
    }

    #[test]
    fn test_attack_to_hold_transition() {
        let mut env = AHD::new(0.01, 0.05, 0.1, SAMPLE_RATE);
        env.trigger(0.8);

        // Run through attack
        let attack_samples = (0.01 * SAMPLE_RATE) as usize;
        for _ in 0..=attack_samples {
            env.next_sample();
        }

        // Should transition to sustain (hold)
        assert_eq!(env.state(), EnvelopeState::Sustain);
        assert_eq!(env.level(), 1.0);
    }

    #[test]
    fn test_hold_phase() {
        let mut env = AHD::new(0.01, 0.05, 0.1, SAMPLE_RATE);
        env.trigger(0.8);

        // Run through attack
        let attack_samples = (0.01 * SAMPLE_RATE) as usize;
        for _ in 0..=attack_samples {
            env.next_sample();
        }

        // During hold, level should stay at 1.0
        for _ in 0..100 {
            let level = env.next_sample();
            assert_eq!(level, 1.0);
        }
    }

    #[test]
    fn test_hold_to_decay_transition() {
        let mut env = AHD::new(0.001, 0.001, 0.1, SAMPLE_RATE);
        env.trigger(0.8);

        // Run through attack and hold
        for _ in 0..200 {
            env.next_sample();
        }

        // Should be in decay
        assert_eq!(env.state(), EnvelopeState::Decay);
    }

    #[test]
    fn test_decay_phase() {
        let mut env = AHD::new(0.001, 0.001, 0.1, SAMPLE_RATE);
        env.trigger(0.8);

        // Run through attack and hold to get to decay
        for _ in 0..200 {
            env.next_sample();
        }

        assert_eq!(env.state(), EnvelopeState::Decay);

        // Decay should decrease
        let first = env.next_sample();
        for _ in 0..100 {
            env.next_sample();
        }
        let later = env.next_sample();
        assert!(later < first);
    }

    #[test]
    fn test_completes_to_idle() {
        let mut env = AHD::new(0.001, 0.001, 0.001, SAMPLE_RATE);
        env.trigger(0.8);

        // Run until idle
        for _ in 0..1000 {
            env.next_sample();
        }

        assert_eq!(env.state(), EnvelopeState::Idle);
        assert!(!env.is_active());
        assert_eq!(env.level(), 0.0);
    }

    #[test]
    fn test_zero_attack_time() {
        let mut env = AHD::new(0.0, 0.05, 0.1, SAMPLE_RATE);
        env.trigger(0.8);
        let level = env.next_sample();
        assert_eq!(level, 1.0);
        assert_eq!(env.state(), EnvelopeState::Sustain);
    }

    #[test]
    fn test_zero_hold_time() {
        let mut env = AHD::new(0.01, 0.0, 0.1, SAMPLE_RATE);
        env.trigger(0.8);

        // Run through attack
        let attack_samples = (0.01 * SAMPLE_RATE) as usize;
        for _ in 0..=attack_samples {
            env.next_sample();
        }

        // Should skip hold and go to decay
        env.next_sample();
        assert_eq!(env.state(), EnvelopeState::Decay);
    }

    #[test]
    fn test_zero_decay_time() {
        let mut env = AHD::new(0.001, 0.001, 0.0, SAMPLE_RATE);
        env.trigger(0.8);

        // Run through attack and hold
        for _ in 0..200 {
            env.next_sample();
        }

        // Should immediately go to idle when decay starts
        assert_eq!(env.state(), EnvelopeState::Idle);
        assert_eq!(env.level(), 0.0);
    }

    #[test]
    fn test_release_skips_to_decay() {
        let mut env = AHD::new(0.1, 0.1, 0.1, SAMPLE_RATE);
        env.trigger(0.8);

        // Release during attack
        for _ in 0..1000 {
            env.next_sample();
        }
        assert_eq!(env.state(), EnvelopeState::Attack);

        env.release();
        assert_eq!(env.state(), EnvelopeState::Decay);
    }

    #[test]
    fn test_reset() {
        let mut env = AHD::new(0.1, 0.05, 0.3, SAMPLE_RATE);
        env.trigger(0.8);
        for _ in 0..100 {
            env.next_sample();
        }
        env.reset();
        assert!(!env.is_active());
        assert_eq!(env.level(), 0.0);
        assert_eq!(env.state(), EnvelopeState::Idle);
    }

    #[test]
    fn test_exponential_attack_curve() {
        let mut env =
            AHD::new(0.1, 0.0, 0.0, SAMPLE_RATE).with_attack_curve(Curve::Exponential(2.0));
        env.trigger(0.8);

        let samples: Vec<f64> = (0..100).map(|_| env.next_sample()).collect();

        // Exponential should have slower start
        assert!(samples[10] < samples[50] - samples[10]);
    }

    #[test]
    fn test_exponential_decay_curve() {
        let mut env =
            AHD::new(0.001, 0.001, 0.1, SAMPLE_RATE).with_decay_curve(Curve::Exponential(3.0));
        env.trigger(0.8);

        // Run to decay phase
        for _ in 0..200 {
            env.next_sample();
        }

        assert_eq!(env.state(), EnvelopeState::Decay);

        // Check that decay happens
        let early = env.level();
        for _ in 0..1000 {
            env.next_sample();
        }
        let late = env.level();

        assert!(
            late < early,
            "Decay should happen: early={}, late={}",
            early,
            late
        );
    }
}
