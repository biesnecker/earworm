#![cfg(feature = "music")]

use earworm::{NoteEvent, note};

#[test]
fn test_note_macro_with_octave() {
    let c4 = note!("C4");
    assert!((c4.pitch - 261.63).abs() < 0.01);
}

#[test]
fn test_note_macro_without_octave() {
    // Defaults to octave 4
    let c = note!("C");
    assert!((c.pitch - 261.63).abs() < 0.01);
}

#[test]
fn test_note_macro_sharps() {
    let csharp = note!("C#4");
    assert!((csharp.pitch - 277.18).abs() < 0.01);
}

#[test]
fn test_note_macro_flats() {
    let bflat = note!("Bb3");
    assert!((bflat.pitch - 233.08).abs() < 0.01);
}

#[test]
fn test_note_macro_a4() {
    let a4 = note!("A4");
    assert!((a4.pitch - 440.0).abs() < 0.01);
}

#[test]
fn test_note_macro_different_octaves() {
    let c5 = note!("C5");
    assert!((c5.pitch - 523.25).abs() < 0.01);
}

#[test]
fn test_note_macro_with_event() {
    let note = note!("D4");
    let event = NoteEvent::new(note, 0.7, Some(0.5));
    assert!((event.note.pitch - 293.66).abs() < 0.01);
    assert_eq!(event.velocity, 0.7);
    assert_eq!(event.duration, Some(0.5));
}
