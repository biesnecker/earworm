//! Oscillator implementations for audio synthesis.
//!
//! This module contains the core `Oscillator` trait and various oscillator implementations.

mod pulse;
mod sawtooth;
mod sine;
mod square;
mod traits;
mod triangle;
mod wavetable;

pub use pulse::PulseOscillator;
pub use sawtooth::SawtoothOscillator;
pub use sine::SineOscillator;
pub use square::SquareOscillator;
pub use traits::Oscillator;
pub use triangle::TriangleOscillator;
pub use wavetable::{InterpolationMode, WavetableOscillator};
