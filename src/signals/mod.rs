//! Signal processing types and traits.
//!
//! This module provides the core signal processing abstractions used throughout
//! the library, including:
//! - `Signal` trait for all signal sources and processors
//! - `AudioSignal` trait for sample-rate-aware signals
//! - `AudioSignalExt` trait for convenient filter methods
//! - `Param` type for fixed or modulated parameters
//! - `ConstantSignal` for fixed values

mod audio;
mod core;

pub use audio::{AudioSignal, AudioSignalExt};
pub use core::{ConstantSignal, Param, Signal};
