//! Audio effects for signal processing.
//!
//! This module provides time-based and modulation effects that can be applied
//! to any signal source.

mod bitcrusher;
mod compressor;
mod delay;
mod distortion;
mod limiter;
mod tremolo;
mod vibrato;

pub use bitcrusher::Bitcrusher;
pub use compressor::Compressor;
pub use delay::Delay;
pub use distortion::Distortion;
pub use limiter::Limiter;
pub use tremolo::Tremolo;
pub use vibrato::Vibrato;
