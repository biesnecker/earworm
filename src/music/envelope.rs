//! Envelope trait for musical performance.

/// Common envelope states.
///
/// This enum represents the typical states an envelope can be in during its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvelopeState {
    /// Envelope is not active
    Idle,
    /// Attack phase - ramping up to peak
    Attack,
    /// Decay phase - ramping down from peak to sustain
    Decay,
    /// Sustain phase - holding at sustain level
    Sustain,
    /// Release phase - ramping down to zero
    Release,
}

/// Trait for envelope generators with lifecycle control.
///
/// Envelopes control parameters over time in response to musical events (note on/off).
/// Unlike general signals, envelopes have a defined lifecycle and are meant to be
/// triggered and released in response to performance gestures.
///
/// # Examples
///
/// ```
/// use earworm::music::envelope::Envelope;
/// use earworm::ADSR;
///
/// let mut env = ADSR::new(0.1, 0.1, 0.7, 0.3, 44100.0);
///
/// // Trigger the envelope with velocity
/// env.trigger(0.8);
/// assert!(env.is_active());
///
/// // Generate samples
/// for _ in 0..1000 {
///     let level = env.next_sample();
///     // Use level to control amplitude, filter cutoff, etc.
/// }
///
/// // Release the envelope
/// env.release();
///
/// // Continue generating until envelope completes
/// while env.is_active() {
///     env.next_sample();
/// }
/// ```
pub trait Envelope {
    /// Triggers the envelope, starting the attack phase.
    ///
    /// # Arguments
    ///
    /// * `velocity` - Note velocity (typically 0.0 to 1.0), which can affect
    ///   the envelope's response. How velocity is applied depends on the
    ///   envelope implementation (e.g., scaling peak level, sustain level, etc.).
    fn trigger(&mut self, velocity: f64);

    /// Releases the envelope, starting the release phase.
    ///
    /// The envelope will begin ramping down to zero. Once the release phase
    /// completes, `is_active()` will return false.
    fn release(&mut self);

    /// Returns true if the envelope is currently active (not idle).
    ///
    /// An envelope is active from when it's triggered until the release phase completes.
    fn is_active(&self) -> bool;

    /// Generates the next envelope value.
    ///
    /// # Returns
    ///
    /// The current envelope level, typically in the range [0.0, 1.0]
    fn next_sample(&mut self) -> f64;

    /// Returns the current envelope level without advancing the state.
    ///
    /// This is useful for voice stealing strategies that need to compare
    /// envelope levels (e.g., stealing the quietest voice).
    ///
    /// # Returns
    ///
    /// The current envelope level, typically in the range [0.0, 1.0]
    fn level(&self) -> f64;

    /// Returns the current envelope state.
    ///
    /// This is useful for voice stealing strategies that prefer voices
    /// in certain states (e.g., preferring voices in release phase).
    ///
    /// # Returns
    ///
    /// The current `EnvelopeState`
    fn state(&self) -> EnvelopeState;

    /// Returns true if the envelope is in its final decay phase (releasing).
    ///
    /// This is useful for voice stealing - voices that are releasing can be
    /// stolen with lower priority. Different envelope types have different
    /// concepts of "releasing":
    /// - ADSR: Release state
    /// - AHD: Decay state
    /// - AR: Release state
    ///
    /// Default implementation checks for Release state, but envelope
    /// implementations can override this for their specific final phase.
    ///
    /// # Returns
    ///
    /// True if the envelope is in its final decay/release phase
    fn is_releasing(&self) -> bool {
        matches!(self.state(), EnvelopeState::Release)
    }
}
