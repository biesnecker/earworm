//! Earworm - An audio synthesis library for Rust
//!
//! This library provides oscillators and other building blocks for audio synthesis.

mod signal;
pub mod oscillators;
pub mod noise;
pub mod envelopes;
pub mod combinators;

// Re-export commonly used types at the crate root
pub use oscillators::{
    AudioSignal, Oscillator, PulseOscillator, SawtoothOscillator, SineOscillator,
    SquareOscillator, TriangleOscillator,
};
pub use noise::{PinkNoise, WhiteNoise};
pub use signal::{ConstantSignal, Param, Signal};
pub use envelopes::{Curve, ADSR};
pub use combinators::{
    Abs, Add, Clamp, Crossfade, Gain, Gate, Invert, Map, Max, Min, Mix, Multiply, Offset,
    SignalExt,
};
