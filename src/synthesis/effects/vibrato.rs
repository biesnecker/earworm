//! Vibrato effect using pitch modulation.

use crate::core::{AudioSignal, Param, Signal};

/// Vibrato effect that creates pitch modulation.
///
/// Vibrato modulates the pitch of an audio signal by using a variable delay line.
/// An LFO (typically a sine wave) modulates the delay time, which causes the pitch
/// to vary up and down. This creates the characteristic "wobble" or vibrato effect
/// commonly used on guitars, vocals, and other instruments.
///
/// The depth parameter controls how much the pitch varies (in cents, where 100 cents = 1 semitone),
/// and the rate parameter controls how fast the vibrato cycles.
///
/// # Examples
///
/// ```
/// use earworm::{SineOscillator, Vibrato};
///
/// // Create a 440 Hz tone with vibrato
/// let osc = SineOscillator::<44100>::new(440.0);
/// let mut vibrato = Vibrato::new(osc, 5.0, 20.0); // 5 Hz rate, 20 cents depth
/// ```
pub struct Vibrato<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> {
    source: S,
    delay_buffer: Vec<f64>,
    write_pos: usize,
    rate: Param,  // vibrato rate in Hz
    depth: Param, // pitch deviation in cents (100 cents = 1 semitone)
    lfo_phase: f64,
}

impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> Vibrato<SAMPLE_RATE, S> {
    /// Creates a new vibrato effect.
    ///
    /// # Arguments
    ///
    /// * `source` - Input audio signal
    /// * `rate` - Vibrato rate in Hz (typically 2-8 Hz)
    /// * `depth` - Pitch deviation in cents (typically 10-50 cents, where 100 cents = 1 semitone)
    ///
    /// The effect uses a delay line modulated by a sine wave LFO. The delay time
    /// variation creates the pitch shift effect.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, Vibrato};
    ///
    /// let audio = SineOscillator::<44100>::new(440.0);
    /// // 5 Hz vibrato with 20 cents depth
    /// let mut vibrato = Vibrato::new(audio, 5.0, 20.0);
    /// ```
    pub fn new(source: S, rate: impl Into<Param>, depth: impl Into<Param>) -> Self {
        // Maximum delay needed for the depth
        // For 50 cents (half semitone), we need about 50ms delay at most
        let max_delay_ms = 50.0;
        let buffer_size = ((max_delay_ms / 1000.0) * SAMPLE_RATE as f64) as usize + 1;

        Self {
            source,
            delay_buffer: vec![0.0; buffer_size],
            write_pos: 0,
            rate: rate.into(),
            depth: depth.into(),
            lfo_phase: 0.0,
        }
    }

    /// Creates a subtle vibrato effect suitable for vocals.
    ///
    /// Uses a rate of 5 Hz and depth of 15 cents.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, Vibrato};
    ///
    /// let audio = SineOscillator::<44100>::new(440.0);
    /// let mut vibrato = Vibrato::subtle(audio);
    /// ```
    pub fn subtle(source: S) -> Self {
        Self::new(source, 5.0, 15.0)
    }

    /// Creates a classic vibrato effect suitable for guitar.
    ///
    /// Uses a rate of 5.5 Hz and depth of 30 cents.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, Vibrato};
    ///
    /// let audio = SineOscillator::<44100>::new(440.0);
    /// let mut vibrato = Vibrato::guitar(audio);
    /// ```
    pub fn guitar(source: S) -> Self {
        Self::new(source, 5.5, 30.0)
    }

    /// Creates a wide vibrato effect.
    ///
    /// Uses a rate of 6 Hz and depth of 50 cents (half semitone).
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{SineOscillator, Vibrato};
    ///
    /// let audio = SineOscillator::<44100>::new(440.0);
    /// let mut vibrato = Vibrato::wide(audio);
    /// ```
    pub fn wide(source: S) -> Self {
        Self::new(source, 6.0, 50.0)
    }
}

impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> Signal for Vibrato<SAMPLE_RATE, S> {
    fn next_sample(&mut self) -> f64 {
        let input = self.source.next_sample();

        // Get parameter values
        let rate = self.rate.value().max(0.1);
        let depth = self.depth.value().max(0.0);

        // Update LFO phase
        let phase_increment = rate / SAMPLE_RATE as f64;
        self.lfo_phase += phase_increment;
        if self.lfo_phase >= 1.0 {
            self.lfo_phase -= 1.0;
        }

        // Generate sine LFO (-1 to 1)
        let lfo_value = (self.lfo_phase * 2.0 * std::f64::consts::PI).sin();

        // Convert depth from cents to delay time
        // Pitch shift formula: delay_time = (2^(cents/1200) - 1) * base_delay
        // For vibrato, we use a small base delay and modulate it
        // Approximation: cents to delay time in milliseconds
        // For small pitch shifts, delay_ms ≈ (cents / 100) * 10ms
        let depth_ms = (depth / 100.0) * 10.0;

        // Modulate delay time: center_delay ± depth
        let center_delay_ms = 5.0; // Center delay time
        let delay_ms = center_delay_ms + (lfo_value * depth_ms);
        let delay_samples = ((delay_ms / 1000.0) * SAMPLE_RATE as f64).max(0.0);

        // Write input to buffer
        self.delay_buffer[self.write_pos] = input;

        // Calculate read position with interpolation
        let read_pos_float = self.write_pos as f64 - delay_samples;
        let read_pos_float = if read_pos_float < 0.0 {
            read_pos_float + self.delay_buffer.len() as f64
        } else {
            read_pos_float
        };

        // Linear interpolation between samples
        let read_pos_int = read_pos_float.floor() as usize % self.delay_buffer.len();
        let read_pos_next = (read_pos_int + 1) % self.delay_buffer.len();
        let frac = read_pos_float.fract();

        let sample1 = self.delay_buffer[read_pos_int];
        let sample2 = self.delay_buffer[read_pos_next];
        let output = sample1 * (1.0 - frac) + sample2 * frac;

        // Advance write position
        self.write_pos = (self.write_pos + 1) % self.delay_buffer.len();

        output
    }
}

impl<const SAMPLE_RATE: u32, S: AudioSignal<SAMPLE_RATE>> AudioSignal<SAMPLE_RATE>
    for Vibrato<SAMPLE_RATE, S>
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConstantSignal;

    #[test]
    fn test_vibrato_creation() {
        let source = ConstantSignal::<44100>(0.5);
        let vibrato = Vibrato::new(source, 5.0, 20.0);
        assert_eq!(vibrato.lfo_phase, 0.0);
        assert!(!vibrato.delay_buffer.is_empty());
    }

    #[test]
    fn test_vibrato_processes_signal() {
        let source = ConstantSignal::<44100>(0.5);
        let mut vibrato = Vibrato::new(source, 5.0, 20.0);

        // Process some samples
        for _ in 0..100 {
            let sample = vibrato.next_sample();
            assert!(sample.is_finite());
        }
    }

    #[test]
    fn test_audio_signal_trait() {
        let source = ConstantSignal::<44100>(0.5);
        let vibrato = Vibrato::new(source, 5.0, 20.0);

        // Just verify it implements AudioSignal
        fn assert_audio_signal<T: AudioSignal<44100>>(_: T) {}
        assert_audio_signal(vibrato);
    }

    #[test]
    fn test_lfo_phase_wraps() {
        let source = ConstantSignal::<44100>(0.5);
        let mut vibrato = Vibrato::new(source, 5.0, 20.0);

        // Process enough samples to wrap the phase
        for _ in 0..44100 {
            vibrato.next_sample();
        }

        // Phase should be between 0 and 1
        assert!(vibrato.lfo_phase >= 0.0 && vibrato.lfo_phase < 1.0);
    }
}
