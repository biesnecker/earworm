//! Audio effects for signal processing.
//!
//! This module provides time-based and modulation effects that can be applied
//! to any signal source.

mod bitcrusher;
mod delay;
mod tremolo;

pub use bitcrusher::Bitcrusher;
pub use delay::Delay;
pub use tremolo::Tremolo;
