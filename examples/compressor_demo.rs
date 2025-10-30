//! Interactive compressor effect demo.
//!
//! This example demonstrates the Compressor effect with various presets.
//! Press SPACE to cycle through compressor presets.
//! Press UP/DOWN to adjust threshold, LEFT/RIGHT to adjust ratio.
//! Press Q or ESC to quit.

mod common;

use anyhow::Result;
use common::{ExampleAudioState, KeyAction, KeyboardConfig, is_quit_key, run_interactive_example};
use crossterm::{
    ExecutableCommand,
    event::{KeyCode, KeyEvent, KeyEventKind},
};
use earworm::{
    Compressor, Gain, Mix3, Multiply, Offset, Signal, SignalExt, SineOscillator, TriangleOscillator,
};
use std::io::{Write, stdout};

const SAMPLE_RATE: u32 = 44100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompressorPreset {
    Off,
    Vocal,
    Punch,
    Glue,
    Custom,
}

impl CompressorPreset {
    fn next(self) -> Self {
        match self {
            CompressorPreset::Off => CompressorPreset::Vocal,
            CompressorPreset::Vocal => CompressorPreset::Punch,
            CompressorPreset::Punch => CompressorPreset::Glue,
            CompressorPreset::Glue => CompressorPreset::Custom,
            CompressorPreset::Custom => CompressorPreset::Off,
        }
    }

    fn name(self) -> &'static str {
        match self {
            CompressorPreset::Off => "Off",
            CompressorPreset::Vocal => "Vocal (3:1, soft)",
            CompressorPreset::Punch => "Punch (4:1, hard)",
            CompressorPreset::Glue => "Glue (2:1, gentle)",
            CompressorPreset::Custom => "Custom",
        }
    }
}

// Dynamic signal with LFO-modulated gain to demonstrate compression
type DynamicSignal = Multiply<
    Mix3<
        Offset<Gain<TriangleOscillator<SAMPLE_RATE>>>,
        Offset<Gain<SineOscillator<SAMPLE_RATE>>>,
        Offset<Gain<SineOscillator<SAMPLE_RATE>>>,
    >,
    Offset<Gain<SineOscillator<SAMPLE_RATE>>>, // LFO for amplitude modulation
>;

enum CompressorWrapper {
    Off(DynamicSignal),
    Vocal(Compressor<SAMPLE_RATE, DynamicSignal>),
    Punch(Compressor<SAMPLE_RATE, DynamicSignal>),
    Glue(Compressor<SAMPLE_RATE, DynamicSignal>),
    Custom(Compressor<SAMPLE_RATE, DynamicSignal>),
}

impl Signal for CompressorWrapper {
    fn next_sample(&mut self) -> f64 {
        match self {
            CompressorWrapper::Off(sig) => sig.next_sample() * 0.6,
            CompressorWrapper::Vocal(comp) => comp.next_sample() * 0.6,
            CompressorWrapper::Punch(comp) => comp.next_sample() * 0.6,
            CompressorWrapper::Glue(comp) => comp.next_sample() * 0.6,
            CompressorWrapper::Custom(comp) => comp.next_sample() * 0.6,
        }
    }
}

impl CompressorWrapper {
    fn current_gain(&self) -> f64 {
        match self {
            CompressorWrapper::Off(_) => 1.0,
            CompressorWrapper::Vocal(comp) => comp.current_gain(),
            CompressorWrapper::Punch(comp) => comp.current_gain(),
            CompressorWrapper::Glue(comp) => comp.current_gain(),
            CompressorWrapper::Custom(comp) => comp.current_gain(),
        }
    }
}

struct AudioState {
    signal: CompressorWrapper,
    reference_signal: DynamicSignal, // Parallel signal to track input level
    preset: CompressorPreset,
    threshold: f64,
    ratio: f64,
    // Metrics for display
    peak_input: f64,
    peak_output: f64,
    peak_decay: f64,
}

impl AudioState {
    fn new() -> Self {
        let preset = CompressorPreset::Off;
        Self {
            signal: Self::create_signal(preset, 0.5, 4.0),
            reference_signal: Self::create_dynamic_source(),
            preset,
            threshold: 0.5,
            ratio: 4.0,
            peak_input: 0.0,
            peak_output: 0.0,
            peak_decay: 0.995, // Decay coefficient for peak meter
        }
    }

    fn create_dynamic_source() -> DynamicSignal {
        // Create a signal with varying dynamics to demonstrate compression
        // Main tone
        let main = TriangleOscillator::<SAMPLE_RATE>::new(220.0)
            .gain(0.8)
            .offset(0.0);

        // Add some harmonics with different levels
        let harmonic1 = SineOscillator::<SAMPLE_RATE>::new(440.0)
            .gain(0.4)
            .offset(0.0);

        let harmonic2 = SineOscillator::<SAMPLE_RATE>::new(660.0)
            .gain(0.6)
            .offset(0.0);

        let mixed = Mix3::new(main, 1.0, harmonic1, 1.0, harmonic2, 1.0);

        // LFO modulating the amplitude at 2 Hz to create pumping/varying levels
        // Maps from [-1, 1] to [0.3, 1.2] so you get quiet and loud sections
        let lfo = SineOscillator::<SAMPLE_RATE>::new(2.0)
            .gain(0.45)
            .offset(0.75);

        mixed.multiply(lfo)
    }

    fn create_signal(preset: CompressorPreset, threshold: f64, ratio: f64) -> CompressorWrapper {
        let source = Self::create_dynamic_source();
        match preset {
            CompressorPreset::Off => CompressorWrapper::Off(source),
            CompressorPreset::Vocal => CompressorWrapper::Vocal(Compressor::vocal(source)),
            CompressorPreset::Punch => CompressorWrapper::Punch(Compressor::punch(source)),
            CompressorPreset::Glue => CompressorWrapper::Glue(Compressor::glue(source)),
            CompressorPreset::Custom => {
                CompressorWrapper::Custom(Compressor::new(source, threshold, ratio, 0.01, 0.1, 0.0))
            }
        }
    }

    fn switch_preset(&mut self) {
        self.preset = self.preset.next();
        self.signal = Self::create_signal(self.preset, self.threshold, self.ratio);
        self.reference_signal = Self::create_dynamic_source();
    }

    fn adjust_threshold(&mut self, delta: f64) {
        self.threshold = (self.threshold + delta).clamp(0.1, 0.9);
        if self.preset == CompressorPreset::Custom {
            self.signal = Self::create_signal(self.preset, self.threshold, self.ratio);
            self.reference_signal = Self::create_dynamic_source();
        }
    }

    fn adjust_ratio(&mut self, delta: f64) {
        self.ratio = (self.ratio + delta).clamp(1.0, 20.0);
        if self.preset == CompressorPreset::Custom {
            self.signal = Self::create_signal(self.preset, self.threshold, self.ratio);
            self.reference_signal = Self::create_dynamic_source();
        }
    }
}

impl ExampleAudioState for AudioState {
    fn next_sample(&mut self) -> f64 {
        // Get output from the (possibly compressed) signal
        let output = self.signal.next_sample();

        // Get reference input level from parallel uncompressed signal
        let reference_input = self.reference_signal.next_sample() * 0.6;

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

        output
    }

    fn output_info(&self) -> Option<String> {
        // Get actual gain reduction from compressor
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

fn draw_ui(state: &AudioState) -> Result<()> {
    let mut stdout = stdout();
    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;
    stdout.execute(crossterm::cursor::MoveTo(0, 0))?;

    let preset_str = state.preset.name();
    let params = if state.preset == CompressorPreset::Custom {
        format!(
            " | Threshold: {:.2} | Ratio: {:.1}:1",
            state.threshold, state.ratio
        )
    } else {
        String::new()
    };

    write!(
        stdout,
        "Compressor: {}{}  |  SPACE=switch  ↑↓=threshold  ←→=ratio  Q=quit",
        preset_str, params
    )?;

    stdout.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    run_interactive_example(
        AudioState::new(),
        KeyboardConfig::default(),
        |state| {
            let state = state.lock().unwrap();
            draw_ui(&state)
        },
        |state, key_event: &KeyEvent| {
            if !matches!(key_event.kind, KeyEventKind::Press) {
                return Ok(KeyAction::Continue);
            }

            match key_event.code {
                KeyCode::Char(' ') => {
                    state.lock().unwrap().switch_preset();
                }
                KeyCode::Up => {
                    state.lock().unwrap().adjust_threshold(0.05);
                }
                KeyCode::Down => {
                    state.lock().unwrap().adjust_threshold(-0.05);
                }
                KeyCode::Right => {
                    state.lock().unwrap().adjust_ratio(0.5);
                }
                KeyCode::Left => {
                    state.lock().unwrap().adjust_ratio(-0.5);
                }
                code if is_quit_key(code) => return Ok(KeyAction::Exit),
                _ => return Ok(KeyAction::Continue),
            }

            let state_guard = state.lock().unwrap();
            draw_ui(&state_guard)?;
            Ok(KeyAction::Continue)
        },
    )?;

    println!("\nGoodbye!");
    Ok(())
}
