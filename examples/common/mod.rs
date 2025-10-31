//! Common utilities for audio examples.

use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, StreamConfig};
use crossterm::{
    ExecutableCommand,
    event::{
        self, Event, KeyCode, KeyEvent, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::io::{Write, stdout};
use std::panic;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Trait for audio state that can generate samples.
/// Types implementing this trait can be used as audio sources in interactive examples.
pub trait ExampleAudioState: Send + 'static {
    fn next_sample(&mut self) -> f64;

    /// Optional output/metrics information to display in the UI.
    /// Return None to hide the output line, or Some(String) to show it.
    /// This is called periodically from the UI thread, not the audio thread.
    fn output_info(&self) -> Option<String> {
        None
    }
}

/// Configuration for keyboard enhancements (needed for detecting key press/release).
#[derive(Default)]
pub struct KeyboardConfig {
    /// Enable keyboard enhancements (for press/release detection)
    pub enable_enhancements: bool,
}

impl KeyboardConfig {
    /// Create config that enables keyboard enhancements for press/release detection
    #[allow(dead_code)]
    pub fn with_enhancements() -> Self {
        Self {
            enable_enhancements: true,
        }
    }
}

/// Key handling result that controls the event loop
pub enum KeyAction {
    /// Continue the event loop
    Continue,
    /// Exit the event loop
    Exit,
}

/// Runs an interactive audio example with terminal UI.
///
/// This function handles all the boilerplate:
/// - Audio device setup and stream creation
/// - Terminal raw mode and alternate screen
/// - Panic hook for terminal cleanup
/// - Event loop with key polling
///
/// # Arguments
///
/// * `state` - The audio state (must implement AudioState trait)
/// * `keyboard_config` - Configuration for keyboard handling
/// * `initial_ui` - Closure to draw the initial UI
/// * `key_handler` - Closure that handles key events and returns whether to continue or exit
///
/// # Example
///
/// ```no_run
/// use earworm::Signal;
///
/// struct MyAudioState { /* ... */ }
///
/// impl AudioState for MyAudioState {
///     fn next_sample(&mut self) -> f64 { /* ... */ }
/// }
///
/// run_interactive_example(
///     MyAudioState::new(),
///     KeyboardConfig::default(),
///     |state| { /* draw initial UI */ Ok(()) },
///     |state, key_event| {
///         match key_event.code {
///             KeyCode::Char('q') => KeyAction::Exit,
///             _ => KeyAction::Continue,
///         }
///     }
/// )
/// ```
pub fn run_interactive_example<S, F, K>(
    state: S,
    keyboard_config: KeyboardConfig,
    initial_ui: F,
    key_handler: K,
) -> Result<()>
where
    S: ExampleAudioState,
    F: FnOnce(&Arc<Mutex<S>>) -> Result<()>,
    K: Fn(&Arc<Mutex<S>>, &KeyEvent) -> Result<KeyAction>,
{
    // Setup audio
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::anyhow!("No output device available"))?;

    let config = device.default_output_config()?;
    let state = Arc::new(Mutex::new(state));

    // Start audio stream
    let _stream = match config.sample_format() {
        SampleFormat::F32 => create_audio_stream::<f32, S>(&device, &config.into(), state.clone())?,
        SampleFormat::I16 => create_audio_stream::<i16, S>(&device, &config.into(), state.clone())?,
        SampleFormat::U16 => create_audio_stream::<u16, S>(&device, &config.into(), state.clone())?,
        sample_format => {
            return Err(anyhow::anyhow!(
                "Unsupported sample format: {}",
                sample_format
            ));
        }
    };

    // Setup terminal - keyboard enhancements MUST come before alternate screen
    if keyboard_config.enable_enhancements {
        stdout().execute(PushKeyboardEnhancementFlags(
            KeyboardEnhancementFlags::REPORT_EVENT_TYPES,
        ))?;
    }

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(crossterm::cursor::Hide)?;

    // Set up panic hook to restore terminal on panic
    let has_enhancements = keyboard_config.enable_enhancements;
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        cleanup_terminal(has_enhancements);
        original_hook(panic_info);
    }));

    // Draw initial UI
    initial_ui(&state)?;

    // Event loop with periodic output info updates
    let mut last_output_update = std::time::Instant::now();
    loop {
        // Poll for keyboard events
        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key_event) = event::read()?
        {
            match key_handler(&state, &key_event)? {
                KeyAction::Continue => {}
                KeyAction::Exit => break,
            }
        }

        // Periodically update output info display (if provided)
        if last_output_update.elapsed() >= Duration::from_millis(100) {
            let state_guard = state.lock().unwrap();
            if let Some(info) = state_guard.output_info() {
                // Move to second line and display output info
                let mut stdout = stdout();
                stdout.execute(crossterm::cursor::MoveTo(0, 1))?;
                stdout.execute(crossterm::terminal::Clear(
                    crossterm::terminal::ClearType::CurrentLine,
                ))?;
                write!(stdout, "{}", info)?;
                stdout.flush()?;
            }
            drop(state_guard);
            last_output_update = std::time::Instant::now();
        }
    }

    // Cleanup terminal
    cleanup_terminal(keyboard_config.enable_enhancements);

    Ok(())
}

/// Creates an audio stream that pulls samples from the audio state.
fn create_audio_stream<T, S>(
    device: &cpal::Device,
    config: &StreamConfig,
    state: Arc<Mutex<S>>,
) -> Result<cpal::Stream>
where
    T: Sample + FromSample<f64> + cpal::SizedSample,
    S: ExampleAudioState,
{
    let channels = config.channels as usize;

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            let mut state = state.lock().unwrap();
            for frame in data.chunks_mut(channels) {
                let sample = state.next_sample();
                let value: T = T::from_sample(sample);
                for s in frame.iter_mut() {
                    *s = value;
                }
            }
        },
        |err| eprintln!("Audio stream error: {}", err),
        None,
    )?;

    stream.play()?;
    Ok(stream)
}

/// Cleans up terminal state (cursor, alternate screen, raw mode).
fn cleanup_terminal(has_keyboard_enhancements: bool) {
    if has_keyboard_enhancements {
        let _ = stdout().execute(PopKeyboardEnhancementFlags);
    }
    let _ = stdout().execute(crossterm::cursor::Show);
    let _ = stdout().execute(LeaveAlternateScreen);
    let _ = disable_raw_mode();
}

/// Helper to check if a key code is a quit key (Q, ESC).
pub fn is_quit_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc)
}

/// Maps computer keyboard keys to MIDI note numbers.
///
/// Layout mimics a piano keyboard with two rows:
/// - Bottom row (A-K): White keys starting at C4 (middle C)
/// - Top row (W-O): Black keys (sharps/flats)
///
/// The mapping follows a piano-style layout:
/// ```text
/// W E   T Y U   O P
///  ↓ ↓   ↓ ↓ ↓   ↓ ↓
/// A S D F G H J K L
/// C D E F G A B C D (note names)
/// ```
///
/// # Returns
///
/// `Some(midi_note)` if the key maps to a note, `None` otherwise.
///
/// # Examples
///
/// ```no_run
/// use common::key_to_midi_note;
/// use crossterm::event::KeyCode;
///
/// assert_eq!(key_to_midi_note(KeyCode::Char('a')), Some(60)); // C4
/// assert_eq!(key_to_midi_note(KeyCode::Char('w')), Some(61)); // C#4
/// assert_eq!(key_to_midi_note(KeyCode::Char('s')), Some(62)); // D4
/// ```
#[allow(dead_code)]
pub fn key_to_midi_note(code: KeyCode) -> Option<u8> {
    match code {
        // Bottom row: white keys (C4 to D5)
        KeyCode::Char('a') | KeyCode::Char('A') => Some(60), // C4 (middle C)
        KeyCode::Char('s') | KeyCode::Char('S') => Some(62), // D4
        KeyCode::Char('d') | KeyCode::Char('D') => Some(64), // E4
        KeyCode::Char('f') | KeyCode::Char('F') => Some(65), // F4
        KeyCode::Char('g') | KeyCode::Char('G') => Some(67), // G4
        KeyCode::Char('h') | KeyCode::Char('H') => Some(69), // A4
        KeyCode::Char('j') | KeyCode::Char('J') => Some(71), // B4
        KeyCode::Char('k') | KeyCode::Char('K') => Some(72), // C5
        KeyCode::Char('l') | KeyCode::Char('L') => Some(74), // D5

        // Top row: black keys (sharps)
        KeyCode::Char('w') | KeyCode::Char('W') => Some(61), // C#4
        KeyCode::Char('e') | KeyCode::Char('E') => Some(63), // D#4
        // No F between E and F
        KeyCode::Char('t') | KeyCode::Char('T') => Some(66), // F#4
        KeyCode::Char('y') | KeyCode::Char('Y') => Some(68), // G#4
        KeyCode::Char('u') | KeyCode::Char('U') => Some(70), // A#4
        // No B# between B and C
        KeyCode::Char('o') | KeyCode::Char('O') => Some(73), // C#5
        KeyCode::Char('p') | KeyCode::Char('P') => Some(75), // D#5

        _ => None,
    }
}

/// Converts a MIDI note number to its musical name (e.g., "C4", "A#3").
///
/// Uses sharp notation for accidentals (e.g., "C#" rather than "Db").
///
/// # Arguments
///
/// * `midi_note` - MIDI note number (0-127)
///
/// # Returns
///
/// A string representation of the note name with octave (e.g., "C4", "G#5").
///
/// # Examples
///
/// ```no_run
/// use common::midi_note_to_name;
///
/// assert_eq!(midi_note_to_name(60), "C4");  // Middle C
/// assert_eq!(midi_note_to_name(69), "A4");  // 440 Hz
/// assert_eq!(midi_note_to_name(61), "C#4"); // C sharp
/// ```
#[allow(dead_code)]
pub fn midi_note_to_name(midi_note: u8) -> String {
    const NOTE_NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];

    let octave = (midi_note as i32 / 12) - 1;
    let note_index = (midi_note % 12) as usize;

    format!("{}{}", NOTE_NAMES[note_index], octave)
}

/// Draws a standard keyboard layout UI for musical examples.
///
/// This function renders a consistent keyboard reference that shows the piano-style
/// layout mapping computer keys to musical notes. It's designed to work with the
/// common example framework's status line (which appears on line 1).
///
/// # Arguments
///
/// * `title` - The title to display at the top of the UI
/// * `extra_info` - Optional additional information to show (e.g., controls, instructions)
///
/// # UI Layout
///
/// The function renders:
/// - Line 0: Title
/// - Line 1: (Reserved for status updates from `output_info()`)
/// - Lines 2+: Keyboard layout diagram
/// - Bottom: Extra info (if provided) and quit instructions
///
/// # Examples
///
/// ```no_run
/// use common::draw_keyboard_ui;
///
/// // Simple usage
/// draw_keyboard_ui("My Synth Demo", None)?;
///
/// // With extra controls
/// draw_keyboard_ui(
///     "Filter Demo",
///     Some("SPACE = Cycle filters | 1-5 = Adjust resonance")
/// )?;
/// ```
#[allow(dead_code)]
pub fn draw_keyboard_ui(title: &str, extra_info: Option<&str>) -> Result<()> {
    let mut stdout = stdout();
    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;
    stdout.execute(crossterm::cursor::MoveTo(0, 0))?;

    // Title
    write!(stdout, "{}\r\n", title)?;
    // Line 1 is reserved for status from output_info()
    write!(stdout, "\r\n")?;
    write!(stdout, "\r\n")?;

    // Keyboard layout
    write!(stdout, "Keyboard Layout:\r\n")?;
    write!(stdout, "\r\n")?;
    write!(stdout, "  W E   T Y U   O P     (Black keys)\r\n")?;
    write!(stdout, " A S D F G H J K L      (White keys)\r\n")?;
    write!(stdout, " C D E F G A B C D      (Notes)\r\n")?;
    write!(stdout, "\r\n")?;

    // Extra info (if provided)
    if let Some(info) = extra_info {
        write!(stdout, "{}\r\n", info)?;
        write!(stdout, "\r\n")?;
    }

    // Quit instructions
    write!(stdout, "Q/ESC = Quit\r\n")?;

    stdout.flush()?;
    Ok(())
}
