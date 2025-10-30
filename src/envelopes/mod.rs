//! Envelope generators and curve utilities for controlling parameter changes over time.
//!
//! This module provides tools for creating time-varying control signals, such as
//! ADSR envelopes and various interpolation curves.

mod curve;
mod adsr;

pub use curve::Curve;
pub use adsr::ADSR;
