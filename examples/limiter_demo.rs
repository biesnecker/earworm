//! Demonstration of the Limiter effect for preventing clipping.
//!
//! This example creates a sine wave that's too loud (gain > 1.0) and uses
//! a limiter to prevent clipping. Press SPACE to toggle the limiter on/off
//! to hear the difference. Press Q or ESC to quit.

mod common;

use anyhow::Result;
use common::{ExampleAudioState, KeyAction, KeyboardConfig, is_quit_key, run_interactive_example};
use crossterm::{
    ExecutableCommand,
    event::{KeyCode, KeyEvent, KeyEventKind},
};
use earworm::{Gain, Limiter, Mix3, Signal, SignalExt, SineOscillator};
use std::io::{Write, stdout};

const SAMPLE_RATE: u32 = 44100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LimiterMode {
    Bypassed,
    Enabled,
}

impl LimiterMode {
    fn toggle(&self) -> Self {
        match self {
            LimiterMode::Bypassed => LimiterMode::Enabled,
            LimiterMode::Enabled => LimiterMode::Bypassed,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            LimiterMode::Bypassed => "OFF (clipping!)",
            LimiterMode::Enabled => "ON (protected)",
        }
    }
}

// Type for our loud signal chain
type LoudSignal = Mix3<
    Gain<SineOscillator<SAMPLE_RATE>>,
    Gain<SineOscillator<SAMPLE_RATE>>,
    Gain<SineOscillator<SAMPLE_RATE>>,
>;

enum LimitedSignal {
    Bypassed(LoudSignal),
    Enabled(Limiter<SAMPLE_RATE, LoudSignal>),
}

impl Signal for LimitedSignal {
    fn next_sample(&mut self) -> f64 {
        match self {
            LimitedSignal::Bypassed(sig) => sig.next_sample(),
            LimitedSignal::Enabled(limiter) => limiter.next_sample(),
        }
    }
}

impl LimitedSignal {
    fn current_gain(&self) -> f64 {
        match self {
            LimitedSignal::Bypassed(_) => 1.0,
            LimitedSignal::Enabled(limiter) => limiter.current_gain(),
        }
    }
}

struct AudioState {
    signal: LimitedSignal,
    reference_signal: LoudSignal, // Parallel signal to track input level
    mode: LimiterMode,
    // Metrics for display
    peak_input: f64,
    peak_output: f64,
    peak_decay: f64,
}

impl AudioState {
    fn new() -> Self {
        Self {
            signal: LimitedSignal::Enabled(Limiter::new(Self::create_loud_signal(), 0.9, 0.05)),
            reference_signal: Self::create_loud_signal(),
            mode: LimiterMode::Enabled,
            peak_input: 0.0,
            peak_output: 0.0,
            peak_decay: 0.995,
        }
    }

    fn create_loud_signal() -> LoudSignal {
        // Create a loud oscillator that would clip without limiting
        // We use a 440 Hz sine wave with excessive gain (2.5x)
        let loud_osc = SineOscillator::<SAMPLE_RATE>::new(440.0).gain(2.5);

        // Add some harmonics to make clipping more audible
        let harmonic1 = SineOscillator::<SAMPLE_RATE>::new(880.0).gain(0.5);
        let harmonic2 = SineOscillator::<SAMPLE_RATE>::new(1320.0).gain(0.25);

        Mix3::new(loud_osc, 1.0, harmonic1, 1.0, harmonic2, 1.0)
    }

    fn toggle_limiter(&mut self) {
        self.mode = self.mode.toggle();
        self.signal = match self.mode {
            LimiterMode::Bypassed => LimitedSignal::Bypassed(Self::create_loud_signal()),
            LimiterMode::Enabled => {
                LimitedSignal::Enabled(Limiter::new(Self::create_loud_signal(), 0.9, 0.05))
            }
        };
        self.reference_signal = Self::create_loud_signal();
    }
}

impl ExampleAudioState for AudioState {
    fn next_sample(&mut self) -> f64 {
        // Get output from the (possibly limited) signal
        let output = self.signal.next_sample();

        // Get reference input level from parallel unlimited signal
        let reference_input = self.reference_signal.next_sample();

        let input_level = reference_input.abs();
        let output_level = output.abs();

        // Update peak meters with decay
        self.peak_input *= self.peak_decay;
        self.peak_output *= self.peak_decay;

        // Update with new peaks
        if input_level > self.peak_input {
            self.peak_input = input_level;
        }
        if output_level > self.peak_output {
            self.peak_output = output_level;
        }

        // When bypassed, clamp to prevent actual clipping (just to be safe for speakers)
        // but the distortion will still be audible
        match self.mode {
            LimiterMode::Bypassed => output.clamp(-1.0, 1.0),
            LimiterMode::Enabled => output,
        }
    }

    fn output_info(&self) -> Option<String> {
        // Get actual gain reduction from limiter
        let current_gain = self.signal.current_gain();
        let gain_reduction_db = 20.0 * current_gain.log10();

        // Create a simple text meter
        let input_meter = create_meter(self.peak_input, 20);
        let output_meter = create_meter(self.peak_output, 20);

        Some(format!(
            "Input: [{}] {:.2}  Output: [{}] {:.2}  GR: {:.1} dB",
            input_meter, self.peak_input, output_meter, self.peak_output, gain_reduction_db
        ))
    }
}

fn create_meter(level: f64, width: usize) -> String {
    let filled = (level * width as f64).round() as usize;
    let filled = filled.min(width);
    let empty = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

fn draw_ui(mode: LimiterMode) -> Result<()> {
    let mut stdout = stdout();
    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;
    stdout.execute(crossterm::cursor::MoveTo(0, 0))?;
    write!(stdout, "Limiter: {} | SPACE=toggle  Q=quit", mode.name())?;
    stdout.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    run_interactive_example(
        AudioState::new(),
        KeyboardConfig::default(),
        |state| draw_ui(state.lock().unwrap().mode),
        |state, key_event: &KeyEvent| {
            if !matches!(key_event.kind, KeyEventKind::Press) {
                return Ok(KeyAction::Continue);
            }

            match key_event.code {
                KeyCode::Char(' ') => {
                    let mut s = state.lock().unwrap();
                    s.toggle_limiter();
                    let mode = s.mode;
                    drop(s);
                    draw_ui(mode)?;
                    Ok(KeyAction::Continue)
                }
                code if is_quit_key(code) => Ok(KeyAction::Exit),
                _ => Ok(KeyAction::Continue),
            }
        },
    )?;

    println!("\nGoodbye!");
    Ok(())
}
