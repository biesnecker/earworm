//! Earworm - An audio synthesis library for Rust
//!
//! This library provides oscillators and other building blocks for audio synthesis.

mod signal;
pub mod oscillators;

// Re-export commonly used types at the crate root
pub use oscillators::{
    AudioSignal, Oscillator, SawtoothOscillator, SineOscillator, SquareOscillator,
    TriangleOscillator,
};
pub use signal::Signal;
