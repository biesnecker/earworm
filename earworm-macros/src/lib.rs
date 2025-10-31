use proc_macro::TokenStream;
use quote::quote;
use syn::{LitStr, parse_macro_input};

/// Creates a `Note` at compile time from a string literal.
///
/// This macro parses the note string at compile time and generates
/// the corresponding `Note::from_pitch()` call with the computed
/// frequency. This is zero-cost and can be used in hot paths.
///
/// # Format
///
/// The format is: `<pitch>[octave]` where:
/// - `pitch` can be: C, D, E, F, G, A, B with optional # or b
/// - `octave` is optional, defaults to 4 (middle octave)
/// - When provided, octave must be -1 to 9
///
/// # Examples
///
/// ```ignore
/// use earworm::note;
///
/// // With octave
/// let c4 = note!("C4");
///
/// // Without octave (defaults to 4)
/// let c = note!("C");
///
/// // With sharps
/// let csharp = note!("C#4");
///
/// // With flats
/// let bflat = note!("Bb3");
/// ```
#[proc_macro]
pub fn note(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let note_str = input.value();

    // Parse the note string at compile time
    match parse_note(&note_str) {
        Ok((pitch, octave)) => {
            let midi_note = pitch_to_midi(pitch, octave);
            let frequency = midi_to_freq(midi_note);

            // Generate the code
            let expanded = quote! {
                {
                    earworm::music::core::Note {
                        pitch: #frequency,
                    }
                }
            };

            TokenStream::from(expanded)
        }
        Err(e) => {
            let error_msg = format!("Invalid note string '{}': {}", note_str, e);
            let expanded = quote! {
                compile_error!(#error_msg)
            };
            TokenStream::from(expanded)
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Pitch {
    C = 0,
    CSharp = 1,
    D = 2,
    DSharp = 3,
    E = 4,
    F = 5,
    FSharp = 6,
    G = 7,
    GSharp = 8,
    A = 9,
    ASharp = 10,
    B = 11,
}

impl Pitch {
    fn semitone_offset(&self) -> u8 {
        *self as u8
    }
}

fn parse_pitch(s: &str) -> Result<Pitch, String> {
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
        _ => Err(format!("invalid pitch '{}'", s)),
    }
}

fn parse_note(s: &str) -> Result<(Pitch, i8), String> {
    if s.is_empty() {
        return Err("empty string".to_string());
    }

    // Find where the octave number starts
    let octave_start = s.chars().position(|c| c.is_numeric() || c == '-');

    let (pitch_str, octave) = match octave_start {
        Some(0) => {
            return Err("string starts with number".to_string());
        }
        Some(pos) => {
            let pitch_str = &s[..pos];
            let octave_str = &s[pos..];

            let octave = octave_str
                .parse::<i8>()
                .map_err(|_| format!("invalid octave '{}'", octave_str))?;

            if !(-1..=9).contains(&octave) {
                return Err(format!("octave {} out of range (-1 to 9)", octave));
            }

            (pitch_str, octave)
        }
        None => {
            // No octave specified, default to 4
            (s, 4)
        }
    };

    let pitch = parse_pitch(pitch_str)?;
    Ok((pitch, octave))
}

fn pitch_to_midi(pitch: Pitch, octave: i8) -> u8 {
    ((octave + 1) as u8 * 12 + pitch.semitone_offset()).clamp(0, 127)
}

fn midi_to_freq(midi_note: u8) -> f64 {
    440.0 * 2.0_f64.powf((midi_note as f64 - 69.0) / 12.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pitch() {
        assert!(matches!(parse_pitch("C"), Ok(Pitch::C)));
        assert!(matches!(parse_pitch("C#"), Ok(Pitch::CSharp)));
        assert!(matches!(parse_pitch("Db"), Ok(Pitch::CSharp)));
        assert!(parse_pitch("H").is_err());
    }

    #[test]
    fn test_parse_note() {
        let (pitch, octave) = parse_note("C4").unwrap();
        assert!(matches!(pitch, Pitch::C));
        assert_eq!(octave, 4);

        let (pitch, octave) = parse_note("C").unwrap();
        assert!(matches!(pitch, Pitch::C));
        assert_eq!(octave, 4); // default

        let (pitch, octave) = parse_note("F#5").unwrap();
        assert!(matches!(pitch, Pitch::FSharp));
        assert_eq!(octave, 5);

        assert!(parse_note("").is_err());
        assert!(parse_note("4").is_err());
        assert!(parse_note("C10").is_err());
    }

    #[test]
    fn test_midi_conversion() {
        assert_eq!(pitch_to_midi(Pitch::C, 4), 60);
        assert_eq!(pitch_to_midi(Pitch::A, 4), 69);

        let freq = midi_to_freq(69);
        assert!((freq - 440.0).abs() < 0.01);

        let freq = midi_to_freq(60);
        assert!((freq - 261.63).abs() < 0.01);
    }
}
