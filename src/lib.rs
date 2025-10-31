//! Earworm - An audio synthesis library for Rust
//!
//! This library provides a flexible and composable system for audio synthesis,
//! built on trait-based signal processing abstractions.
//!
//! ## Feature Flags
//!
//! - `synth` (default): Enables synthesis components (oscillators, filters, effects, envelopes, noise)
//! - `music`: Enables music theory abstractions (notes, scales, sequencers)

// Core module - always compiled
pub mod core;

// Synthesis module - requires synth feature
#[cfg(feature = "synth")]
pub mod synthesis;

// Music module - requires music feature
#[cfg(feature = "music")]
pub mod music;

// Re-export core types at the crate root (always available)
pub use core::{
    Abs, Add, AudioSignal, Clamp, ConstantSignal, Crossfade, Gain, Gate, Invert, Map, Max, Min,
    Mix2, Mix3, Mix4, Multiply, Offset, Param, Pitched, Signal, SignalExt,
};

// Re-export synthesis types (only with synth feature)
#[cfg(feature = "synth")]
pub use synthesis::{
    AudioSignalExt, BiquadFilter, Bitcrusher, Compressor, Curve, Delay, Distortion, FilterType,
    Limiter, Oscillator, PinkNoise, PulseOscillator, SawtoothOscillator, SineOscillator,
    SquareOscillator, Tremolo, TriangleOscillator, Vibrato, WhiteNoise,
};

// Re-export music types (only with music feature)
#[cfg(feature = "music")]
pub use music::{
    ADSR, AHD, AR, Envelope, EnvelopeState, Metronome, Pattern, PlayState, Sequencer,
    StealingStrategy, Voice, VoiceAllocator,
    core::{Note, NoteEvent, ParseError, Pitch},
};

// Re-export the note! macro (only with music feature)
#[cfg(feature = "music")]
pub use earworm_macros::note;
