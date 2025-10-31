//! Envelope trait for musical performance.

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
}
