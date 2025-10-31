//! Step-based musical patterns.
//!
//! A `Pattern` represents a sequence of musical events (notes) arranged on a timeline
//! divided into discrete steps. This is the foundation for step sequencers, drum machines,
//! and pattern-based composition.

use super::core::NoteEvent;

/// A step-based musical pattern.
///
/// A pattern is a collection of note events placed at specific step positions.
/// Multiple events can occur at the same step, and steps can be empty.
///
/// # Timing Independence
///
/// **Important**: Patterns are timing-agnostic. They contain only step numbers (0, 1, 2...)
/// with no concept of BPM, tempo, or musical duration. The *interpretation* of what
/// each step means musically (16th notes, 8th notes, etc.) is determined by the
/// `Metronome` and `Sequencer` that play the pattern.
///
/// For example, the same 16-step pattern could represent:
/// - One bar of 16th notes (Metronome with `steps_per_beat=4`)
/// - One bar of 8th notes (Metronome with `steps_per_beat=2`)
/// - Four bars of quarter notes (Metronome with `steps_per_beat=1`)
///
/// # Pattern Length
///
/// The pattern has a fixed length in steps. When played by a sequencer, it will
/// typically loop back to step 0 after reaching the end.
///
/// # Examples
///
/// ```
/// use earworm::{NoteEvent, Pitch};
/// use earworm::music::Pattern;
///
/// // Create a 16-step pattern
/// // (Musical timing determined by Metronome when played)
/// let mut pattern = Pattern::new(16);
/// pattern.set_name("Kick Pattern");
///
/// // Add kick drum on steps 0, 4, 8, 12
/// pattern.add_event(0, NoteEvent::from_pitch(Pitch::C, 2, 0.8, Some(0.1)));
/// pattern.add_event(4, NoteEvent::from_pitch(Pitch::C, 2, 0.8, Some(0.1)));
/// pattern.add_event(8, NoteEvent::from_pitch(Pitch::C, 2, 0.8, Some(0.1)));
/// pattern.add_event(12, NoteEvent::from_pitch(Pitch::C, 2, 0.8, Some(0.1)));
///
/// // Query events at a step
/// let events = pattern.events_at_step(0);
/// assert_eq!(events.len(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct Pattern {
    /// Pattern name (optional)
    name: Option<String>,
    /// Pattern description (optional)
    description: Option<String>,
    /// Length of the pattern in steps
    length: usize,
    /// Events stored as (step_index, NoteEvent) tuples
    /// Invariant: step_index < length
    events: Vec<(usize, NoteEvent)>,
}

impl Pattern {
    /// Creates a new empty pattern with the given length.
    ///
    /// The pattern contains N numbered steps (0 through length-1) but has no
    /// inherent timing information. Musical timing is determined by the `Metronome`
    /// that plays the pattern.
    ///
    /// # Arguments
    ///
    /// * `length` - Number of steps in the pattern (must be > 0)
    ///
    /// # Panics
    ///
    /// Panics if `length` is 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::Pattern;
    ///
    /// // Create a 16-step pattern (steps 0-15)
    /// let pattern = Pattern::new(16);
    /// assert_eq!(pattern.length(), 16);
    /// assert_eq!(pattern.event_count(), 0);
    /// ```
    pub fn new(length: usize) -> Self {
        assert!(length > 0, "Pattern length must be greater than 0");
        Self {
            name: None,
            description: None,
            length,
            events: Vec::new(),
        }
    }

    /// Sets the pattern name.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::Pattern;
    ///
    /// let mut pattern = Pattern::new(16);
    /// pattern.set_name("Kick Pattern");
    /// assert_eq!(pattern.name(), Some("Kick Pattern"));
    /// ```
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Returns the pattern name, if set.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::Pattern;
    ///
    /// let mut pattern = Pattern::new(16);
    /// assert_eq!(pattern.name(), None);
    ///
    /// pattern.set_name("Bass Line");
    /// assert_eq!(pattern.name(), Some("Bass Line"));
    /// ```
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the pattern description.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::Pattern;
    ///
    /// let mut pattern = Pattern::new(16);
    /// pattern.set_description("Main drum loop for verse");
    /// ```
    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = Some(description.into());
    }

    /// Returns the pattern description, if set.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::Pattern;
    ///
    /// let mut pattern = Pattern::new(16);
    /// assert_eq!(pattern.description(), None);
    ///
    /// pattern.set_description("Intro melody");
    /// assert_eq!(pattern.description(), Some("Intro melody"));
    /// ```
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns the pattern length in steps.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::music::Pattern;
    ///
    /// let pattern = Pattern::new(32);
    /// assert_eq!(pattern.length(), 32);
    /// ```
    pub fn length(&self) -> usize {
        self.length
    }

    /// Returns the total number of events in the pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{NoteEvent, Pitch};
    /// use earworm::music::Pattern;
    ///
    /// let mut pattern = Pattern::new(16);
    /// assert_eq!(pattern.event_count(), 0);
    ///
    /// pattern.add_event(0, NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.1)));
    /// assert_eq!(pattern.event_count(), 1);
    /// ```
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Adds an event at the specified step.
    ///
    /// Multiple events can be added to the same step.
    ///
    /// # Arguments
    ///
    /// * `step` - Step index (0-based, must be < pattern length)
    /// * `event` - The note event to add
    ///
    /// # Panics
    ///
    /// Panics if `step` >= pattern length.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{NoteEvent, Pitch};
    /// use earworm::music::Pattern;
    ///
    /// let mut pattern = Pattern::new(16);
    /// pattern.add_event(0, NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5)));
    /// pattern.add_event(4, NoteEvent::from_pitch(Pitch::E, 4, 0.7, Some(0.5)));
    /// ```
    pub fn add_event(&mut self, step: usize, event: NoteEvent) {
        assert!(
            step < self.length,
            "Step index {} out of bounds (pattern length is {})",
            step,
            self.length
        );
        self.events.push((step, event));
    }

    /// Removes all events at the specified step.
    ///
    /// # Arguments
    ///
    /// * `step` - Step index to clear
    ///
    /// # Returns
    ///
    /// The number of events removed.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{NoteEvent, Pitch};
    /// use earworm::music::Pattern;
    ///
    /// let mut pattern = Pattern::new(16);
    /// pattern.add_event(0, NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5)));
    /// pattern.add_event(0, NoteEvent::from_pitch(Pitch::E, 4, 0.7, Some(0.5)));
    ///
    /// let removed = pattern.clear_step(0);
    /// assert_eq!(removed, 2);
    /// assert_eq!(pattern.event_count(), 0);
    /// ```
    pub fn clear_step(&mut self, step: usize) -> usize {
        let original_len = self.events.len();
        self.events.retain(|(s, _)| *s != step);
        original_len - self.events.len()
    }

    /// Clears all events from the pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{NoteEvent, Pitch};
    /// use earworm::music::Pattern;
    ///
    /// let mut pattern = Pattern::new(16);
    /// pattern.add_event(0, NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5)));
    /// pattern.add_event(4, NoteEvent::from_pitch(Pitch::E, 4, 0.7, Some(0.5)));
    ///
    /// pattern.clear();
    /// assert_eq!(pattern.event_count(), 0);
    /// ```
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Returns all events at the specified step.
    ///
    /// # Arguments
    ///
    /// * `step` - Step index to query
    ///
    /// # Returns
    ///
    /// A slice of `NoteEvent`s occurring at this step (may be empty).
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{NoteEvent, Pitch};
    /// use earworm::music::Pattern;
    ///
    /// let mut pattern = Pattern::new(16);
    /// pattern.add_event(0, NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5)));
    /// pattern.add_event(0, NoteEvent::from_pitch(Pitch::E, 4, 0.7, Some(0.5)));
    ///
    /// let events = pattern.events_at_step(0);
    /// assert_eq!(events.len(), 2);
    ///
    /// let no_events = pattern.events_at_step(1);
    /// assert_eq!(no_events.len(), 0);
    /// ```
    pub fn events_at_step(&self, step: usize) -> Vec<&NoteEvent> {
        self.events
            .iter()
            .filter(|(s, _)| *s == step)
            .map(|(_, event)| event)
            .collect()
    }

    /// Returns an iterator over all (step, event) pairs in the pattern.
    ///
    /// Events are returned in the order they were added, not sorted by step.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{NoteEvent, Pitch};
    /// use earworm::music::Pattern;
    ///
    /// let mut pattern = Pattern::new(16);
    /// pattern.add_event(0, NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5)));
    /// pattern.add_event(4, NoteEvent::from_pitch(Pitch::E, 4, 0.7, Some(0.5)));
    ///
    /// for (step, _event) in pattern.events() {
    ///     println!("Event at step {}", step);
    /// }
    /// ```
    pub fn events(&self) -> impl Iterator<Item = (usize, &NoteEvent)> {
        self.events.iter().map(|(step, event)| (*step, event))
    }

    /// Changes the pattern length.
    ///
    /// If the new length is shorter than the current length, events beyond the
    /// new length are removed. If longer, no events are added.
    ///
    /// # Arguments
    ///
    /// * `new_length` - New pattern length in steps (must be > 0)
    ///
    /// # Panics
    ///
    /// Panics if `new_length` is 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{NoteEvent, Pitch};
    /// use earworm::music::Pattern;
    ///
    /// let mut pattern = Pattern::new(16);
    /// pattern.add_event(0, NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5)));
    /// pattern.add_event(15, NoteEvent::from_pitch(Pitch::E, 4, 0.7, Some(0.5)));
    ///
    /// // Shorten pattern - event at step 15 is removed
    /// pattern.set_length(8);
    /// assert_eq!(pattern.length(), 8);
    /// assert_eq!(pattern.event_count(), 1);
    /// ```
    pub fn set_length(&mut self, new_length: usize) {
        assert!(new_length > 0, "Pattern length must be greater than 0");
        self.length = new_length;
        self.events.retain(|(step, _)| *step < new_length);
    }

    /// Returns true if the pattern has no events.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{NoteEvent, Pitch};
    /// use earworm::music::Pattern;
    ///
    /// let mut pattern = Pattern::new(16);
    /// assert!(pattern.is_empty());
    ///
    /// pattern.add_event(0, NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5)));
    /// assert!(!pattern.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music::core::Pitch;

    #[test]
    fn test_creation() {
        let pattern = Pattern::new(16);
        assert_eq!(pattern.length(), 16);
        assert_eq!(pattern.event_count(), 0);
        assert!(pattern.is_empty());
        assert_eq!(pattern.name(), None);
        assert_eq!(pattern.description(), None);
    }

    #[test]
    #[should_panic(expected = "Pattern length must be greater than 0")]
    fn test_invalid_length() {
        Pattern::new(0);
    }

    #[test]
    fn test_metadata() {
        let mut pattern = Pattern::new(16);

        pattern.set_name("Test Pattern");
        assert_eq!(pattern.name(), Some("Test Pattern"));

        pattern.set_description("A test pattern");
        assert_eq!(pattern.description(), Some("A test pattern"));
    }

    #[test]
    fn test_add_event() {
        let mut pattern = Pattern::new(16);
        let event = NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5));

        pattern.add_event(0, event);
        assert_eq!(pattern.event_count(), 1);
        assert!(!pattern.is_empty());
    }

    #[test]
    #[should_panic(expected = "Step index 16 out of bounds")]
    fn test_add_event_out_of_bounds() {
        let mut pattern = Pattern::new(16);
        let event = NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5));
        pattern.add_event(16, event); // Should panic
    }

    #[test]
    fn test_multiple_events_same_step() {
        let mut pattern = Pattern::new(16);
        let event1 = NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5));
        let event2 = NoteEvent::from_pitch(Pitch::E, 4, 0.7, Some(0.5));
        let event3 = NoteEvent::from_pitch(Pitch::G, 4, 0.6, Some(0.5));

        pattern.add_event(0, event1);
        pattern.add_event(0, event2);
        pattern.add_event(0, event3);

        assert_eq!(pattern.event_count(), 3);
        assert_eq!(pattern.events_at_step(0).len(), 3);
    }

    #[test]
    fn test_events_at_step() {
        let mut pattern = Pattern::new(16);
        let event1 = NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5));
        let event2 = NoteEvent::from_pitch(Pitch::E, 4, 0.7, Some(0.5));

        pattern.add_event(0, event1);
        pattern.add_event(4, event2);

        let events_0 = pattern.events_at_step(0);
        assert_eq!(events_0.len(), 1);

        let events_4 = pattern.events_at_step(4);
        assert_eq!(events_4.len(), 1);

        let events_1 = pattern.events_at_step(1);
        assert_eq!(events_1.len(), 0);
    }

    #[test]
    fn test_clear_step() {
        let mut pattern = Pattern::new(16);
        let event1 = NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5));
        let event2 = NoteEvent::from_pitch(Pitch::E, 4, 0.7, Some(0.5));
        let event3 = NoteEvent::from_pitch(Pitch::G, 4, 0.6, Some(0.5));

        pattern.add_event(0, event1);
        pattern.add_event(0, event2);
        pattern.add_event(4, event3);

        let removed = pattern.clear_step(0);
        assert_eq!(removed, 2);
        assert_eq!(pattern.event_count(), 1);
        assert_eq!(pattern.events_at_step(0).len(), 0);
        assert_eq!(pattern.events_at_step(4).len(), 1);
    }

    #[test]
    fn test_clear() {
        let mut pattern = Pattern::new(16);
        let event = NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5));

        pattern.add_event(0, event);
        pattern.add_event(4, event);
        pattern.add_event(8, event);

        pattern.clear();
        assert_eq!(pattern.event_count(), 0);
        assert!(pattern.is_empty());
    }

    #[test]
    fn test_events_iterator() {
        let mut pattern = Pattern::new(16);
        let event1 = NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5));
        let event2 = NoteEvent::from_pitch(Pitch::E, 4, 0.7, Some(0.5));

        pattern.add_event(0, event1);
        pattern.add_event(4, event2);

        let events: Vec<_> = pattern.events().collect();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].0, 0);
        assert_eq!(events[1].0, 4);
    }

    #[test]
    fn test_set_length_expand() {
        let mut pattern = Pattern::new(16);
        let event = NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5));

        pattern.add_event(0, event);
        pattern.set_length(32);

        assert_eq!(pattern.length(), 32);
        assert_eq!(pattern.event_count(), 1);
    }

    #[test]
    fn test_set_length_shrink() {
        let mut pattern = Pattern::new(16);
        let event = NoteEvent::from_pitch(Pitch::C, 4, 0.8, Some(0.5));

        pattern.add_event(0, event);
        pattern.add_event(8, event);
        pattern.add_event(15, event);

        pattern.set_length(10);

        assert_eq!(pattern.length(), 10);
        assert_eq!(pattern.event_count(), 2); // Event at step 15 removed
        assert_eq!(pattern.events_at_step(0).len(), 1);
        assert_eq!(pattern.events_at_step(8).len(), 1);
        assert_eq!(pattern.events_at_step(15).len(), 0);
    }

    #[test]
    fn test_drum_pattern() {
        let mut pattern = Pattern::new(16);
        pattern.set_name("Basic Beat");

        // Kick on 1 and 9 (steps 0 and 8)
        let kick = NoteEvent::from_midi(36, 115, Some(0.1));
        pattern.add_event(0, kick);
        pattern.add_event(8, kick);

        // Snare on 5 and 13 (steps 4 and 12)
        let snare = NoteEvent::from_midi(38, 102, Some(0.1));
        pattern.add_event(4, snare);
        pattern.add_event(12, snare);

        // Hi-hat on every other step
        let hihat = NoteEvent::from_midi(42, 77, Some(0.05));
        for step in (0..16).step_by(2) {
            pattern.add_event(step, hihat);
        }

        assert_eq!(pattern.event_count(), 12); // 2 kicks + 2 snares + 8 hihats
        assert_eq!(pattern.events_at_step(0).len(), 2); // Kick + hihat
        assert_eq!(pattern.events_at_step(4).len(), 2); // Snare + hihat
    }
}
