//! Wavetable oscillator for sample-based synthesis with arbitrary waveforms.
//!
//! # Design Overview
//!
//! `WavetableOscillator` enables playback of arbitrary waveforms stored as sample tables,
//! providing a flexible alternative to analytical oscillators. This is particularly useful
//! for complex timbres, sampled waveforms, or band-limited synthesis techniques.
//!
//! ## Core Architecture
//!
//! The oscillator is built around these key components:
//!
//! 1. **Wavetable Storage**: A `Vec<f64>` containing one cycle of the waveform
//!    - Samples are normalized to [-1.0, 1.0] range
//!    - Table size is arbitrary but powers of 2 (256, 512, 1024, 2048) are recommended for efficiency
//!    - Can be generated from functions or loaded from external sources
//!
//! 2. **Phase Accumulator**: Tracks playback position through the table
//!    - Uses fractional phase (0.0 to table_size as f64) for sub-sample precision
//!    - Phase increment calculated from frequency: `frequency * table_size / sample_rate`
//!    - Wraps modulo table_size to loop seamlessly
//!
//! 3. **Interpolation**: Handles fractional phase positions
//!    - Linear interpolation (default): good quality/performance balance
//!    - Cubic interpolation: higher quality at cost of ~4x computation
//!    - None/Nearest neighbor: lowest quality but fastest (mostly for testing)
//!
//! ## Type Parameters
//!
//! * `SAMPLE_RATE` - Sample rate in Hz (const generic, e.g., 44100 for CD quality)
//!
//! ## Trait Implementations
//!
//! The oscillator implements:
//! - `Signal`: Core sample generation via `next_sample()` and `process()`
//! - `AudioSignal<SAMPLE_RATE>`: Provides sample rate awareness
//! - `Pitched`: Frequency control via `set_frequency()` and `frequency()`
//! - `Oscillator`: State reset capability via `reset()`
//!
//! ## Construction Methods
//!
//! ```ignore
//! // From a pre-filled wavetable
//! let table = vec![0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5];
//! let mut osc = WavetableOscillator::<44100>::from_samples(440.0, table);
//!
//! // From a function (samples one cycle)
//! let mut osc = WavetableOscillator::<44100>::from_function(
//!     440.0,
//!     512,
//!     |phase| (phase * 2.0 * PI).sin()
//! );
//!
//! // Using helper functions for common waveforms
//! let mut sine = WavetableOscillator::<44100>::sine(440.0, 512);
//! let mut saw = WavetableOscillator::<44100>::saw(440.0, 1024);
//! ```
//!
//! ## Interpolation Modes
//!
//! The oscillator supports multiple interpolation modes for trading off quality vs. performance:
//!
//! ```ignore
//! let mut osc = WavetableOscillator::<44100>::from_samples(440.0, table)
//!     .with_interpolation(InterpolationMode::Cubic);
//! ```
//!
//! - `None`/`Nearest`: No interpolation, just rounds to nearest sample
//!   - Fastest but produces aliasing artifacts
//!   - Mainly useful for testing or lo-fi effects
//!
//! - `Linear`: Linear interpolation between adjacent samples (default)
//!   - Good balance of quality and performance
//!   - Single multiply and add per sample
//!   - Suitable for most applications
//!
//! - `Cubic`: 4-point Hermite cubic interpolation
//!   - Highest quality with smoothest results
//!   - ~4x more computation than linear
//!   - Best for high-quality synthesis or slow playback rates
//!
//! ## Example Usage
//!
//! ```ignore
//! use earworm::{Signal, WavetableOscillator, InterpolationMode};
//!
//! // Create a sine wave wavetable with 512 samples
//! let mut osc = WavetableOscillator::<44100>::sine(440.0, 512);
//!
//! // Generate some samples
//! for _ in 0..100 {
//!     let sample = osc.next_sample();
//! }
//!
//! // Switch to cubic interpolation for higher quality
//! osc.set_interpolation(InterpolationMode::Cubic);
//!
//! // Change frequency (detunes the wavetable playback)
//! osc.set_frequency(880.0);
//! ```
//!
//! ## Performance Considerations
//!
//! - Table size affects memory and cache performance
//! - Powers of 2 enable potential optimization (though not currently used)
//! - Linear interpolation is recommended default for most use cases
//! - Cubic interpolation best reserved for cases where quality is paramount
//! - Batch processing via `process()` may be optimized in the future
//!
//! ## Implementation Notes
//!
//! The oscillator maintains phase as a floating-point value representing the current
//! position in the wavetable. When generating samples:
//!
//! 1. The integer part of phase indexes into the table
//! 2. The fractional part is used for interpolation
//! 3. Phase is incremented by `frequency * table_size / sample_rate`
//! 4. Phase wraps using modulo to loop the waveform
//!
//! This approach provides:
//! - Smooth pitch changes without clicks
//! - Sub-sample timing precision
//! - Seamless looping of the waveform
//! - Efficient computation via simple arithmetic

use super::Oscillator;
use crate::core::Pitched;
use crate::{AudioSignal, Signal};
use std::f64::consts::PI;

#[cfg(feature = "wavetable-loader")]
use std::path::Path;

/// Interpolation mode for wavetable playback.
///
/// Determines how fractional positions between wavetable samples are handled.
/// Higher quality modes provide smoother output but require more computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpolationMode {
    /// No interpolation - round to nearest sample (lowest quality, fastest)
    None,
    /// Linear interpolation between adjacent samples (good quality/performance balance)
    Linear,
    /// Cubic (Hermite) interpolation using 4 points (highest quality, slowest)
    Cubic,
}

/// A wavetable oscillator for sample-based synthesis.
///
/// This oscillator plays back arbitrary waveforms stored as sample tables,
/// with configurable interpolation modes for quality vs. performance tradeoffs.
///
/// # Type Parameters
///
/// * `SAMPLE_RATE` - Sample rate in Hz (e.g., 44100 for CD quality)
///
/// # Examples
///
/// ```ignore
/// use earworm::{Signal, WavetableOscillator};
///
/// // Create a sine wave wavetable
/// let mut osc = WavetableOscillator::<44100>::sine(440.0, 512);
/// let sample = osc.next_sample();
/// ```
pub struct WavetableOscillator<const SAMPLE_RATE: u32> {
    /// The wavetable samples (one complete cycle)
    table: Vec<f64>,
    /// Current phase position in the table (0.0 to table.len() as f64)
    phase: f64,
    /// Phase increment per sample
    phase_increment: f64,
    /// Interpolation mode for playback
    interpolation: InterpolationMode,
}

impl<const SAMPLE_RATE: u32> WavetableOscillator<SAMPLE_RATE> {
    /// Creates a new wavetable oscillator from a vector of samples.
    ///
    /// The samples should represent one complete cycle of the waveform,
    /// normalized to the range [-1.0, 1.0].
    ///
    /// # Arguments
    ///
    /// * `frequency` - Initial playback frequency in Hz
    /// * `samples` - Wavetable samples (one cycle)
    ///
    /// # Panics
    ///
    /// Panics if `samples` is empty.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use earworm::WavetableOscillator;
    ///
    /// let table = vec![0.0, 1.0, 0.0, -1.0]; // Simple square-ish wave
    /// let mut osc = WavetableOscillator::<44100>::from_samples(440.0, table);
    /// ```
    pub fn from_samples(frequency: f64, samples: Vec<f64>) -> Self {
        assert!(!samples.is_empty(), "Wavetable cannot be empty");
        let table_size = samples.len() as f64;
        let phase_increment = frequency * table_size / SAMPLE_RATE as f64;

        Self {
            table: samples,
            phase: 0.0,
            phase_increment,
            interpolation: InterpolationMode::Linear,
        }
    }

    /// Creates a wavetable oscillator by sampling a function.
    ///
    /// The function should map phase (0.0 to 1.0) to amplitude (-1.0 to 1.0).
    ///
    /// # Arguments
    ///
    /// * `frequency` - Initial playback frequency in Hz
    /// * `table_size` - Number of samples in the wavetable
    /// * `f` - Function mapping phase [0.0, 1.0) to amplitude
    ///
    /// # Panics
    ///
    /// Panics if `table_size` is zero.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use earworm::WavetableOscillator;
    /// use std::f64::consts::PI;
    ///
    /// // Create a sine wave
    /// let mut osc = WavetableOscillator::<44100>::from_function(
    ///     440.0,
    ///     512,
    ///     |phase| (phase * 2.0 * PI).sin()
    /// );
    /// ```
    pub fn from_function<F>(frequency: f64, table_size: usize, f: F) -> Self
    where
        F: Fn(f64) -> f64,
    {
        assert!(table_size > 0, "Table size must be greater than zero");

        let samples: Vec<f64> = (0..table_size)
            .map(|i| {
                let phase = i as f64 / table_size as f64;
                f(phase)
            })
            .collect();

        Self::from_samples(frequency, samples)
    }

    /// Creates a sine wave wavetable.
    ///
    /// # Arguments
    ///
    /// * `frequency` - Initial playback frequency in Hz
    /// * `table_size` - Number of samples in the wavetable (recommend power of 2)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use earworm::WavetableOscillator;
    ///
    /// let mut osc = WavetableOscillator::<44100>::sine(440.0, 512);
    /// ```
    pub fn sine(frequency: f64, table_size: usize) -> Self {
        Self::from_function(frequency, table_size, |phase| (phase * 2.0 * PI).sin())
    }

    /// Creates a sawtooth wave wavetable.
    ///
    /// # Arguments
    ///
    /// * `frequency` - Initial playback frequency in Hz
    /// * `table_size` - Number of samples in the wavetable (recommend power of 2)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use earworm::WavetableOscillator;
    ///
    /// let mut osc = WavetableOscillator::<44100>::saw(440.0, 1024);
    /// ```
    pub fn saw(frequency: f64, table_size: usize) -> Self {
        Self::from_function(frequency, table_size, |phase| 2.0 * phase - 1.0)
    }

    /// Creates a square wave wavetable.
    ///
    /// # Arguments
    ///
    /// * `frequency` - Initial playback frequency in Hz
    /// * `table_size` - Number of samples in the wavetable (recommend power of 2)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use earworm::WavetableOscillator;
    ///
    /// let mut osc = WavetableOscillator::<44100>::square(440.0, 512);
    /// ```
    pub fn square(frequency: f64, table_size: usize) -> Self {
        Self::from_function(
            frequency,
            table_size,
            |phase| {
                if phase < 0.5 { 1.0 } else { -1.0 }
            },
        )
    }

    /// Creates a triangle wave wavetable.
    ///
    /// # Arguments
    ///
    /// * `frequency` - Initial playback frequency in Hz
    /// * `table_size` - Number of samples in the wavetable (recommend power of 2)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use earworm::WavetableOscillator;
    ///
    /// let mut osc = WavetableOscillator::<44100>::triangle(440.0, 512);
    /// ```
    pub fn triangle(frequency: f64, table_size: usize) -> Self {
        Self::from_function(frequency, table_size, |phase| {
            if phase < 0.5 {
                4.0 * phase - 1.0
            } else {
                -4.0 * phase + 3.0
            }
        })
    }

    /// Sets the interpolation mode.
    ///
    /// # Arguments
    ///
    /// * `mode` - The interpolation mode to use
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use earworm::{WavetableOscillator, InterpolationMode};
    ///
    /// let mut osc = WavetableOscillator::<44100>::sine(440.0, 512);
    /// osc.set_interpolation(InterpolationMode::Cubic);
    /// ```
    pub fn set_interpolation(&mut self, mode: InterpolationMode) {
        self.interpolation = mode;
    }

    /// Builder-style method to set interpolation mode.
    ///
    /// # Arguments
    ///
    /// * `mode` - The interpolation mode to use
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use earworm::{WavetableOscillator, InterpolationMode};
    ///
    /// let mut osc = WavetableOscillator::<44100>::sine(440.0, 512)
    ///     .with_interpolation(InterpolationMode::Cubic);
    /// ```
    pub fn with_interpolation(mut self, mode: InterpolationMode) -> Self {
        self.interpolation = mode;
        self
    }

    /// Gets the current interpolation mode.
    pub fn interpolation(&self) -> InterpolationMode {
        self.interpolation
    }

    /// Gets the size of the wavetable.
    pub fn table_size(&self) -> usize {
        self.table.len()
    }

    /// Loads a wavetable from a WAV file (requires `wavetable-loader` feature).
    ///
    /// Reads the first channel of a mono or stereo WAV file and uses the samples
    /// as a wavetable. The audio data is automatically normalized to [-1.0, 1.0].
    ///
    /// # Arguments
    ///
    /// * `frequency` - Initial playback frequency in Hz
    /// * `path` - Path to the WAV file
    ///
    /// # Returns
    ///
    /// Returns `Ok(WavetableOscillator)` on success, or an error if the file
    /// cannot be read or is not a valid WAV file.
    ///
    /// # Notes
    ///
    /// - The entire file is loaded into memory as the wavetable
    /// - For single-cycle waveforms, use short WAV files (one cycle)
    /// - For longer samples, this will create a looping wavetable
    /// - Sample rate conversion is not performed - the file is read as-is
    /// - Multi-channel files will only use the first channel
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use earworm::WavetableOscillator;
    ///
    /// // Load a single-cycle waveform
    /// let osc = WavetableOscillator::<44100>::from_wav_file(
    ///     440.0,
    ///     "waveforms/saw.wav"
    /// )?;
    /// ```
    #[cfg(feature = "wavetable-loader")]
    pub fn from_wav_file<P: AsRef<Path>>(
        frequency: f64,
        path: P,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut reader = hound::WavReader::open(path)?;
        let spec = reader.spec();

        // Read all samples from the first channel
        let samples: Result<Vec<f64>, _> = match spec.sample_format {
            hound::SampleFormat::Float => reader
                .samples::<f32>()
                .map(|s| s.map(|v| v as f64))
                .collect(),
            hound::SampleFormat::Int => {
                let max_value = (1 << (spec.bits_per_sample - 1)) as f64;
                reader
                    .samples::<i32>()
                    .map(|s| s.map(|v| v as f64 / max_value))
                    .collect()
            }
        };

        let samples = samples?;

        if samples.is_empty() {
            return Err("WAV file contains no samples".into());
        }

        // For multi-channel files, we only take every Nth sample (first channel)
        let channel_samples: Vec<f64> = if spec.channels > 1 {
            samples
                .iter()
                .step_by(spec.channels as usize)
                .copied()
                .collect()
        } else {
            samples
        };

        Ok(Self::from_samples(frequency, channel_samples))
    }

    /// Reads a sample from the wavetable at the current phase using the configured interpolation.
    #[inline]
    fn read_sample(&self) -> f64 {
        let table_size = self.table.len();

        match self.interpolation {
            InterpolationMode::None => {
                // Round to nearest sample
                let index = (self.phase.round() as usize) % table_size;
                self.table[index]
            }
            InterpolationMode::Linear => {
                // Linear interpolation between two adjacent samples
                let index0 = self.phase.floor() as usize % table_size;
                let index1 = (index0 + 1) % table_size;
                let frac = self.phase.fract();

                let sample0 = self.table[index0];
                let sample1 = self.table[index1];

                sample0 + frac * (sample1 - sample0)
            }
            InterpolationMode::Cubic => {
                // Cubic (Hermite) interpolation using 4 points
                let index1 = self.phase.floor() as usize % table_size;
                let index0 = if index1 == 0 {
                    table_size - 1
                } else {
                    index1 - 1
                };
                let index2 = (index1 + 1) % table_size;
                let index3 = (index1 + 2) % table_size;
                let frac = self.phase.fract();

                let y0 = self.table[index0];
                let y1 = self.table[index1];
                let y2 = self.table[index2];
                let y3 = self.table[index3];

                // Hermite interpolation
                let c0 = y1;
                let c1 = 0.5 * (y2 - y0);
                let c2 = y0 - 2.5 * y1 + 2.0 * y2 - 0.5 * y3;
                let c3 = 0.5 * (y3 - y0) + 1.5 * (y1 - y2);

                c0 + frac * (c1 + frac * (c2 + frac * c3))
            }
        }
    }
}

impl<const SAMPLE_RATE: u32> Signal for WavetableOscillator<SAMPLE_RATE> {
    fn next_sample(&mut self) -> f64 {
        let sample = self.read_sample();

        // Advance phase and wrap
        self.phase += self.phase_increment;
        let table_size = self.table.len() as f64;
        if self.phase >= table_size {
            self.phase -= table_size;
        }

        sample
    }
}

impl<const SAMPLE_RATE: u32> AudioSignal<SAMPLE_RATE> for WavetableOscillator<SAMPLE_RATE> {}

impl<const SAMPLE_RATE: u32> Pitched for WavetableOscillator<SAMPLE_RATE> {
    fn set_frequency(&mut self, frequency: f64) {
        let table_size = self.table.len() as f64;
        self.phase_increment = frequency * table_size / SAMPLE_RATE as f64;
    }

    fn frequency(&self) -> f64 {
        let table_size = self.table.len() as f64;
        self.phase_increment * SAMPLE_RATE as f64 / table_size
    }
}

impl<const SAMPLE_RATE: u32> Oscillator for WavetableOscillator<SAMPLE_RATE> {
    fn reset(&mut self) {
        self.phase = 0.0;
    }
}
