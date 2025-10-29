//! Noise generators for audio synthesis.
//!
//! This module contains various noise generator implementations.

mod white;
mod pink;

pub use white::WhiteNoise;
pub use pink::PinkNoise;
