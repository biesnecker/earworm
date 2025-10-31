//! Core signal processing types and traits.
//!
//! This module provides the fundamental signal processing abstractions used
//! throughout the library, including:
//! - `Signal` trait for all signal sources and processors
//! - `AudioSignal` trait for sample-rate-aware signals
//! - `AudioSignalExt` and `SignalExt` traits for convenient combinators
//! - `Param` type for fixed or modulated parameters
//! - `ConstantSignal` for fixed values
//! - Signal combinators for composing signals

mod audio;
pub mod combinators;
mod signal;

pub use audio::AudioSignal;
pub use combinators::{
    Abs, Add, Clamp, Crossfade, Gain, Gate, Invert, Map, Max, Min, Mix2, Mix3, Mix4, Multiply,
    Offset, SignalExt,
};
pub use signal::{ConstantSignal, Param, Pitched, Signal};
