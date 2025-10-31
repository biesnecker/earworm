//! ADSR (Attack, Decay, Sustain, Release) envelope generator.

use super::envelope::Envelope;
use crate::synthesis::envelopes::Curve;

/// State of the ADSR envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EnvelopeState {
    /// Envelope is not active
    Idle,
    /// Ramping from 0 to peak level
    Attack,
    /// Ramping from peak to sustain level
    Decay,
    /// Holding at sustain level
    Sustain,
    /// Ramping from current level to 0
    Release,
}

/// ADSR (Attack, Decay, Sustain, Release) envelope generator.
///
/// Generates a control signal that follows the classic ADSR envelope shape:
/// - **Attack**: ramps from 0 to peak level (1.0)
/// - **Decay**: ramps from peak to sustain level
/// - **Sustain**: holds at sustain level until note off
/// - **Release**: ramps from current level to 0
///
/// # Examples
///
/// ```
/// use earworm::{ADSR, Curve};
/// use earworm::music::envelope::Envelope;
///
/// // Create an ADSR with 0.1s attack, 0.2s decay, 0.7 sustain level, 0.3s release
/// let mut env = ADSR::new(0.1, 0.2, 0.7, 0.3, 44100.0)
///     .with_attack_curve(Curve::Exponential(2.0))
///     .with_release_curve(Curve::Exponential(3.0));
///
/// // Trigger the envelope with velocity
/// env.trigger(0.8);
///
/// // Generate samples during attack/decay/sustain
/// for _ in 0..1000 {
///     let level = env.next_sample();
///     // Use level to control amplitude, filter cutoff, etc.
/// }
///
/// // Release the envelope
/// env.release();
///
/// // Generate samples during release
/// while env.is_active() {
///     let level = env.next_sample();
/// }
/// ```
#[derive(Clone)]
pub struct ADSR {
    state: EnvelopeState,
    phase_position: f64,      // samples elapsed in current phase
    current_level: f64,       // current output level
    release_start_level: f64, // level when release was triggered

    // Time parameters (in seconds)
    attack_time: f64,
    decay_time: f64,
    sustain_level: f64, // 0.0 to 1.0
    release_time: f64,

    // Curves for each phase
    attack_curve: Curve,
    decay_curve: Curve,
    release_curve: Curve,

    sample_rate: f64,
}

impl ADSR {
    /// Creates a new ADSR envelope with linear curves.
    ///
    /// # Arguments
    ///
    /// * `attack_time` - Attack time in seconds (0 or positive)
    /// * `decay_time` - Decay time in seconds (0 or positive)
    /// * `sustain_level` - Sustain level (0.0 to 1.0, will be clamped)
    /// * `release_time` - Release time in seconds (0 or positive)
    /// * `sample_rate` - Sample rate in Hz
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::ADSR;
    ///
    /// // Classic envelope: 10ms attack, 50ms decay, 70% sustain, 100ms release
    /// let env = ADSR::new(0.01, 0.05, 0.7, 0.1, 44100.0);
    /// ```
    pub fn new(
        attack_time: f64,
        decay_time: f64,
        sustain_level: f64,
        release_time: f64,
        sample_rate: f64,
    ) -> Self {
        Self {
            state: EnvelopeState::Idle,
            phase_position: 0.0,
            current_level: 0.0,
            release_start_level: 0.0,
            attack_time: attack_time.max(0.0),
            decay_time: decay_time.max(0.0),
            sustain_level: sustain_level.clamp(0.0, 1.0),
            release_time: release_time.max(0.0),
            attack_curve: Curve::Linear,
            decay_curve: Curve::Linear,
            release_curve: Curve::Linear,
            sample_rate,
        }
    }

    /// Sets the curve for the attack phase.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{ADSR, Curve};
    ///
    /// let env = ADSR::new(0.1, 0.1, 0.7, 0.1, 44100.0)
    ///     .with_attack_curve(Curve::Exponential(2.0));
    /// ```
    pub fn with_attack_curve(mut self, curve: Curve) -> Self {
        self.attack_curve = curve;
        self
    }

    /// Sets the curve for the decay phase.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{ADSR, Curve};
    ///
    /// let env = ADSR::new(0.1, 0.1, 0.7, 0.1, 44100.0)
    ///     .with_decay_curve(Curve::Exponential(2.0));
    /// ```
    pub fn with_decay_curve(mut self, curve: Curve) -> Self {
        self.decay_curve = curve;
        self
    }

    /// Sets the curve for the release phase.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{ADSR, Curve};
    ///
    /// let env = ADSR::new(0.1, 0.1, 0.7, 0.1, 44100.0)
    ///     .with_release_curve(Curve::Exponential(3.0));
    /// ```
    pub fn with_release_curve(mut self, curve: Curve) -> Self {
        self.release_curve = curve;
        self
    }

    /// Resets the envelope to idle state.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::ADSR;
    /// use earworm::music::envelope::Envelope;
    ///
    /// let mut env = ADSR::new(0.1, 0.1, 0.7, 0.1, 44100.0);
    /// env.trigger(0.8);
    /// env.reset();
    /// assert!(!env.is_active());
    /// ```
    pub fn reset(&mut self) {
        self.state = EnvelopeState::Idle;
        self.phase_position = 0.0;
        self.current_level = 0.0;
        self.release_start_level = 0.0;
    }

    /// Gets the current envelope state (for debugging/testing).
    #[cfg(test)]
    fn state(&self) -> EnvelopeState {
        self.state
    }
}

impl Envelope for ADSR {
    fn trigger(&mut self, _velocity: f64) {
        // For now, velocity is ignored. Future enhancement: scale peak level by velocity
        self.state = EnvelopeState::Attack;
        self.phase_position = 0.0;
    }

    fn release(&mut self) {
        if !matches!(self.state, EnvelopeState::Idle) {
            self.state = EnvelopeState::Release;
            self.phase_position = 0.0;
            self.release_start_level = self.current_level;
        }
    }

    fn is_active(&self) -> bool {
        !matches!(self.state, EnvelopeState::Idle)
    }

    fn next_sample(&mut self) -> f64 {
        match self.state {
            EnvelopeState::Idle => 0.0,

            EnvelopeState::Attack => {
                if self.attack_time <= 0.0 {
                    // Skip attack if time is zero
                    self.state = EnvelopeState::Decay;
                    self.phase_position = 0.0;
                    self.current_level = 1.0;
                    return 1.0;
                }

                let progress = self.phase_position / (self.attack_time * self.sample_rate);

                if progress >= 1.0 {
                    // Attack complete, move to decay
                    self.state = EnvelopeState::Decay;
                    self.phase_position = 0.0;
                    self.current_level = 1.0;
                    1.0
                } else {
                    self.phase_position += 1.0;
                    self.current_level = self.attack_curve.apply(progress);
                    self.current_level
                }
            }

            EnvelopeState::Decay => {
                if self.decay_time <= 0.0 {
                    // Skip decay if time is zero
                    self.state = EnvelopeState::Sustain;
                    self.current_level = self.sustain_level;
                    return self.sustain_level;
                }

                let progress = self.phase_position / (self.decay_time * self.sample_rate);

                if progress >= 1.0 {
                    // Decay complete, move to sustain
                    self.state = EnvelopeState::Sustain;
                    self.current_level = self.sustain_level;
                    self.sustain_level
                } else {
                    self.phase_position += 1.0;
                    let curved = self.decay_curve.apply(progress);
                    self.current_level = 1.0 - curved * (1.0 - self.sustain_level);
                    self.current_level
                }
            }

            EnvelopeState::Sustain => {
                self.current_level = self.sustain_level;
                self.sustain_level
            }

            EnvelopeState::Release => {
                if self.release_time <= 0.0 {
                    // Skip release if time is zero
                    self.state = EnvelopeState::Idle;
                    self.current_level = 0.0;
                    return 0.0;
                }

                let release_start = self.release_start_level;
                let progress = self.phase_position / (self.release_time * self.sample_rate);

                if progress >= 1.0 {
                    // Release complete, go idle
                    self.state = EnvelopeState::Idle;
                    self.current_level = 0.0;
                    0.0
                } else {
                    self.phase_position += 1.0;
                    let curved = self.release_curve.apply(progress);
                    self.current_level = release_start * (1.0 - curved);
                    self.current_level
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f64 = 100.0;
    const EPSILON: f64 = 1e-6;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn test_creation() {
        let env = ADSR::new(0.1, 0.2, 0.7, 0.3, SAMPLE_RATE);
        assert!(!env.is_active());
    }

    #[test]
    fn test_note_on_activates() {
        let mut env = ADSR::new(0.1, 0.2, 0.7, 0.3, SAMPLE_RATE);
        env.trigger(1.0);
        assert!(env.is_active());
        assert_eq!(env.state(), EnvelopeState::Attack);
    }

    #[test]
    fn test_idle_outputs_zero() {
        let mut env = ADSR::new(0.1, 0.2, 0.7, 0.3, SAMPLE_RATE);
        assert_eq!(env.next_sample(), 0.0);
        assert_eq!(env.next_sample(), 0.0);
    }

    #[test]
    fn test_attack_phase_linear() {
        let mut env = ADSR::new(1.0, 0.0, 1.0, 0.0, SAMPLE_RATE);
        env.trigger(1.0);

        // First sample should be near 0
        let s1 = env.next_sample();
        assert!(s1 < 0.02);

        // Middle of attack should be around 0.5
        for _ in 0..49 {
            env.next_sample();
        }
        let s_mid = env.next_sample();
        assert!(approx_eq(s_mid, 0.5));

        // End of attack should reach 1.0 and transition to decay
        for _ in 0..49 {
            env.next_sample();
        }
        let s_end = env.next_sample();
        assert!(approx_eq(s_end, 1.0));
        assert_eq!(env.state(), EnvelopeState::Decay);
    }

    #[test]
    fn test_decay_phase_linear() {
        let mut env = ADSR::new(0.0, 1.0, 0.5, 0.0, SAMPLE_RATE);
        env.trigger(1.0);

        // Skip attack (instant) - moves to decay
        let first = env.next_sample();
        assert_eq!(first, 1.0);
        assert_eq!(env.state(), EnvelopeState::Decay);

        // Generate decay samples
        // Decay is 1.0 seconds = 100 samples at 100Hz
        // We need to generate samples until we reach sustain
        let mut sample_count = 0;
        while env.state() == EnvelopeState::Decay && sample_count < 200 {
            env.next_sample();
            sample_count += 1;
        }

        // Should have transitioned to sustain after ~100 samples
        assert_eq!(env.state(), EnvelopeState::Sustain);
        assert!(sample_count > 90 && sample_count < 110);

        // Current level should be at sustain
        let s_sustain = env.next_sample();
        assert!(approx_eq(s_sustain, 0.5));
    }

    #[test]
    fn test_sustain_phase() {
        let mut env = ADSR::new(0.0, 0.0, 0.6, 0.0, SAMPLE_RATE);
        env.trigger(1.0);

        // Skip to sustain
        env.next_sample(); // attack
        env.next_sample(); // decay

        assert_eq!(env.state(), EnvelopeState::Sustain);

        // Sustain should hold constant
        for _ in 0..100 {
            let level = env.next_sample();
            assert!(approx_eq(level, 0.6));
        }
    }

    #[test]
    fn test_release_phase_linear() {
        let mut env = ADSR::new(0.0, 0.0, 0.8, 1.0, SAMPLE_RATE);
        env.trigger(1.0);

        // Get to sustain
        env.next_sample();
        env.next_sample();
        assert_eq!(env.state(), EnvelopeState::Sustain);

        // Trigger release
        env.release();
        assert_eq!(env.state(), EnvelopeState::Release);

        // First release sample should be near sustain level
        let s1 = env.next_sample();
        assert!(approx_eq(s1, 0.8));

        // Middle of release should be around 0.4 (halfway from 0.8 to 0)
        for _ in 0..49 {
            env.next_sample();
        }
        let s_mid = env.next_sample();
        assert!(approx_eq(s_mid, 0.4));

        // End of release should reach 0 and go idle
        for _ in 0..49 {
            env.next_sample();
        }
        let s_end = env.next_sample();
        assert!(approx_eq(s_end, 0.0));
        assert_eq!(env.state(), EnvelopeState::Idle);
        assert!(!env.is_active());
    }

    #[test]
    fn test_note_off_during_attack() {
        let mut env = ADSR::new(1.0, 0.1, 0.7, 0.5, SAMPLE_RATE);
        env.trigger(1.0);

        // Generate a few attack samples
        for _ in 0..10 {
            env.next_sample();
        }
        assert_eq!(env.state(), EnvelopeState::Attack);
        let level_before_release = env.current_level;

        // Release during attack
        env.release();
        assert_eq!(env.state(), EnvelopeState::Release);

        // Should release from current level
        let release_start = env.next_sample();
        assert!(approx_eq(release_start, level_before_release));
    }

    #[test]
    fn test_note_off_during_decay() {
        let mut env = ADSR::new(0.0, 1.0, 0.5, 0.5, SAMPLE_RATE);
        env.trigger(1.0);
        env.next_sample(); // Skip attack

        // Generate a few decay samples
        for _ in 0..10 {
            env.next_sample();
        }
        assert_eq!(env.state(), EnvelopeState::Decay);
        let level_before_release = env.current_level;

        // Release during decay
        env.release();
        assert_eq!(env.state(), EnvelopeState::Release);

        // Should release from current level
        let release_start = env.next_sample();
        assert!(approx_eq(release_start, level_before_release));
    }

    #[test]
    fn test_reset() {
        let mut env = ADSR::new(0.1, 0.1, 0.7, 0.1, SAMPLE_RATE);
        env.trigger(1.0);

        // Generate some samples
        for _ in 0..50 {
            env.next_sample();
        }

        env.reset();
        assert!(!env.is_active());
        assert_eq!(env.next_sample(), 0.0);
    }

    #[test]
    fn test_retrigger() {
        let mut env = ADSR::new(0.5, 0.1, 0.7, 0.1, SAMPLE_RATE);
        env.trigger(1.0);

        // Generate samples until partway through attack
        for _ in 0..25 {
            env.next_sample();
        }
        assert_eq!(env.state(), EnvelopeState::Attack);

        // Retrigger
        env.trigger(1.0);
        assert_eq!(env.state(), EnvelopeState::Attack);
        assert_eq!(env.phase_position, 0.0);

        // First sample should be near 0 again
        let s = env.next_sample();
        assert!(s < 0.02);
    }

    #[test]
    fn test_exponential_attack_curve() {
        let mut env =
            ADSR::new(1.0, 0.0, 1.0, 0.0, SAMPLE_RATE).with_attack_curve(Curve::Exponential(2.0));
        env.trigger(1.0);

        // At 50% progress, exponential(2.0) should give 0.25
        for _ in 0..50 {
            env.next_sample();
        }
        let level = env.next_sample();
        assert!(approx_eq(level, 0.25));
    }

    #[test]
    fn test_exponential_release_curve() {
        let mut env =
            ADSR::new(0.0, 0.0, 1.0, 1.0, SAMPLE_RATE).with_release_curve(Curve::Exponential(2.0));
        env.trigger(1.0);
        env.next_sample();
        env.next_sample();

        env.release();

        // At 50% progress through release, should be at 1.0 * (1 - 0.25) = 0.75
        for _ in 0..50 {
            env.next_sample();
        }
        let level = env.next_sample();
        assert!(approx_eq(level, 0.75));
    }

    #[test]
    fn test_sustain_level_clamping() {
        let env1 = ADSR::new(0.1, 0.1, -0.5, 0.1, SAMPLE_RATE);
        assert_eq!(env1.sustain_level, 0.0);

        let env2 = ADSR::new(0.1, 0.1, 1.5, 0.1, SAMPLE_RATE);
        assert_eq!(env2.sustain_level, 1.0);
    }

    #[test]
    fn test_zero_attack_time() {
        let mut env = ADSR::new(0.0, 0.1, 0.7, 0.1, SAMPLE_RATE);
        env.trigger(1.0);

        // First sample should immediately jump to 1.0 and move to decay
        let s = env.next_sample();
        assert_eq!(s, 1.0);
        assert_eq!(env.state(), EnvelopeState::Decay);
    }

    #[test]
    fn test_zero_decay_time() {
        let mut env = ADSR::new(0.0, 0.0, 0.5, 0.1, SAMPLE_RATE);
        env.trigger(1.0);

        env.next_sample(); // Skip attack
        let s = env.next_sample(); // Should skip decay
        assert_eq!(s, 0.5);
        assert_eq!(env.state(), EnvelopeState::Sustain);
    }

    #[test]
    fn test_zero_release_time() {
        let mut env = ADSR::new(0.0, 0.0, 0.7, 0.0, SAMPLE_RATE);
        env.trigger(1.0);
        env.next_sample();
        env.next_sample();

        env.release();
        let s = env.next_sample();
        assert_eq!(s, 0.0);
        assert_eq!(env.state(), EnvelopeState::Idle);
    }

    #[test]
    fn test_full_envelope_cycle() {
        let mut env = ADSR::new(0.1, 0.1, 0.6, 0.1, SAMPLE_RATE);
        env.trigger(1.0);

        // Attack: continues until progress >= 1.0
        // At 100Hz, 0.1s = 10 samples. The 11th sample triggers transition.
        for _ in 0..11 {
            let level = env.next_sample();
            assert!((0.0..=1.0).contains(&level));
        }
        assert_eq!(env.state(), EnvelopeState::Decay);

        // Decay: 11 samples to complete
        for _ in 0..11 {
            let level = env.next_sample();
            assert!((0.6..=1.0).contains(&level));
        }
        assert_eq!(env.state(), EnvelopeState::Sustain);

        // Sustain: arbitrary duration
        for _ in 0..20 {
            let level = env.next_sample();
            assert!(approx_eq(level, 0.6));
        }

        env.release();

        // Release: 11 samples to complete
        for _ in 0..11 {
            let level = env.next_sample();
            assert!((0.0..=0.6).contains(&level));
        }

        // Should be idle now
        assert!(!env.is_active());
        assert_eq!(env.next_sample(), 0.0);
    }

    #[test]
    fn test_note_off_while_idle() {
        let mut env = ADSR::new(0.1, 0.1, 0.7, 0.1, SAMPLE_RATE);
        env.release(); // Should have no effect
        assert!(!env.is_active());
        assert_eq!(env.next_sample(), 0.0);
    }

    #[test]
    fn test_generate_buffer() {
        let mut env = ADSR::new(0.1, 0.1, 0.7, 0.1, SAMPLE_RATE);
        env.trigger(1.0);

        let mut buffer = vec![0.0; 50];
        for sample in buffer.iter_mut() {
            *sample = env.next_sample();
        }

        // Verify all samples are in valid range
        for sample in buffer {
            assert!((0.0..=1.0).contains(&sample));
        }
    }
}
