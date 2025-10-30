//! Delay effect with feedback and dry/wet mix.

use crate::signals::{AudioSignal, Param, Signal};

/// Delay effect with feedback and dry/wet mix.
///
/// Stores input samples in a ring buffer and plays them back after a specified time.
/// Feedback creates repeating echoes.
pub struct Delay<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> {
    source: S,
    buffer: Vec<f64>,
    write_pos: usize,

    // Parameters
    delay_time: Param, // delay time in seconds
    feedback: Param,   // 0.0 to ~0.95 (higher = more repeats, >1.0 = infinite/growing)
    mix: Param,        // dry/wet mix, 0.0 = all dry, 1.0 = all wet
}

impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> Delay<SAMPLE_RATE, S> {
    /// Creates a new delay effect.
    ///
    /// # Arguments
    ///
    /// * `source` - Input signal
    /// * `max_delay_time` - Maximum delay time in seconds (determines buffer size)
    /// * `delay_time` - Initial/modulated delay time in seconds
    /// * `feedback` - Feedback amount (0.0 = single echo, 0.5 = gradual decay, 0.95 = long tail)
    /// * `mix` - Dry/wet mix (0.0 = all dry/original, 1.0 = all wet/delayed)
    pub fn new(
        source: S,
        max_delay_time: f64,
        delay_time: impl Into<Param>,
        feedback: impl Into<Param>,
        mix: impl Into<Param>,
    ) -> Self {
        let buffer_size = (max_delay_time * SAMPLE_RATE as f64).ceil() as usize + 1;

        Self {
            source,
            buffer: vec![0.0; buffer_size],
            write_pos: 0,
            delay_time: delay_time.into(),
            feedback: feedback.into(),
            mix: mix.into(),
        }
    }

    /// Creates a simple echo effect.
    ///
    /// # Arguments
    ///
    /// * `source` - Input signal
    /// * `delay_time` - Time between echoes in seconds
    /// * `feedback` - Number of echoes (0.0-0.95)
    pub fn echo(source: S, delay_time: f64, feedback: f64) -> Self {
        Self::new(source, delay_time, delay_time, feedback, 0.5)
    }

    /// Creates a slapback delay (short, single echo).
    ///
    /// Common in rockabilly and vintage recordings.
    pub fn slapback(source: S) -> Self {
        Self::new(source, 0.2, 0.075, 0.3, 0.4)
    }
}

impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> Signal for Delay<SAMPLE_RATE, S> {
    fn next_sample(&mut self) -> f64 {
        let input = self.source.next_sample();

        // Get current parameter values
        let delay_time = self.delay_time.value().max(0.0);
        let feedback = self.feedback.value().clamp(0.0, 0.99); // Prevent runaway feedback
        let mix = self.mix.value().clamp(0.0, 1.0);

        // Calculate delay in samples
        let delay_samples = (delay_time * SAMPLE_RATE as f64) as usize;
        let delay_samples = delay_samples.min(self.buffer.len() - 1);

        // Calculate read position
        let read_pos = (self.write_pos + self.buffer.len() - delay_samples) % self.buffer.len();

        // Read delayed sample
        let delayed = self.buffer[read_pos];

        // Write input + feedback to buffer
        self.buffer[self.write_pos] = input + delayed * feedback;

        // Advance write position
        self.write_pos = (self.write_pos + 1) % self.buffer.len();

        // Mix dry and wet signals
        input * (1.0 - mix) + delayed * mix
    }
}

impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> AudioSignal<SAMPLE_RATE>
    for Delay<SAMPLE_RATE, S>
{
}
