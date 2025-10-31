use std::fmt;
use std::str::FromStr;

/// Error type for parsing musical notes from strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The input string was empty
    Empty,
    /// The pitch name was invalid or unrecognized
    InvalidPitch(String),
    /// The octave was invalid or out of range
    InvalidOctave(String),
    /// The input format was invalid
    InvalidFormat(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Empty => write!(f, "input string is empty"),
            ParseError::InvalidPitch(s) => write!(f, "invalid pitch name: '{}'", s),
            ParseError::InvalidOctave(s) => write!(f, "invalid octave: '{}'", s),
            ParseError::InvalidFormat(s) => write!(f, "invalid note format: '{}'", s),
        }
    }
}

impl std::error::Error for ParseError {}

/// Musical note names in the chromatic scale.
///
/// Each variant represents one of the 12 notes in the chromatic scale.
/// Use sharp notation (e.g., `FSharp` instead of G flat).
///
/// # Examples
///
/// ```
/// use earworm::music::core::{Pitch, Note};
///
/// // Create a C4 note (Middle C)
/// let note = Note::from_pitch(Pitch::C, 4);
/// assert!((note.pitch - 261.63).abs() < 0.01);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pitch {
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B,
}

impl Pitch {
    /// Returns the semitone offset from C (0-11) for this note.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::core::Pitch;
    ///
    /// assert_eq!(Pitch::C.semitone_offset(), 0);
    /// assert_eq!(Pitch::CSharp.semitone_offset(), 1);
    /// assert_eq!(Pitch::A.semitone_offset(), 9);
    /// ```
    pub fn semitone_offset(&self) -> u8 {
        match self {
            Pitch::C => 0,
            Pitch::CSharp => 1,
            Pitch::D => 2,
            Pitch::DSharp => 3,
            Pitch::E => 4,
            Pitch::F => 5,
            Pitch::FSharp => 6,
            Pitch::G => 7,
            Pitch::GSharp => 8,
            Pitch::A => 9,
            Pitch::ASharp => 10,
            Pitch::B => 11,
        }
    }

    /// Converts a note name and octave to a MIDI note number.
    ///
    /// MIDI note numbers range from 0-127, where:
    /// - C-1 = 0
    /// - C0 = 12
    /// - C4 (Middle C) = 60
    /// - A4 = 69
    /// - G9 = 127
    ///
    /// # Arguments
    ///
    /// * `octave` - The octave number (-1 to 9)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::core::Pitch;
    ///
    /// assert_eq!(Pitch::C.to_midi_note(4), 60); // Middle C
    /// assert_eq!(Pitch::A.to_midi_note(4), 69); // A4 = 440 Hz
    /// ```
    pub fn to_midi_note(&self, octave: i8) -> u8 {
        ((octave + 1) as u8 * 12 + self.semitone_offset()).clamp(0, 127)
    }

    /// Parses a pitch from a string.
    ///
    /// Supports both sharp (#) and flat (b) notation.
    /// - Sharps: "C#", "D#", "F#", "G#", "A#"
    /// - Flats: "Db", "Eb", "Gb", "Ab", "Bb" (converted to sharp equivalents)
    ///
    /// # Arguments
    ///
    /// * `s` - The pitch string (case-insensitive)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::core::Pitch;
    ///
    /// let pitch = Pitch::from_str("C#").unwrap();
    /// assert_eq!(pitch, Pitch::CSharp);
    ///
    /// let pitch = Pitch::from_str("Bb").unwrap(); // B flat = A#
    /// assert_eq!(pitch, Pitch::ASharp);
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, ParseError> {
        s.parse()
    }
}

impl std::str::FromStr for Pitch {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_uppercase();

        match s.as_str() {
            "C" => Ok(Pitch::C),
            "C#" | "DB" => Ok(Pitch::CSharp),
            "D" => Ok(Pitch::D),
            "D#" | "EB" => Ok(Pitch::DSharp),
            "E" | "FB" => Ok(Pitch::E),
            "F" | "E#" => Ok(Pitch::F),
            "F#" | "GB" => Ok(Pitch::FSharp),
            "G" => Ok(Pitch::G),
            "G#" | "AB" => Ok(Pitch::GSharp),
            "A" => Ok(Pitch::A),
            "A#" | "BB" => Ok(Pitch::ASharp),
            "B" | "CB" => Ok(Pitch::B),
            _ => Err(ParseError::InvalidPitch(s)),
        }
    }
}

/// A musical note representing a pitch.
///
/// `Note` contains only the frequency (pitch) information.
/// Performance details like velocity and duration are separate concerns.
///
/// # Examples
///
/// ```
/// use earworm::music::core::Note;
///
/// // A4 note at 440 Hz
/// let note = Note::new(440.0);
///
/// // Middle C
/// let c4 = Note::from_pitch(earworm::music::core::Pitch::C, 4);
///
/// // Custom/microtonal pitch
/// let custom = Note::new(432.5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Note {
    /// The frequency of the note in Hz
    pub pitch: f64,
}

/// A note event for sequencing and performance.
///
/// `NoteEvent` bundles a `Note` with performance parameters like
/// velocity (how hard to play) and duration (how long to play).
///
/// # Examples
///
/// ```
/// use earworm::music::core::{Note, NoteEvent};
///
/// let note = Note::new(440.0);
/// let event = NoteEvent::new(note, 0.8, Some(0.5));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoteEvent {
    /// The note to play
    pub note: Note,

    /// The velocity/amplitude (typically 0.0 to 1.0)
    pub velocity: f64,

    /// Optional duration in seconds
    pub duration: Option<f64>,
}

impl Note {
    /// Creates a new `Note` with the given pitch (frequency in Hz).
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::core::Note;
    ///
    /// let note = Note::new(440.0);  // A4
    /// assert_eq!(note.pitch, 440.0);
    /// ```
    pub fn new(pitch: f64) -> Self {
        Self { pitch }
    }

    /// Converts a MIDI note number to frequency in Hz using equal temperament tuning.
    ///
    /// Uses the formula: f = 440 * 2^((midi - 69) / 12)
    /// where MIDI note 69 = A4 = 440 Hz
    ///
    /// # Arguments
    ///
    /// * `midi_note` - MIDI note number (0-127, where 69 = A4)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::core::Note;
    ///
    /// let freq = Note::midi_to_freq(69); // A4
    /// assert!((freq - 440.0).abs() < 0.01);
    ///
    /// let freq = Note::midi_to_freq(60); // Middle C
    /// assert!((freq - 261.63).abs() < 0.01);
    /// ```
    pub fn midi_to_freq(midi_note: u8) -> f64 {
        440.0 * 2.0_f64.powf((midi_note as f64 - 69.0) / 12.0)
    }

    /// Creates a note from a MIDI note number.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::core::Note;
    ///
    /// // Middle C
    /// let note = Note::from_midi(60);
    /// assert!((note.pitch - 261.63).abs() < 0.01);
    /// ```
    pub fn from_midi(midi_note: u8) -> Self {
        Self::new(Self::midi_to_freq(midi_note))
    }

    /// Creates a note from a pitch name and octave.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::core::{Note, Pitch};
    ///
    /// // Middle C (C4)
    /// let note = Note::from_pitch(Pitch::C, 4);
    /// assert!((note.pitch - 261.63).abs() < 0.01);
    ///
    /// // A4 (440 Hz reference pitch)
    /// let note = Note::from_pitch(Pitch::A, 4);
    /// assert!((note.pitch - 440.0).abs() < 0.01);
    /// ```
    pub fn from_pitch(pitch: Pitch, octave: i8) -> Self {
        let midi_note = pitch.to_midi_note(octave);
        Self::new(Self::midi_to_freq(midi_note))
    }
}

impl NoteEvent {
    /// Creates a new `NoteEvent`.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::core::{Note, NoteEvent};
    ///
    /// let note = Note::new(440.0);
    /// let event = NoteEvent::new(note, 0.8, Some(0.5));
    /// assert_eq!(event.velocity, 0.8);
    /// assert_eq!(event.duration, Some(0.5));
    /// ```
    pub fn new(note: Note, velocity: f64, duration: Option<f64>) -> Self {
        Self {
            note,
            velocity,
            duration,
        }
    }

    /// Creates a `NoteEvent` from a MIDI note number.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::core::NoteEvent;
    ///
    /// let event = NoteEvent::from_midi(60, 64, Some(0.5));
    /// assert!((event.note.pitch - 261.63).abs() < 0.01);
    /// assert!((event.velocity - 0.503).abs() < 0.01);
    /// ```
    pub fn from_midi(midi_note: u8, velocity: u8, duration: Option<f64>) -> Self {
        Self::new(
            Note::from_midi(midi_note),
            velocity as f64 / 127.0,
            duration,
        )
    }

    /// Creates a `NoteEvent` from a pitch name and octave.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::core::{NoteEvent, Pitch};
    ///
    /// let event = NoteEvent::from_pitch(Pitch::C, 4, 1.0, Some(0.5));
    /// assert!((event.note.pitch - 261.63).abs() < 0.01);
    /// ```
    pub fn from_pitch(pitch: Pitch, octave: i8, velocity: f64, duration: Option<f64>) -> Self {
        Self::new(Note::from_pitch(pitch, octave), velocity, duration)
    }
}

impl FromStr for Note {
    type Err = ParseError;

    /// Parses a musical note from a string.
    ///
    /// The format is: `<pitch>[octave]` where:
    /// - `pitch` can be: C, D, E, F, G, A, B with optional # or b
    /// - `octave` is optional, defaults to 4 (middle octave) if not provided
    /// - When provided, octave must be a number from -1 to 9
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use earworm::music::core::Note;
    ///
    /// // Parse a note with octave
    /// let note = Note::from_str("C4").unwrap();
    /// assert!((note.pitch - 261.63).abs() < 0.01);
    ///
    /// // Parse without octave (defaults to octave 4)
    /// let note = Note::from_str("C").unwrap();
    /// assert!((note.pitch - 261.63).abs() < 0.01);
    ///
    /// // Flat notation
    /// let note = Note::from_str("Bb3").unwrap(); // B flat = A#3
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseError::Empty);
        }

        // Find where the octave number starts
        let octave_start = s.chars().position(|c| c.is_numeric() || c == '-');

        let (pitch_str, octave) = match octave_start {
            Some(0) => {
                // String starts with a number, invalid
                return Err(ParseError::InvalidPitch(String::new()));
            }
            Some(pos) => {
                // Has an octave specified
                let pitch_str = &s[..pos];
                let octave_str = &s[pos..];

                let octave = octave_str
                    .parse::<i8>()
                    .map_err(|_| ParseError::InvalidOctave(octave_str.to_string()))?;

                if !(-1..=9).contains(&octave) {
                    return Err(ParseError::InvalidOctave(octave_str.to_string()));
                }

                (pitch_str, octave)
            }
            None => {
                // No octave specified, default to 4
                (s, 4)
            }
        };

        // Parse pitch
        let pitch = Pitch::from_str(pitch_str)?;

        // Create note
        Ok(Self::from_pitch(pitch, octave))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_note() {
        let note = Note::new(440.0);
        assert_eq!(note.pitch, 440.0);
    }

    #[test]
    fn test_note_event() {
        let note = Note::new(440.0);
        let event = NoteEvent::new(note, 0.8, Some(1.0));
        assert_eq!(event.note.pitch, 440.0);
        assert_eq!(event.velocity, 0.8);
        assert_eq!(event.duration, Some(1.0));
    }

    #[test]
    fn test_midi_to_freq() {
        // A4 = 440 Hz
        let freq = Note::midi_to_freq(69);
        assert!((freq - 440.0).abs() < 0.01);

        // Middle C (C4) ≈ 261.63 Hz
        let freq = Note::midi_to_freq(60);
        assert!((freq - 261.63).abs() < 0.01);

        // C5 ≈ 523.25 Hz
        let freq = Note::midi_to_freq(72);
        assert!((freq - 523.25).abs() < 0.01);
    }

    #[test]
    fn test_from_midi() {
        let note = Note::from_midi(60);
        assert!((note.pitch - 261.63).abs() < 0.01);
    }

    #[test]
    fn test_from_midi_event() {
        let event = NoteEvent::from_midi(60, 64, Some(0.5));
        assert!((event.note.pitch - 261.63).abs() < 0.01);
        assert!((event.velocity - 0.503).abs() < 0.01);
        assert_eq!(event.duration, Some(0.5));
    }

    #[test]
    fn test_note_copy_clone() {
        let note1 = Note::new(440.0);
        let note2 = note1;
        assert_eq!(note1, note2);
    }

    #[test]
    fn test_pitch_semitone_offset() {
        assert_eq!(Pitch::C.semitone_offset(), 0);
        assert_eq!(Pitch::CSharp.semitone_offset(), 1);
        assert_eq!(Pitch::D.semitone_offset(), 2);
        assert_eq!(Pitch::E.semitone_offset(), 4);
        assert_eq!(Pitch::F.semitone_offset(), 5);
        assert_eq!(Pitch::FSharp.semitone_offset(), 6);
        assert_eq!(Pitch::A.semitone_offset(), 9);
        assert_eq!(Pitch::B.semitone_offset(), 11);
    }

    #[test]
    fn test_pitch_to_midi_note() {
        // Middle C (C4) = MIDI 60
        assert_eq!(Pitch::C.to_midi_note(4), 60);

        // A4 = MIDI 69 (440 Hz reference)
        assert_eq!(Pitch::A.to_midi_note(4), 69);

        // C0 = MIDI 12
        assert_eq!(Pitch::C.to_midi_note(0), 12);

        // C-1 = MIDI 0 (lowest MIDI note)
        assert_eq!(Pitch::C.to_midi_note(-1), 0);

        // Various notes in different octaves
        assert_eq!(Pitch::FSharp.to_midi_note(5), 78);
        assert_eq!(Pitch::B.to_midi_note(3), 59);
    }

    #[test]
    fn test_from_pitch() {
        // Middle C (C4) ≈ 261.63 Hz
        let note = Note::from_pitch(Pitch::C, 4);
        assert!((note.pitch - 261.63).abs() < 0.01);

        // A4 = 440 Hz
        let note = Note::from_pitch(Pitch::A, 4);
        assert!((note.pitch - 440.0).abs() < 0.01);

        // F#5 ≈ 739.99 Hz
        let note = Note::from_pitch(Pitch::FSharp, 5);
        assert!((note.pitch - 739.99).abs() < 0.01);
    }

    #[test]
    fn test_from_pitch_event() {
        let event = NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5));
        assert!((event.note.pitch - 261.63).abs() < 0.01);
        assert_eq!(event.velocity, 0.8);
        assert_eq!(event.duration, Some(0.5));
    }

    #[test]
    fn test_from_pitch_various_octaves() {
        // Test different octaves of the same note
        let c4 = Note::from_pitch(Pitch::C, 4);
        let c5 = Note::from_pitch(Pitch::C, 5);

        // C5 should be exactly twice the frequency of C4 (one octave higher)
        assert!((c5.pitch / c4.pitch - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_pitch_from_str() {
        // Natural notes
        assert_eq!(Pitch::from_str("C").unwrap(), Pitch::C);
        assert_eq!(Pitch::from_str("D").unwrap(), Pitch::D);
        assert_eq!(Pitch::from_str("E").unwrap(), Pitch::E);

        // Sharps
        assert_eq!(Pitch::from_str("C#").unwrap(), Pitch::CSharp);
        assert_eq!(Pitch::from_str("F#").unwrap(), Pitch::FSharp);

        // Flats (converted to sharp equivalents)
        assert_eq!(Pitch::from_str("Db").unwrap(), Pitch::CSharp);
        assert_eq!(Pitch::from_str("Bb").unwrap(), Pitch::ASharp);

        // Case insensitive
        assert_eq!(Pitch::from_str("c").unwrap(), Pitch::C);
        assert_eq!(Pitch::from_str("c#").unwrap(), Pitch::CSharp);
        assert_eq!(Pitch::from_str("bb").unwrap(), Pitch::ASharp);

        // Invalid pitch
        assert!(Pitch::from_str("H").is_err());
        assert!(Pitch::from_str("X#").is_err());
    }

    #[test]
    fn test_note_from_str() {
        // Basic parsing
        let note = Note::from_str("C4").unwrap();
        assert!((note.pitch - 261.63).abs() < 0.01);

        // A4 (440 Hz)
        let note = Note::from_str("A4").unwrap();
        assert!((note.pitch - 440.0).abs() < 0.01);

        // With sharps
        let note = Note::from_str("C#4").unwrap();
        assert!((note.pitch - 277.18).abs() < 0.01);

        // With flats
        let note = Note::from_str("Bb3").unwrap();
        let note_sharp = Note::from_str("A#3").unwrap();
        assert!((note.pitch - note_sharp.pitch).abs() < 0.01);

        // Different octaves
        let note = Note::from_str("C0").unwrap();
        assert!((note.pitch - 16.35).abs() < 0.01);

        let note = Note::from_str("C5").unwrap();
        assert!((note.pitch - 523.25).abs() < 0.01);

        // Negative octave
        let note = Note::from_str("C-1").unwrap();
        assert!((note.pitch - 8.18).abs() < 0.01);

        // Case insensitive
        let note = Note::from_str("c4").unwrap();
        assert!((note.pitch - 261.63).abs() < 0.01);

        let note = Note::from_str("C#4").unwrap();
        let note_lower = Note::from_str("c#4").unwrap();
        assert!((note.pitch - note_lower.pitch).abs() < 0.01);
    }

    #[test]
    fn test_note_from_str_default_octave() {
        // No octave defaults to 4 (middle octave)
        let note = Note::from_str("C").unwrap();
        assert!((note.pitch - 261.63).abs() < 0.01);

        // All natural notes without octave
        let note = Note::from_str("A").unwrap();
        assert!((note.pitch - 440.0).abs() < 0.01); // A4

        // Sharps without octave
        let note = Note::from_str("C#").unwrap();
        assert!((note.pitch - 277.18).abs() < 0.01); // C#4

        // Flats without octave
        let note = Note::from_str("Bb").unwrap();
        let note_sharp = Note::from_str("A#").unwrap();
        assert!((note.pitch - note_sharp.pitch).abs() < 0.01);
    }

    #[test]
    fn test_note_from_str_errors() {
        // Empty string
        assert!(matches!(Note::from_str(""), Err(ParseError::Empty)));

        // Invalid pitch
        assert!(matches!(
            Note::from_str("H4"),
            Err(ParseError::InvalidPitch(_))
        ));

        // Invalid pitch without octave
        assert!(matches!(
            Note::from_str("H"),
            Err(ParseError::InvalidPitch(_))
        ));

        // Invalid octave
        assert!(matches!(
            Note::from_str("C10"),
            Err(ParseError::InvalidOctave(_))
        ));

        assert!(matches!(
            Note::from_str("C-2"),
            Err(ParseError::InvalidOctave(_))
        ));

        // Octave without pitch
        assert!(matches!(
            Note::from_str("4"),
            Err(ParseError::InvalidPitch(_))
        ));

        // Invalid pitch (multiple letters before octave)
        assert!(matches!(
            Note::from_str("CC4"),
            Err(ParseError::InvalidPitch(_))
        ));
    }

    // Note: Tests for the note! macro are in tests/note_macro.rs
}
