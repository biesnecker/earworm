//! Noise generators for audio synthesis.
//!
//! This module contains various noise generator implementations.

mod pink;
mod white;

pub use pink::PinkNoise;
pub use white::WhiteNoise;
