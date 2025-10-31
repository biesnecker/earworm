//! Audio filters for signal processing.
//!
//! This module provides various types of audio filters including
//! low-pass, high-pass, band-pass, notch, and all-pass filters.
//!
//! The primary filter implementation is [`BiquadFilter`], which uses
//! second-order IIR filtering to provide efficient, high-quality filtering
//! with support for parameter modulation.

mod biquad;

pub use self::biquad::{BiquadFilter, FilterType};
// mod bandpass;
