//! Curve utilities for controlling parameter changes over time.
//!
//! This module provides interpolation curves for shaping envelopes, LFOs,
//! and other time-varying parameters.

mod curve;

pub use curve::Curve;
