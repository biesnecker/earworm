//! Earworm - An audio synthesis library for Rust
//!
//! This library provides oscillators and other building blocks for audio synthesis.

pub mod combinators;
pub mod effects;
pub mod envelopes;
pub mod filters;
pub mod noise;
pub mod oscillators;
pub mod signals;

// Re-export commonly used types at the crate root
pub use combinators::{
    Abs, Add, Clamp, Crossfade, Gain, Gate, Invert, Map, Max, Min, Mix2, Mix3, Mix4, Multiply,
    Offset, SignalExt,
};
pub use effects::{Compressor, Delay, Distortion, Limiter, Tremolo, Vibrato};
pub use envelopes::{ADSR, Curve};
pub use filters::{BiquadFilter, FilterType};
pub use noise::{PinkNoise, WhiteNoise};
pub use oscillators::{
    Oscillator, PulseOscillator, SawtoothOscillator, SineOscillator, SquareOscillator,
    TriangleOscillator,
};
pub use signals::{AudioSignal, AudioSignalExt, ConstantSignal, Param, Signal};
