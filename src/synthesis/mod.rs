//! Audio synthesis components.
//!
//! This module provides high-level building blocks for audio synthesis, including:
//! - Oscillators (sine, triangle, sawtooth, square, pulse)
//! - Filters (biquad IIR filters)
//! - Effects (delay, tremolo, vibrato, distortion, etc.)
//! - Envelopes (ADSR)
//! - Noise generators (white, pink)
//! - AudioSignalExt trait for convenient filter/effect chaining
//!
//! All synthesis components require the `synth` feature to be enabled.

mod audio_ext;
pub mod effects;
pub mod envelopes;
pub mod filters;
pub mod noise;
pub mod oscillators;

pub use audio_ext::AudioSignalExt;
pub use effects::{Bitcrusher, Compressor, Delay, Distortion, Limiter, Tremolo, Vibrato};
pub use envelopes::{ADSR, Curve};
pub use filters::{BiquadFilter, FilterType};
pub use noise::{PinkNoise, WhiteNoise};
pub use oscillators::{
    Oscillator, PulseOscillator, SawtoothOscillator, SineOscillator, SquareOscillator,
    TriangleOscillator,
};
