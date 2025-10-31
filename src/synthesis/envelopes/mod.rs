//! Envelope generators and curve utilities for controlling parameter changes over time.
//!
//! This module provides tools for creating time-varying control signals, such as
//! ADSR envelopes and various interpolation curves.

mod adsr;
mod curve;

pub use adsr::ADSR;
pub use curve::Curve;
