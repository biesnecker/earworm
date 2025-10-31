//! Frequency type for representing pitch in Hz.

use super::core::Note;

/// A frequency value in Hz.
///
/// This type provides a unified interface for working with pitch, accepting
/// frequencies directly in Hz, MIDI note numbers, or `Note` structs.
///
/// # Examples
///
/// ```
/// use earworm::music::frequency::Frequency;
///
/// // From Hz
/// let freq: Frequency = 440.0.into();
/// assert_eq!(freq.as_f64(), 440.0);
///
/// // From MIDI note number (69 = A4 = 440 Hz)
/// let freq: Frequency = 69u8.into();
/// assert!((freq.as_f64() - 440.0).abs() < 0.01);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frequency(f64);

impl Frequency {
    /// Creates a new frequency from Hz.
    ///
    /// # Arguments
    ///
    /// * `hz` - Frequency in Hz
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::frequency::Frequency;
    ///
    /// let freq = Frequency::from_hz(440.0);
    /// assert_eq!(freq.as_f64(), 440.0);
    /// ```
    pub fn from_hz(hz: f64) -> Self {
        Frequency(hz)
    }

    /// Creates a new frequency from a MIDI note number.
    ///
    /// # Arguments
    ///
    /// * `midi_note` - MIDI note number (0-127, where 69 = A4 = 440 Hz)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::frequency::Frequency;
    ///
    /// let freq = Frequency::from_midi(69); // A4
    /// assert!((freq.as_f64() - 440.0).abs() < 0.01);
    /// ```
    pub fn from_midi(midi_note: u8) -> Self {
        // MIDI note to frequency: f = 440 * 2^((n - 69) / 12)
        let hz = 440.0 * 2.0_f64.powf((f64::from(midi_note) - 69.0) / 12.0);
        Frequency(hz)
    }

    /// Returns the frequency value in Hz.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::frequency::Frequency;
    ///
    /// let freq = Frequency::from_hz(440.0);
    /// assert_eq!(freq.as_f64(), 440.0);
    /// ```
    pub fn as_f64(&self) -> f64 {
        self.0
    }
}

impl From<f64> for Frequency {
    fn from(hz: f64) -> Self {
        Frequency::from_hz(hz)
    }
}

impl From<u8> for Frequency {
    fn from(midi_note: u8) -> Self {
        Frequency::from_midi(midi_note)
    }
}

impl From<Note> for Frequency {
    fn from(note: Note) -> Self {
        Frequency::from_hz(note.pitch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_hz() {
        let freq = Frequency::from_hz(440.0);
        assert_eq!(freq.as_f64(), 440.0);
    }

    #[test]
    fn test_from_f64() {
        let freq: Frequency = 440.0.into();
        assert_eq!(freq.as_f64(), 440.0);
    }

    #[test]
    fn test_from_midi() {
        let freq = Frequency::from_midi(69); // A4
        assert!((freq.as_f64() - 440.0).abs() < 0.01);

        let freq = Frequency::from_midi(57); // A3
        assert!((freq.as_f64() - 220.0).abs() < 0.01);
    }

    #[test]
    fn test_from_u8() {
        let freq: Frequency = 69u8.into();
        assert!((freq.as_f64() - 440.0).abs() < 0.01);
    }

    #[test]
    fn test_from_note() {
        let note = Note::from_midi(69);
        let freq: Frequency = note.into();
        assert!((freq.as_f64() - 440.0).abs() < 0.01);
    }
}
