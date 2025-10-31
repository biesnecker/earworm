//! Musical metronome for sample-accurate timing.
//!
//! The `Metronome` provides sample-accurate timing for sequencers and rhythm-based
//! musical applications. It converts musical time (beats, steps) to audio time (samples).

/// A sample-accurate musical metronome.
///
/// The metronome tracks musical time in beats and subdivisions (steps), converting
/// between musical time and audio sample time. It's the timing foundation for
/// sequencers and pattern-based music generation.
///
/// # Musical Time Concepts
///
/// - **BPM** (Beats Per Minute): The tempo, e.g., 120 BPM = 2 beats per second
/// - **Beat**: One quarter note (typically)
/// - **Step**: A subdivision of a beat (configured via `steps_per_beat`)
///   - `steps_per_beat = 4` → 16th notes
///   - `steps_per_beat = 2` → 8th notes
///   - `steps_per_beat = 1` → quarter notes
///
/// # Sample Accuracy
///
/// The metronome uses floating-point accumulation to maintain sample-accurate
/// timing without drift. Each call to `tick()` advances by exactly one sample,
/// and step boundaries are detected with sub-sample precision.
///
/// # Examples
///
/// ```
/// use earworm::music::Metronome;
///
/// const SAMPLE_RATE: u32 = 44100;
///
/// // Create a metronome at 120 BPM with 16th note steps (4 steps per beat)
/// let mut metronome = Metronome::new(120.0, 4, SAMPLE_RATE);
///
/// // Process audio samples
/// let mut step_count = 0;
/// for _ in 0..44100 {
///     if metronome.tick() {
///         // A step boundary was crossed
///         step_count += 1;
///         println!("Step {}", metronome.current_step());
///     }
/// }
///
/// // At 120 BPM with 4 steps per beat:
/// // - 2 beats per second = 8 steps per second
/// // - In 1 second (44100 samples) we should get ~8 steps
/// assert_eq!(step_count, 8);
/// ```
#[derive(Debug, Clone)]
pub struct Metronome {
    /// Tempo in beats per minute
    bpm: f64,
    /// Number of steps per beat (e.g., 4 = 16th notes)
    steps_per_beat: u32,
    /// Sample rate in Hz
    sample_rate: u32,
    /// Number of samples per step (calculated from BPM and steps_per_beat)
    samples_per_step: f64,
    /// Accumulated sample count (fractional for accuracy)
    sample_accumulator: f64,
    /// Current step number (wraps based on pattern length)
    current_step: u64,
}

impl Metronome {
    /// Creates a new metronome with the given tempo and resolution.
    ///
    /// # Arguments
    ///
    /// * `bpm` - Tempo in beats per minute (must be > 0)
    /// * `steps_per_beat` - Number of steps per beat (must be > 0)
    ///   - 1 = quarter notes
    ///   - 2 = eighth notes
    ///   - 4 = sixteenth notes
    /// * `sample_rate` - Audio sample rate in Hz
    ///
    /// # Panics
    ///
    /// Panics if `bpm` or `steps_per_beat` is <= 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::Metronome;
    ///
    /// // 120 BPM with 16th note resolution at 44.1kHz
    /// let metronome = Metronome::new(120.0, 4, 44100);
    /// ```
    pub fn new(bpm: f64, steps_per_beat: u32, sample_rate: u32) -> Self {
        assert!(bpm > 0.0, "BPM must be greater than 0");
        assert!(steps_per_beat > 0, "steps_per_beat must be greater than 0");

        let samples_per_step = Self::calculate_samples_per_step(bpm, steps_per_beat, sample_rate);

        Self {
            bpm,
            steps_per_beat,
            sample_rate,
            samples_per_step,
            sample_accumulator: 0.0,
            current_step: 0,
        }
    }

    /// Calculates the number of samples per step based on tempo and resolution.
    fn calculate_samples_per_step(bpm: f64, steps_per_beat: u32, sample_rate: u32) -> f64 {
        // BPM = beats per minute
        // beats_per_second = BPM / 60
        // steps_per_second = beats_per_second * steps_per_beat
        // samples_per_step = sample_rate / steps_per_second
        let beats_per_second = bpm / 60.0;
        let steps_per_second = beats_per_second * steps_per_beat as f64;
        sample_rate as f64 / steps_per_second
    }

    /// Advances the metronome by one sample.
    ///
    /// Returns `true` if a step boundary was crossed, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::Metronome;
    ///
    /// let mut metronome = Metronome::new(120.0, 4, 44100);
    ///
    /// // Process samples until we hit the first step
    /// let mut samples = 0;
    /// while !metronome.tick() {
    ///     samples += 1;
    /// }
    ///
    /// // Should take approximately samples_per_step samples
    /// // At 120 BPM, 4 steps/beat: ~5512 samples per step
    /// assert!(samples > 5500 && samples < 5525);
    /// ```
    pub fn tick(&mut self) -> bool {
        self.sample_accumulator += 1.0;

        if self.sample_accumulator >= self.samples_per_step {
            self.sample_accumulator -= self.samples_per_step;
            self.current_step = self.current_step.wrapping_add(1);
            true
        } else {
            false
        }
    }

    /// Returns the current step number.
    ///
    /// The step counter increments indefinitely and wraps at `u64::MAX`.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::Metronome;
    ///
    /// let mut metronome = Metronome::new(120.0, 4, 44100);
    /// assert_eq!(metronome.current_step(), 0);
    ///
    /// // Advance until first step
    /// while !metronome.tick() {}
    /// assert_eq!(metronome.current_step(), 1);
    /// ```
    pub fn current_step(&self) -> u64 {
        self.current_step
    }

    /// Resets the metronome to step 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::Metronome;
    ///
    /// let mut metronome = Metronome::new(120.0, 4, 44100);
    ///
    /// // Advance a few steps
    /// for _ in 0..100000 {
    ///     metronome.tick();
    /// }
    ///
    /// metronome.reset();
    /// assert_eq!(metronome.current_step(), 0);
    /// ```
    pub fn reset(&mut self) {
        self.sample_accumulator = 0.0;
        self.current_step = 0;
    }

    /// Sets the tempo in BPM.
    ///
    /// # Arguments
    ///
    /// * `bpm` - New tempo in beats per minute (must be > 0)
    ///
    /// # Panics
    ///
    /// Panics if `bpm` is <= 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::Metronome;
    ///
    /// let mut metronome = Metronome::new(120.0, 4, 44100);
    /// metronome.set_tempo(140.0);
    /// ```
    pub fn set_tempo(&mut self, bpm: f64) {
        assert!(bpm > 0.0, "BPM must be greater than 0");
        self.bpm = bpm;
        self.samples_per_step =
            Self::calculate_samples_per_step(bpm, self.steps_per_beat, self.sample_rate);
    }

    /// Returns the current tempo in BPM.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::Metronome;
    ///
    /// let metronome = Metronome::new(120.0, 4, 44100);
    /// assert_eq!(metronome.tempo(), 120.0);
    /// ```
    pub fn tempo(&self) -> f64 {
        self.bpm
    }

    /// Returns the number of steps per beat.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::Metronome;
    ///
    /// let metronome = Metronome::new(120.0, 4, 44100);
    /// assert_eq!(metronome.steps_per_beat(), 4);
    /// ```
    pub fn steps_per_beat(&self) -> u32 {
        self.steps_per_beat
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: u32 = 44100;

    #[test]
    fn test_creation() {
        let metronome = Metronome::new(120.0, 4, SAMPLE_RATE);
        assert_eq!(metronome.current_step(), 0);
        assert_eq!(metronome.tempo(), 120.0);
        assert_eq!(metronome.steps_per_beat(), 4);
    }

    #[test]
    #[should_panic(expected = "BPM must be greater than 0")]
    fn test_invalid_bpm() {
        Metronome::new(0.0, 4, SAMPLE_RATE);
    }

    #[test]
    #[should_panic(expected = "steps_per_beat must be greater than 0")]
    fn test_invalid_steps_per_beat() {
        Metronome::new(120.0, 0, SAMPLE_RATE);
    }

    #[test]
    fn test_tick_advances_step() {
        let mut metronome = Metronome::new(120.0, 4, SAMPLE_RATE);

        // At 120 BPM, 4 steps per beat:
        // 120 BPM = 2 beats/sec
        // 2 beats/sec * 4 steps/beat = 8 steps/sec
        // 44100 samples/sec / 8 steps/sec = 5512.5 samples/step

        let mut ticks = 0;
        while !metronome.tick() {
            ticks += 1;
        }

        // Should take approximately 5512 samples to reach first step
        assert!((5512..=5513).contains(&ticks));
        assert_eq!(metronome.current_step(), 1);
    }

    #[test]
    fn test_multiple_steps() {
        let mut metronome = Metronome::new(120.0, 4, SAMPLE_RATE);

        let mut step_count = 0;
        // Run for 1 second worth of samples
        for _ in 0..SAMPLE_RATE {
            if metronome.tick() {
                step_count += 1;
            }
        }

        // At 120 BPM with 4 steps per beat, we should get 8 steps per second
        assert_eq!(step_count, 8);
        assert_eq!(metronome.current_step(), 8);
    }

    #[test]
    fn test_reset() {
        let mut metronome = Metronome::new(120.0, 4, SAMPLE_RATE);

        // Advance several steps
        for _ in 0..100000 {
            metronome.tick();
        }

        assert!(metronome.current_step() > 0);

        metronome.reset();
        assert_eq!(metronome.current_step(), 0);
    }

    #[test]
    fn test_set_tempo() {
        let mut metronome = Metronome::new(120.0, 4, SAMPLE_RATE);

        // At 120 BPM: 8 steps per second
        let mut step_count_120 = 0;
        for _ in 0..SAMPLE_RATE {
            if metronome.tick() {
                step_count_120 += 1;
            }
        }
        assert_eq!(step_count_120, 8);

        // Change to 60 BPM: should be 4 steps per second
        metronome.reset();
        metronome.set_tempo(60.0);
        assert_eq!(metronome.tempo(), 60.0);

        let mut step_count_60 = 0;
        for _ in 0..SAMPLE_RATE {
            if metronome.tick() {
                step_count_60 += 1;
            }
        }
        assert_eq!(step_count_60, 4);
    }

    #[test]
    fn test_different_subdivisions() {
        // Quarter notes (1 step per beat)
        let mut metronome_quarters = Metronome::new(120.0, 1, SAMPLE_RATE);
        let mut quarter_steps = 0;
        for _ in 0..SAMPLE_RATE {
            if metronome_quarters.tick() {
                quarter_steps += 1;
            }
        }
        // 120 BPM = 2 beats per second
        assert_eq!(quarter_steps, 2);

        // Eighth notes (2 steps per beat)
        let mut metronome_eighths = Metronome::new(120.0, 2, SAMPLE_RATE);
        let mut eighth_steps = 0;
        for _ in 0..SAMPLE_RATE {
            if metronome_eighths.tick() {
                eighth_steps += 1;
            }
        }
        // 2 beats per second * 2 steps per beat = 4 steps per second
        assert_eq!(eighth_steps, 4);

        // Sixteenth notes (4 steps per beat)
        let mut metronome_sixteenths = Metronome::new(120.0, 4, SAMPLE_RATE);
        let mut sixteenth_steps = 0;
        for _ in 0..SAMPLE_RATE {
            if metronome_sixteenths.tick() {
                sixteenth_steps += 1;
            }
        }
        // 2 beats per second * 4 steps per beat = 8 steps per second
        assert_eq!(sixteenth_steps, 8);
    }

    #[test]
    fn test_timing_accuracy_over_long_period() {
        let mut metronome = Metronome::new(120.0, 4, SAMPLE_RATE);

        // Run for 10 seconds
        let mut step_count = 0;
        for _ in 0..(SAMPLE_RATE * 10) {
            if metronome.tick() {
                step_count += 1;
            }
        }

        // Should get exactly 80 steps (8 per second * 10 seconds)
        // This tests that there's no drift with the fractional accumulator
        assert_eq!(step_count, 80);
        assert_eq!(metronome.current_step(), 80);
    }

    #[test]
    fn test_different_sample_rates() {
        // Test that different sample rates produce correct step counts
        for sample_rate in [44100, 48000, 96000] {
            let mut metronome = Metronome::new(120.0, 4, sample_rate);
            let mut step_count = 0;

            // Run for 1 second
            for _ in 0..sample_rate {
                if metronome.tick() {
                    step_count += 1;
                }
            }

            // Should always get 8 steps per second regardless of sample rate
            assert_eq!(step_count, 8);
        }
    }

    #[test]
    fn test_step_wrapping() {
        let mut metronome = Metronome::new(120.0, 4, SAMPLE_RATE);

        // Set step counter near max
        metronome.current_step = u64::MAX - 2;

        // Advance a few steps and ensure it wraps properly
        while !metronome.tick() {}
        assert_eq!(metronome.current_step(), u64::MAX - 1);

        while !metronome.tick() {}
        assert_eq!(metronome.current_step(), u64::MAX);

        while !metronome.tick() {}
        assert_eq!(metronome.current_step(), 0); // Wrapped
    }
}
