//! Earworm - An audio synthesis library for Rust
//!
//! This library provides oscillators and other building blocks for audio synthesis.

pub mod combinators;
pub mod envelopes;
pub mod filters;
pub mod noise;
pub mod oscillators;
mod signal;

// Re-export commonly used types at the crate root
pub use combinators::{
    Abs, Add, Clamp, Crossfade, Gain, Gate, Invert, Map, Max, Min, Mix, Multiply, Offset,
    SignalExt,
};
pub use envelopes::{Curve, ADSR};
pub use filters::{BiquadFilter, FilterType};
pub use noise::{PinkNoise, WhiteNoise};
pub use oscillators::{
    AudioSignal, AudioSignalExt, Oscillator, PulseOscillator, SawtoothOscillator, SineOscillator,
    SquareOscillator, TriangleOscillator,
};
pub use signal::{ConstantSignal, Param, Signal};
