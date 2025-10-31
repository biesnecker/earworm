//! Voice - a combination of a pitched signal and an envelope.

use super::{envelope::Envelope, frequency::Frequency};
use crate::{AudioSignal, Pitched, Signal};

/// A voice combines a pitched signal source with an envelope.
///
/// This is the basic building block for polyphonic synthesis, representing
/// a single note being played. The voice owns both the signal source (oscillator,
/// wavetable, etc.) and its amplitude envelope.
///
/// # Type Parameters
///
/// * `SAMPLE_RATE` - Sample rate in Hz
/// * `S` - Signal type (must be `AudioSignal` + `Pitched`)
/// * `E` - Envelope type (defaults to `ADSR`)
///
/// # Examples
///
/// ```
/// use earworm::{ADSR, SineOscillator, Signal};
/// use earworm::music::{Voice, envelope::Envelope};
///
/// const SAMPLE_RATE: u32 = 44100;
///
/// // Create a simple voice with sine oscillator and ADSR
/// let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
/// let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
/// let mut voice = Voice::new(osc, env);
///
/// // Play a note
/// voice.note_on(440.0, 0.8);
///
/// // Generate samples
/// for _ in 0..1000 {
///     let sample = voice.next_sample();
///     // Output sample...
/// }
///
/// // Release the note
/// voice.note_off();
///
/// // Continue generating until voice is done
/// while voice.is_active() {
///     voice.next_sample();
/// }
/// ```
pub struct Voice<const SAMPLE_RATE: u32, S, E>
where
    S: AudioSignal<SAMPLE_RATE> + Pitched,
    E: Envelope,
{
    signal: S,
    envelope: E,
}

impl<const SAMPLE_RATE: u32, S, E> Voice<SAMPLE_RATE, S, E>
where
    S: AudioSignal<SAMPLE_RATE> + Pitched,
    E: Envelope,
{
    /// Creates a new voice with the given signal source and envelope.
    ///
    /// # Arguments
    ///
    /// * `signal` - The pitched signal source (oscillator, wavetable, etc.)
    /// * `envelope` - The amplitude envelope
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{ADSR, SineOscillator};
    /// use earworm::music::Voice;
    ///
    /// const SAMPLE_RATE: u32 = 44100;
    ///
    /// let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
    /// let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
    /// let voice = Voice::new(osc, env);
    /// ```
    pub fn new(signal: S, envelope: E) -> Self {
        Self { signal, envelope }
    }

    /// Triggers a note with the given pitch and velocity.
    ///
    /// This sets the signal's frequency and triggers the envelope.
    ///
    /// # Arguments
    ///
    /// * `pitch` - The pitch to play (accepts Hz, MIDI note, or Note)
    /// * `velocity` - Note velocity (0.0 to 1.0), passed to the envelope
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{ADSR, SineOscillator};
    /// use earworm::music::Voice;
    ///
    /// const SAMPLE_RATE: u32 = 44100;
    ///
    /// let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
    /// let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
    /// let mut voice = Voice::new(osc, env);
    ///
    /// // Play A4 at 80% velocity using Hz
    /// voice.note_on(440.0, 0.8);
    ///
    /// // Or using MIDI note number
    /// voice.note_on(69u8, 0.8);
    /// ```
    pub fn note_on(&mut self, pitch: impl Into<Frequency>, velocity: f64) {
        let freq = pitch.into();
        self.signal.set_frequency(freq.as_f64());
        self.envelope.trigger(velocity);
    }

    /// Releases the note, starting the envelope's release phase.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{ADSR, SineOscillator};
    /// use earworm::music::{Voice, envelope::Envelope};
    ///
    /// const SAMPLE_RATE: u32 = 44100;
    ///
    /// let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
    /// let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
    /// let mut voice = Voice::new(osc, env);
    ///
    /// voice.note_on(440.0, 0.8);
    /// // ... generate some samples ...
    /// voice.note_off();
    /// ```
    pub fn note_off(&mut self) {
        self.envelope.release();
    }

    /// Returns true if the voice is currently active.
    ///
    /// A voice is active when its envelope is active (not in idle state).
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{ADSR, SineOscillator, Signal};
    /// use earworm::music::{Voice, envelope::Envelope};
    ///
    /// const SAMPLE_RATE: u32 = 44100;
    ///
    /// let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
    /// let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
    /// let mut voice = Voice::new(osc, env);
    ///
    /// assert!(!voice.is_active());
    ///
    /// voice.note_on(440.0, 0.8);
    /// assert!(voice.is_active());
    ///
    /// voice.note_off();
    /// // Still active during release
    /// assert!(voice.is_active());
    ///
    /// // Generate samples until release completes
    /// while voice.is_active() {
    ///     voice.next_sample();
    /// }
    /// assert!(!voice.is_active());
    /// ```
    pub fn is_active(&self) -> bool {
        self.envelope.is_active()
    }
}

impl<const SAMPLE_RATE: u32, S, E> Signal for Voice<SAMPLE_RATE, S, E>
where
    S: AudioSignal<SAMPLE_RATE> + Pitched,
    E: Envelope,
{
    fn next_sample(&mut self) -> f64 {
        let signal_sample = self.signal.next_sample();
        let envelope_sample = self.envelope.next_sample();
        signal_sample * envelope_sample
    }
}

impl<const SAMPLE_RATE: u32, S, E> AudioSignal<SAMPLE_RATE> for Voice<SAMPLE_RATE, S, E>
where
    S: AudioSignal<SAMPLE_RATE> + Pitched,
    E: Envelope,
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ADSR, SineOscillator};

    const SAMPLE_RATE: u32 = 44100;

    #[test]
    fn test_voice_creation() {
        let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
        let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
        let voice = Voice::new(osc, env);
        assert!(!voice.is_active());
    }

    #[test]
    fn test_voice_note_on_hz() {
        let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
        let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
        let mut voice = Voice::new(osc, env);

        voice.note_on(880.0, 0.8);
        assert!(voice.is_active());
        assert_eq!(voice.signal.frequency(), 880.0);
    }

    #[test]
    fn test_voice_note_on_midi() {
        let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
        let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
        let mut voice = Voice::new(osc, env);

        voice.note_on(69u8, 0.8); // A4 = 440 Hz
        assert!(voice.is_active());
        assert!((voice.signal.frequency() - 440.0).abs() < 0.01);
    }

    #[test]
    fn test_voice_lifecycle() {
        let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
        let env = ADSR::new(0.001, 0.001, 0.7, 0.001, SAMPLE_RATE as f64);
        let mut voice = Voice::new(osc, env);

        // Initially inactive
        assert!(!voice.is_active());

        // Trigger note
        voice.note_on(440.0, 0.8);
        assert!(voice.is_active());

        // Generate some samples
        for _ in 0..100 {
            let sample = voice.next_sample();
            assert!(sample.abs() <= 1.0);
        }

        // Release note
        voice.note_off();
        assert!(voice.is_active()); // Still active during release

        // Generate until done
        let mut count = 0;
        while voice.is_active() && count < 10000 {
            voice.next_sample();
            count += 1;
        }
        assert!(!voice.is_active());
    }

    #[test]
    fn test_voice_signal_multiplication() {
        let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
        let env = ADSR::new(0.0, 0.0, 1.0, 0.0, SAMPLE_RATE as f64);
        let mut voice = Voice::new(osc, env);

        voice.note_on(440.0, 0.8);

        // During sustain at 1.0, voice output should equal oscillator * envelope
        // Since envelope is at 1.0, voice output should equal oscillator output
        for _ in 0..10 {
            voice.next_sample();
        }

        // Voice should produce non-zero samples when active
        let sample = voice.next_sample();
        assert!(sample.abs() > 0.01); // Signal is present

        // When we release, samples should eventually go to zero
        voice.note_off();
        let mut count = 0;
        let mut final_sample = 1.0;
        while voice.is_active() && count < 10000 {
            final_sample = voice.next_sample();
            count += 1;
        }
        assert!((final_sample).abs() < 0.01); // Should be near zero after release
    }
}
