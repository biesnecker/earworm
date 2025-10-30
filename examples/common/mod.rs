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
use std::io::stdout;
use std::panic;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Trait for audio state that can generate samples.
/// Types implementing this trait can be used as audio sources in interactive examples.
pub trait ExampleAudioState: Send + 'static {
    fn next_sample(&mut self) -> f64;
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

    // Event loop
    loop {
        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key_event) = event::read()?
        {
            match key_handler(&state, &key_event)? {
                KeyAction::Continue => {}
                KeyAction::Exit => break,
            }
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
