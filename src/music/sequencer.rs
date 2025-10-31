//! Musical sequencer for pattern-based playback.
//!
//! The `Sequencer` combines a `Metronome` (for timing) with one or more `Pattern`s
//! (for note data) to trigger musical events in sync with audio sample generation.

use super::{core::NoteEvent, metronome::Metronome, pattern::Pattern};

/// Playback state of the sequencer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayState {
    /// Sequencer is stopped
    Stopped,
    /// Sequencer is playing
    Playing,
}

/// A musical sequencer that plays patterns in time.
///
/// The sequencer combines timing (via `Metronome`) with musical content (via `Pattern`)
/// to trigger note events at the correct sample times. It maintains transport state
/// (play/stop) and handles pattern looping.
///
/// # Architecture
///
/// - **Metronome**: Provides sample-accurate timing and step advancement
/// - **Pattern**: Contains the musical events to play at each step
/// - **Sequencer**: Coordinates them, returning events when it's time to trigger them
///
/// # Usage Pattern
///
/// In your audio callback, call `tick()` once per sample. When `tick()` returns events,
/// trigger those notes on your synthesizer/voice allocator.
///
/// # Examples
///
/// ```
/// use earworm::music::{Sequencer, Pattern, Metronome};
/// use earworm::{NoteEvent, Pitch};
///
/// const SAMPLE_RATE: u32 = 44100;
///
/// // Create a pattern
/// let mut pattern = Pattern::new(16);
/// pattern.add_event(0, NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5)));
/// pattern.add_event(4, NoteEvent::from_pitch(Pitch::E, 4, 0.7, Some(0.5)));
///
/// // Create a sequencer at 120 BPM with 16th note steps
/// let mut sequencer = Sequencer::new(120.0, 4, SAMPLE_RATE);
/// sequencer.set_pattern(pattern);
/// sequencer.play();
///
/// // In your audio callback:
/// for _sample in 0..1000 {
///     if let Some(events) = sequencer.tick() {
///         for event in events {
///             println!("Trigger note!");
///             // voice_allocator.note_on(event.note, event.velocity);
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Sequencer {
    /// The metronome that provides timing
    metronome: Metronome,
    /// The currently active pattern (if any)
    pattern: Option<Pattern>,
    /// Current playback state
    state: PlayState,
}

impl Sequencer {
    /// Creates a new sequencer with the given tempo and step resolution.
    ///
    /// The sequencer starts in `Stopped` state with no pattern loaded.
    ///
    /// # Arguments
    ///
    /// * `bpm` - Tempo in beats per minute
    /// * `steps_per_beat` - Step subdivision (4 = 16th notes, 2 = 8th notes, etc.)
    /// * `sample_rate` - Audio sample rate in Hz
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::Sequencer;
    ///
    /// // 120 BPM with 16th note resolution
    /// let sequencer = Sequencer::new(120.0, 4, 44100);
    /// ```
    pub fn new(bpm: f64, steps_per_beat: u32, sample_rate: u32) -> Self {
        Self {
            metronome: Metronome::new(bpm, steps_per_beat, sample_rate),
            pattern: None,
            state: PlayState::Stopped,
        }
    }

    /// Sets the active pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::{Sequencer, Pattern};
    ///
    /// let mut sequencer = Sequencer::new(120.0, 4, 44100);
    /// let pattern = Pattern::new(16);
    /// sequencer.set_pattern(pattern);
    /// ```
    pub fn set_pattern(&mut self, pattern: Pattern) {
        self.pattern = Some(pattern);
    }

    /// Returns a reference to the current pattern, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::{Sequencer, Pattern};
    ///
    /// let mut sequencer = Sequencer::new(120.0, 4, 44100);
    /// assert!(sequencer.pattern().is_none());
    ///
    /// sequencer.set_pattern(Pattern::new(16));
    /// assert!(sequencer.pattern().is_some());
    /// ```
    pub fn pattern(&self) -> Option<&Pattern> {
        self.pattern.as_ref()
    }

    /// Removes the current pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::{Sequencer, Pattern};
    ///
    /// let mut sequencer = Sequencer::new(120.0, 4, 44100);
    /// sequencer.set_pattern(Pattern::new(16));
    /// sequencer.clear_pattern();
    /// assert!(sequencer.pattern().is_none());
    /// ```
    pub fn clear_pattern(&mut self) {
        self.pattern = None;
    }

    /// Starts playback.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::Sequencer;
    ///
    /// let mut sequencer = Sequencer::new(120.0, 4, 44100);
    /// sequencer.play();
    /// assert!(sequencer.is_playing());
    /// ```
    pub fn play(&mut self) {
        self.state = PlayState::Playing;
    }

    /// Stops playback.
    ///
    /// The sequencer position is maintained - call `reset()` to return to step 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::Sequencer;
    ///
    /// let mut sequencer = Sequencer::new(120.0, 4, 44100);
    /// sequencer.play();
    /// sequencer.stop();
    /// assert!(!sequencer.is_playing());
    /// ```
    pub fn stop(&mut self) {
        self.state = PlayState::Stopped;
    }

    /// Resets the sequencer to step 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::Sequencer;
    ///
    /// let mut sequencer = Sequencer::new(120.0, 4, 44100);
    /// sequencer.reset();
    /// ```
    pub fn reset(&mut self) {
        self.metronome.reset();
    }

    /// Returns true if the sequencer is currently playing.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::Sequencer;
    ///
    /// let mut sequencer = Sequencer::new(120.0, 4, 44100);
    /// assert!(!sequencer.is_playing());
    ///
    /// sequencer.play();
    /// assert!(sequencer.is_playing());
    /// ```
    pub fn is_playing(&self) -> bool {
        self.state == PlayState::Playing
    }

    /// Returns the current playback state.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::{Sequencer, PlayState};
    ///
    /// let sequencer = Sequencer::new(120.0, 4, 44100);
    /// assert_eq!(sequencer.state(), PlayState::Stopped);
    /// ```
    pub fn state(&self) -> PlayState {
        self.state
    }

    /// Returns the current step number.
    ///
    /// This is the absolute step count from when the sequencer was created or last reset.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::Sequencer;
    ///
    /// let sequencer = Sequencer::new(120.0, 4, 44100);
    /// assert_eq!(sequencer.current_step(), 0);
    /// ```
    pub fn current_step(&self) -> u64 {
        self.metronome.current_step()
    }

    /// Returns the current step within the pattern (wraps at pattern length).
    ///
    /// Returns `None` if no pattern is loaded.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::{Sequencer, Pattern};
    ///
    /// let mut sequencer = Sequencer::new(120.0, 4, 44100);
    /// assert!(sequencer.pattern_step().is_none());
    ///
    /// sequencer.set_pattern(Pattern::new(16));
    /// assert_eq!(sequencer.pattern_step(), Some(0));
    /// ```
    pub fn pattern_step(&self) -> Option<usize> {
        self.pattern
            .as_ref()
            .map(|p| (self.metronome.current_step() % p.length() as u64) as usize)
    }

    /// Sets the tempo in BPM.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::Sequencer;
    ///
    /// let mut sequencer = Sequencer::new(120.0, 4, 44100);
    /// sequencer.set_tempo(140.0);
    /// assert_eq!(sequencer.tempo(), 140.0);
    /// ```
    pub fn set_tempo(&mut self, bpm: f64) {
        self.metronome.set_tempo(bpm);
    }

    /// Returns the current tempo in BPM.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::Sequencer;
    ///
    /// let sequencer = Sequencer::new(120.0, 4, 44100);
    /// assert_eq!(sequencer.tempo(), 120.0);
    /// ```
    pub fn tempo(&self) -> f64 {
        self.metronome.tempo()
    }

    /// Advances the sequencer by one sample.
    ///
    /// If the sequencer is playing and a step boundary is crossed, returns the events
    /// that should be triggered at this step. Otherwise returns `None`.
    ///
    /// # Returns
    ///
    /// - `Some(Vec<NoteEvent>)` - Events to trigger at this step
    /// - `None` - No events to trigger (not on a step boundary, stopped, or empty step)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::{Sequencer, Pattern};
    /// use earworm::{NoteEvent, Pitch};
    ///
    /// let mut sequencer = Sequencer::new(120.0, 4, 44100);
    /// let mut pattern = Pattern::new(16);
    /// pattern.add_event(0, NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5)));
    ///
    /// sequencer.set_pattern(pattern);
    /// sequencer.play();
    ///
    /// // Tick until we hit the first step
    /// let mut events_found = false;
    /// for _ in 0..10000 {
    ///     if let Some(events) = sequencer.tick() {
    ///         assert_eq!(events.len(), 1);
    ///         events_found = true;
    ///         break;
    ///     }
    /// }
    /// assert!(events_found);
    /// ```
    pub fn tick(&mut self) -> Option<Vec<NoteEvent>> {
        // If stopped, don't advance
        if self.state != PlayState::Playing {
            return None;
        }

        // If no pattern, just advance metronome but return no events
        let pattern = self.pattern.as_ref()?;

        // Advance metronome - returns true on step boundary
        if self.metronome.tick() {
            // Get current step within pattern (with wrapping)
            // current_step() has already been incremented by tick(), so subtract 1
            let step = ((self.metronome.current_step() - 1) % pattern.length() as u64) as usize;

            // Get events at this step and copy them (NoteEvent is Copy)
            let events: Vec<NoteEvent> =
                pattern.events_at_step(step).into_iter().copied().collect();

            if !events.is_empty() {
                return Some(events);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music::core::{NoteEvent, Pitch};

    const SAMPLE_RATE: u32 = 44100;

    #[test]
    fn test_creation() {
        let sequencer = Sequencer::new(120.0, 4, SAMPLE_RATE);
        assert_eq!(sequencer.state(), PlayState::Stopped);
        assert!(!sequencer.is_playing());
        assert_eq!(sequencer.current_step(), 0);
        assert!(sequencer.pattern().is_none());
        assert_eq!(sequencer.tempo(), 120.0);
    }

    #[test]
    fn test_transport_controls() {
        let mut sequencer = Sequencer::new(120.0, 4, SAMPLE_RATE);

        assert!(!sequencer.is_playing());

        sequencer.play();
        assert!(sequencer.is_playing());
        assert_eq!(sequencer.state(), PlayState::Playing);

        sequencer.stop();
        assert!(!sequencer.is_playing());
        assert_eq!(sequencer.state(), PlayState::Stopped);
    }

    #[test]
    fn test_pattern_management() {
        let mut sequencer = Sequencer::new(120.0, 4, SAMPLE_RATE);

        assert!(sequencer.pattern().is_none());

        let pattern = Pattern::new(16);
        sequencer.set_pattern(pattern);
        assert!(sequencer.pattern().is_some());
        assert_eq!(sequencer.pattern().unwrap().length(), 16);

        sequencer.clear_pattern();
        assert!(sequencer.pattern().is_none());
    }

    #[test]
    fn test_tick_when_stopped() {
        let mut sequencer = Sequencer::new(120.0, 4, SAMPLE_RATE);
        let mut pattern = Pattern::new(16);
        pattern.add_event(0, NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5)));
        sequencer.set_pattern(pattern);

        // Sequencer is stopped, tick should return None
        for _ in 0..10000 {
            assert!(sequencer.tick().is_none());
        }
    }

    #[test]
    fn test_tick_triggers_events() {
        let mut sequencer = Sequencer::new(120.0, 4, SAMPLE_RATE);
        let mut pattern = Pattern::new(16);
        pattern.add_event(0, NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5)));

        sequencer.set_pattern(pattern);
        sequencer.play();

        // Tick until we hit the first step
        let mut events_found = false;
        for _ in 0..10000 {
            if let Some(events) = sequencer.tick() {
                assert_eq!(events.len(), 1);
                events_found = true;
                break;
            }
        }
        assert!(events_found);
    }

    #[test]
    fn test_pattern_looping() {
        let mut sequencer = Sequencer::new(120.0, 4, SAMPLE_RATE);
        let mut pattern = Pattern::new(4); // Short pattern for faster testing
        pattern.add_event(0, NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5)));

        sequencer.set_pattern(pattern);
        sequencer.play();

        // Should trigger event at step 0, then again when it loops
        let mut trigger_count = 0;
        for _ in 0..50000 {
            if let Some(events) = sequencer.tick() {
                assert_eq!(events.len(), 1); // Should have one event
                trigger_count += 1;
                if trigger_count >= 3 {
                    break;
                }
            }
        }

        assert!(
            trigger_count >= 3,
            "Pattern should loop and trigger multiple times"
        );
    }

    #[test]
    fn test_multiple_events_per_step() {
        let mut sequencer = Sequencer::new(120.0, 4, SAMPLE_RATE);
        let mut pattern = Pattern::new(16);

        // Add multiple events at step 0 (chord)
        pattern.add_event(0, NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5)));
        pattern.add_event(0, NoteEvent::from_pitch(Pitch::E, 4, 0.7, Some(0.5)));
        pattern.add_event(0, NoteEvent::from_pitch(Pitch::G, 4, 0.6, Some(0.5)));

        sequencer.set_pattern(pattern);
        sequencer.play();

        // Tick until we hit the first step
        for _ in 0..10000 {
            if let Some(events) = sequencer.tick() {
                assert_eq!(events.len(), 3, "Should trigger all three notes");
                break;
            }
        }
    }

    #[test]
    fn test_reset() {
        let mut sequencer = Sequencer::new(120.0, 4, SAMPLE_RATE);
        let mut pattern = Pattern::new(4);
        pattern.add_event(0, NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5)));

        sequencer.set_pattern(pattern);
        sequencer.play();

        // Advance past first step
        for _ in 0..20000 {
            sequencer.tick();
        }

        assert!(sequencer.current_step() > 0);

        sequencer.reset();
        assert_eq!(sequencer.current_step(), 0);
    }

    #[test]
    fn test_tempo_change() {
        let mut sequencer = Sequencer::new(120.0, 4, SAMPLE_RATE);

        assert_eq!(sequencer.tempo(), 120.0);

        sequencer.set_tempo(140.0);
        assert_eq!(sequencer.tempo(), 140.0);
    }

    #[test]
    fn test_pattern_step_wrapping() {
        let mut sequencer = Sequencer::new(120.0, 4, SAMPLE_RATE);
        let pattern = Pattern::new(4);

        sequencer.set_pattern(pattern);
        sequencer.play();

        // Advance through multiple pattern loops
        for _ in 0..30000 {
            sequencer.tick();
        }

        // Pattern step should always be 0-3
        if let Some(step) = sequencer.pattern_step() {
            assert!(step < 4, "Pattern step should wrap at pattern length");
        }
    }

    #[test]
    fn test_no_pattern_tick() {
        let mut sequencer = Sequencer::new(120.0, 4, SAMPLE_RATE);
        sequencer.play();

        // Without a pattern, tick should always return None
        for _ in 0..10000 {
            assert!(sequencer.tick().is_none());
        }
    }
}
