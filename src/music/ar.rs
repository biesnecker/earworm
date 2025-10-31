//! AR (Attack, Release) envelope generator.

use super::envelope::{Envelope, EnvelopeState};
use crate::synthesis::envelopes::Curve;

/// AR (Attack, Release) envelope generator.
///
/// Generates a control signal that follows a simple AR envelope shape:
/// - **Attack**: ramps from 0 to peak level (1.0)
/// - **Release**: ramps from current level to 0
///
/// This is a simplified envelope useful for percussive sounds where you don't need
/// sustain or decay phases. The envelope immediately begins releasing when triggered,
/// or can be explicitly released.
///
/// # Examples
///
/// ```
/// use earworm::music::{AR, Envelope};
/// use earworm::Curve;
///
/// // Create an AR with 0.01s attack, 0.2s release
/// let mut env = AR::new(0.01, 0.2, 44100.0)
///     .with_attack_curve(Curve::Exponential(2.0));
///
/// // Trigger the envelope
/// env.trigger(0.8);
///
/// // Generate samples - will attack then automatically release
/// while env.is_active() {
///     let level = env.next_sample();
///     // Use level to control amplitude
/// }
/// ```
#[derive(Clone)]
pub struct AR {
    state: EnvelopeState,
    phase_position: f64,      // samples elapsed in current phase
    current_level: f64,       // current output level
    release_start_level: f64, // level when release was triggered

    // Time parameters (in seconds)
    attack_time: f64,
    release_time: f64,

    // Curves for each phase
    attack_curve: Curve,
    release_curve: Curve,

    sample_rate: f64,
}

impl AR {
    /// Creates a new AR envelope with linear curves.
    ///
    /// # Arguments
    ///
    /// * `attack_time` - Attack time in seconds (0 or positive)
    /// * `release_time` - Release time in seconds (0 or positive)
    /// * `sample_rate` - Sample rate in Hz
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::AR;
    ///
    /// // Percussive envelope: 5ms attack, 200ms release
    /// let env = AR::new(0.005, 0.2, 44100.0);
    /// ```
    pub fn new(attack_time: f64, release_time: f64, sample_rate: f64) -> Self {
        Self {
            state: EnvelopeState::Idle,
            phase_position: 0.0,
            current_level: 0.0,
            release_start_level: 0.0,
            attack_time: attack_time.max(0.0),
            release_time: release_time.max(0.0),
            attack_curve: Curve::Linear,
            release_curve: Curve::Linear,
            sample_rate,
        }
    }

    /// Sets the attack curve.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::AR;
    /// use earworm::Curve;
    ///
    /// let env = AR::new(0.01, 0.1, 44100.0)
    ///     .with_attack_curve(Curve::Exponential(2.0));
    /// ```
    pub fn with_attack_curve(mut self, curve: Curve) -> Self {
        self.attack_curve = curve;
        self
    }

    /// Sets the release curve.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::AR;
    /// use earworm::Curve;
    ///
    /// let env = AR::new(0.01, 0.1, 44100.0)
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
    /// use earworm::music::{AR, Envelope};
    ///
    /// let mut env = AR::new(0.01, 0.1, 44100.0);
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
}

impl Envelope for AR {
    fn trigger(&mut self, _velocity: f64) {
        self.state = EnvelopeState::Attack;
        self.phase_position = 0.0;
    }

    fn release(&mut self) {
        if !matches!(self.state, EnvelopeState::Idle | EnvelopeState::Release) {
            self.state = EnvelopeState::Release;
            self.phase_position = 0.0;
            self.release_start_level = self.current_level;
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

    fn next_sample(&mut self) -> f64 {
        match self.state {
            EnvelopeState::Idle => 0.0,

            EnvelopeState::Attack => {
                if self.attack_time <= 0.0 {
                    // Skip attack if time is zero, go straight to release
                    self.state = EnvelopeState::Release;
                    self.phase_position = 0.0;
                    self.release_start_level = 1.0;
                    self.current_level = 1.0;
                    return 1.0;
                }

                let progress = self.phase_position / (self.attack_time * self.sample_rate);

                if progress >= 1.0 {
                    // Attack complete, move to release
                    self.state = EnvelopeState::Release;
                    self.phase_position = 0.0;
                    self.release_start_level = 1.0;
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

            EnvelopeState::Release => {
                if self.release_time <= 0.0 {
                    // Instant release
                    self.state = EnvelopeState::Idle;
                    self.current_level = 0.0;
                    return 0.0;
                }

                let progress = self.phase_position / (self.release_time * self.sample_rate);

                if progress >= 1.0 {
                    // Release complete
                    self.state = EnvelopeState::Idle;
                    self.current_level = 0.0;
                    0.0
                } else {
                    // Still in release
                    let curve_value = 1.0 - self.release_curve.apply(progress);
                    let level = self.release_start_level * curve_value;
                    self.phase_position += 1.0;
                    self.current_level = level;
                    level
                }
            }

            // AR doesn't use Decay or Sustain, but we need to handle them for the enum
            EnvelopeState::Decay | EnvelopeState::Sustain => {
                // Shouldn't happen, but if it does, treat as release
                self.state = EnvelopeState::Release;
                self.phase_position = 0.0;
                self.release_start_level = self.current_level;
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
        let env = AR::new(0.1, 0.2, SAMPLE_RATE);
        assert!(!env.is_active());
        assert_eq!(env.level(), 0.0);
        assert_eq!(env.state(), EnvelopeState::Idle);
    }

    #[test]
    fn test_trigger_activates() {
        let mut env = AR::new(0.1, 0.2, SAMPLE_RATE);
        env.trigger(0.8);
        assert!(env.is_active());
        assert_eq!(env.state(), EnvelopeState::Attack);
    }

    #[test]
    fn test_attack_phase() {
        let mut env = AR::new(0.1, 0.2, SAMPLE_RATE);
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
    fn test_attack_to_release_transition() {
        let mut env = AR::new(0.01, 0.1, SAMPLE_RATE);
        env.trigger(0.8);

        // Run through attack
        let attack_samples = (0.01 * SAMPLE_RATE) as usize;
        for _ in 0..attack_samples {
            env.next_sample();
        }

        // Should transition to release
        env.next_sample();
        assert_eq!(env.state(), EnvelopeState::Release);
    }

    #[test]
    fn test_release_phase() {
        let mut env = AR::new(0.01, 0.1, SAMPLE_RATE);
        env.trigger(0.8);

        // Run through attack to get to release
        let attack_samples = (0.01 * SAMPLE_RATE) as usize;
        for _ in 0..=attack_samples {
            env.next_sample();
        }

        assert_eq!(env.state(), EnvelopeState::Release);

        // Release should decay
        let first = env.next_sample();
        for _ in 0..100 {
            env.next_sample();
        }
        let later = env.next_sample();
        assert!(later < first);
    }

    #[test]
    fn test_completes_to_idle() {
        let mut env = AR::new(0.001, 0.001, SAMPLE_RATE);
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
        let mut env = AR::new(0.0, 0.1, SAMPLE_RATE);
        env.trigger(0.8);
        let level = env.next_sample();
        assert_eq!(level, 1.0);
        assert_eq!(env.state(), EnvelopeState::Release);
    }

    #[test]
    fn test_zero_release_time() {
        let mut env = AR::new(0.01, 0.0, SAMPLE_RATE);
        env.trigger(0.8);

        // Run through attack
        let attack_samples = (0.01 * SAMPLE_RATE) as usize;
        for _ in 0..=attack_samples {
            env.next_sample();
        }

        // Should be in release state after attack
        assert_eq!(env.state(), EnvelopeState::Release);

        // One more sample should process the zero-length release
        env.next_sample();
        assert_eq!(env.state(), EnvelopeState::Idle);
        assert_eq!(env.level(), 0.0);
    }

    #[test]
    fn test_explicit_release() {
        let mut env = AR::new(0.1, 0.1, SAMPLE_RATE);
        env.trigger(0.8);

        // Trigger release during attack
        for _ in 0..1000 {
            env.next_sample();
        }
        assert_eq!(env.state(), EnvelopeState::Attack);

        env.release();
        assert_eq!(env.state(), EnvelopeState::Release);
    }

    #[test]
    fn test_reset() {
        let mut env = AR::new(0.1, 0.1, SAMPLE_RATE);
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
        let mut env = AR::new(0.1, 0.1, SAMPLE_RATE).with_attack_curve(Curve::Exponential(2.0));
        env.trigger(0.8);

        let samples: Vec<f64> = (0..100).map(|_| env.next_sample()).collect();

        // Exponential should have slower start
        assert!(samples[10] < samples[50] - samples[10]);
    }

    #[test]
    fn test_exponential_release_curve() {
        let mut env = AR::new(0.001, 0.1, SAMPLE_RATE).with_release_curve(Curve::Exponential(3.0));
        env.trigger(0.8);

        // Run through very short attack to get to release quickly
        for _ in 0..100 {
            env.next_sample();
        }

        // Should be in release now
        assert_eq!(env.state(), EnvelopeState::Release);

        // Collect samples during release
        let early = env.level();
        for _ in 0..1000 {
            env.next_sample();
        }
        let late = env.level();

        // Should decay during release
        assert!(
            late < early,
            "Release should decay: early={}, late={}",
            early,
            late
        );
    }
}
