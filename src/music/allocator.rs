//! Voice allocator for polyphonic synthesis.
//!
//! # Design Overview
//!
//! The `VoiceAllocator` manages a fixed pool of voices for polyphonic playback,
//! automatically allocating and deallocating voices as notes are triggered and released.
//! When all voices are in use and a new note arrives, a voice stealing strategy
//! determines which existing voice to reuse.
//!
//! # Architecture
//!
//! ## Core Components
//!
//! ### VoiceAllocator<SAMPLE_RATE, VOICES, S, E>
//!
//! Main structure managing voice allocation:
//! - `SAMPLE_RATE`: Sample rate in Hz (const generic)
//! - `VOICES`: Maximum number of simultaneous voices (const generic)
//! - `S`: Signal type (oscillator, wavetable, etc.) - must implement `AudioSignal + Pitched + Clone`
//! - `E`: Envelope type - must implement `Envelope + Clone`
//!
//! ### VoiceState
//!
//! Tracks the state of each voice:
//! - `voice`: The actual Voice instance
//! - `note`: Current MIDI note number (0-127), or None if inactive
//! - `age`: Counter incremented on each note_on, used for "oldest" stealing
//! - `velocity`: Note velocity (0.0-1.0)
//!
//! ### StealingStrategy
//!
//! Enum determining which voice to steal when all are active:
//! - `Oldest`: Steal the voice that was triggered longest ago
//! - `Quietest`: Steal the voice with the lowest envelope level
//! - `Released`: Prefer voices in release phase, then fall back to Oldest
//!
//! ## API Design
//!
//! ### Construction
//!
//! ```rust,ignore
//! impl<const SAMPLE_RATE: u32, const VOICES: usize, S, E> VoiceAllocator<SAMPLE_RATE, VOICES, S, E>
//! where
//!     S: AudioSignal<SAMPLE_RATE> + Pitched + Clone,
//!     E: Envelope + Clone,
//! {
//!     /// Creates a new voice allocator with the given signal and envelope templates.
//!     ///
//!     /// Each voice is created by cloning the provided signal and envelope.
//!     /// The stealing strategy defaults to `Released`.
//!     pub fn new(signal_template: S, envelope_template: E) -> Self;
//!
//!     /// Sets the voice stealing strategy.
//!     pub fn with_strategy(mut self, strategy: StealingStrategy) -> Self;
//! }
//! ```
//!
//! ### Note Control
//!
//! ```rust,ignore
//! impl<const SAMPLE_RATE: u32, const VOICES: usize, S, E> VoiceAllocator<SAMPLE_RATE, VOICES, S, E>
//! where
//!     S: AudioSignal<SAMPLE_RATE> + Pitched + Clone,
//!     E: Envelope + Clone,
//! {
//!     /// Triggers a note with the given MIDI note number and velocity.
//!     ///
//!     /// If a free voice is available, it is used. Otherwise, a voice is stolen
//!     /// according to the stealing strategy.
//!     pub fn note_on(&mut self, note: u8, velocity: f64);
//!
//!     /// Releases the note with the given MIDI note number.
//!     ///
//!     /// If multiple voices are playing the same note, only the first one found
//!     /// is released.
//!     pub fn note_off(&mut self, note: u8);
//!
//!     /// Releases all currently playing notes.
//!     pub fn all_notes_off(&mut self);
//!
//!     /// Returns true if the given note is currently playing.
//!     pub fn is_note_playing(&self, note: u8) -> bool;
//!
//!     /// Returns the number of currently active voices.
//!     pub fn active_voice_count(&self) -> usize;
//! }
//! ```
//!
//! ### Signal Generation
//!
//! ```rust,ignore
//! impl<const SAMPLE_RATE: u32, const VOICES: usize, S, E> Signal
//!     for VoiceAllocator<SAMPLE_RATE, VOICES, S, E>
//! where
//!     S: AudioSignal<SAMPLE_RATE> + Pitched + Clone,
//!     E: Envelope + Clone,
//! {
//!     /// Generates the next sample by summing all active voices.
//!     ///
//!     /// The output is normalized by dividing by the square root of VOICES
//!     /// to prevent clipping while maintaining reasonable volume.
//!     fn next_sample(&mut self) -> f64;
//!
//!     /// Optimized batch processing that generates a buffer of samples.
//!     fn process(&mut self, buffer: &mut [f64]);
//! }
//! ```
//!
//! ## Voice Stealing Algorithm
//!
//! When `note_on()` is called and all voices are active:
//!
//! 1. Check for inactive voices (envelope idle) - use if available
//! 2. If all voices are active, apply stealing strategy:
//!    - **Released**: Find voices in release phase, steal oldest among them.
//!      If none in release, fall back to Oldest strategy.
//!    - **Oldest**: Steal the voice with the lowest age counter
//!    - **Quietest**: Steal the voice with the lowest envelope level
//! 3. Trigger the stolen voice with the new note
//!
//! ## Normalization
//!
//! To prevent clipping when mixing multiple voices:
//! - Output is divided by `sqrt(VOICES)` instead of `VOICES`
//! - This assumes voices are somewhat uncorrelated (phase cancellation)
//! - Provides better perceived loudness while preventing clipping in most cases
//! - For fully correlated signals (all voices playing same note/phase), may still clip
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use earworm::{ADSR, SineOscillator, Signal};
//! use earworm::music::{VoiceAllocator, StealingStrategy};
//!
//! const SAMPLE_RATE: u32 = 44100;
//! const VOICE_COUNT: usize = 8;
//!
//! // Create templates
//! let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
//! let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
//!
//! // Create allocator
//! let mut allocator = VoiceAllocator::<SAMPLE_RATE, VOICE_COUNT, _, _>::new(osc, env)
//!     .with_strategy(StealingStrategy::Released);
//!
//! // Play a chord
//! allocator.note_on(60, 0.8); // C4
//! allocator.note_on(64, 0.8); // E4
//! allocator.note_on(67, 0.8); // G4
//!
//! // Generate audio
//! for _ in 0..44100 {
//!     let sample = allocator.next_sample();
//!     // Output sample...
//! }
//!
//! // Release one note
//! allocator.note_off(64);
//! ```
//!
//! ## Implementation Notes
//!
//! - Voice state is stored in a fixed-size array `[VoiceState; VOICES]`
//! - Age counter wraps at u64::MAX (practically never in real usage)
//! - Clone is required for S and E to create voice instances
//! - Each voice maintains independent state (phase, envelope position, etc.)
//! - Signal mixing is done in next_sample() - no separate mixing buffer needed

use super::{envelope::Envelope, voice::Voice};
use crate::{AudioSignal, Pitched, Signal};

/// Voice stealing strategy for when all voices are active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StealingStrategy {
    /// Steal the voice that was triggered longest ago.
    Oldest,
    /// Steal the voice with the lowest envelope level.
    Quietest,
    /// Prefer stealing voices in release phase, then fall back to Oldest.
    #[default]
    Released,
}

/// State tracking for a single voice in the allocator.
struct VoiceState<const SAMPLE_RATE: u32, S, E>
where
    S: AudioSignal<SAMPLE_RATE> + Pitched,
    E: Envelope,
{
    voice: Voice<SAMPLE_RATE, S, E>,
    note: Option<u8>,
    age: u64,
    velocity: f64,
}

/// Voice allocator for polyphonic synthesis.
///
/// Manages a fixed pool of voices, automatically allocating and stealing voices
/// as needed for polyphonic playback.
///
/// # Type Parameters
///
/// * `SAMPLE_RATE` - Sample rate in Hz
/// * `VOICES` - Maximum number of simultaneous voices
/// * `S` - Signal type (must be `AudioSignal + Pitched + Clone`)
/// * `E` - Envelope type (must be `Envelope + Clone`)
///
/// # Examples
///
/// ```
/// use earworm::{ADSR, SineOscillator, Signal};
/// use earworm::music::VoiceAllocator;
///
/// const SAMPLE_RATE: u32 = 44100;
///
/// let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
/// let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
/// let mut allocator = VoiceAllocator::<SAMPLE_RATE, 8, _, _>::new(osc, env);
///
/// // Play a chord
/// allocator.note_on(60, 0.8);
/// allocator.note_on(64, 0.8);
/// allocator.note_on(67, 0.8);
///
/// // Generate audio
/// let sample = allocator.next_sample();
/// ```
pub struct VoiceAllocator<const SAMPLE_RATE: u32, const VOICES: usize, S, E>
where
    S: AudioSignal<SAMPLE_RATE> + Pitched + Clone,
    E: Envelope + Clone,
{
    voices: [VoiceState<SAMPLE_RATE, S, E>; VOICES],
    strategy: StealingStrategy,
    age_counter: u64,
}

impl<const SAMPLE_RATE: u32, const VOICES: usize, S, E> VoiceAllocator<SAMPLE_RATE, VOICES, S, E>
where
    S: AudioSignal<SAMPLE_RATE> + Pitched + Clone,
    E: Envelope + Clone,
{
    /// Creates a new voice allocator with the given signal and envelope templates.
    ///
    /// Each voice is created by cloning the provided signal and envelope.
    /// The stealing strategy defaults to `Released`.
    ///
    /// # Arguments
    ///
    /// * `signal_template` - Template signal to clone for each voice
    /// * `envelope_template` - Template envelope to clone for each voice
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{ADSR, SineOscillator};
    /// use earworm::music::VoiceAllocator;
    ///
    /// const SAMPLE_RATE: u32 = 44100;
    ///
    /// let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
    /// let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
    /// let allocator = VoiceAllocator::<SAMPLE_RATE, 4, _, _>::new(osc, env);
    /// ```
    pub fn new(signal_template: S, envelope_template: E) -> Self {
        // Create array of voice states by cloning templates
        let voices = std::array::from_fn(|_| VoiceState {
            voice: Voice::new(signal_template.clone(), envelope_template.clone()),
            note: None,
            age: 0,
            velocity: 0.0,
        });

        Self {
            voices,
            strategy: StealingStrategy::default(),
            age_counter: 0,
        }
    }

    /// Sets the voice stealing strategy.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{ADSR, SineOscillator};
    /// use earworm::music::{VoiceAllocator, StealingStrategy};
    ///
    /// const SAMPLE_RATE: u32 = 44100;
    ///
    /// let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
    /// let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
    /// let allocator = VoiceAllocator::<SAMPLE_RATE, 4, _, _>::new(osc, env)
    ///     .with_strategy(StealingStrategy::Oldest);
    /// ```
    pub fn with_strategy(mut self, strategy: StealingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Triggers a note with the given MIDI note number and velocity.
    ///
    /// If a free voice is available, it is used. Otherwise, a voice is stolen
    /// according to the stealing strategy.
    ///
    /// # Arguments
    ///
    /// * `note` - MIDI note number (0-127)
    /// * `velocity` - Note velocity (0.0 to 1.0)
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{ADSR, SineOscillator};
    /// use earworm::music::VoiceAllocator;
    ///
    /// const SAMPLE_RATE: u32 = 44100;
    ///
    /// let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
    /// let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
    /// let mut allocator = VoiceAllocator::<SAMPLE_RATE, 4, _, _>::new(osc, env);
    ///
    /// allocator.note_on(60, 0.8); // Middle C at 80% velocity
    /// ```
    pub fn note_on(&mut self, note: u8, velocity: f64) {
        // Find a voice to use
        let voice_idx = self.find_voice_to_use();

        // Increment age counter
        self.age_counter = self.age_counter.wrapping_add(1);

        // Activate the voice
        let state = &mut self.voices[voice_idx];
        state.note = Some(note);
        state.age = self.age_counter;
        state.velocity = velocity;
        state.voice.note_on(note, velocity);
    }

    /// Releases the note with the given MIDI note number.
    ///
    /// If multiple voices are playing the same note, only the first one found
    /// is released.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{ADSR, SineOscillator};
    /// use earworm::music::VoiceAllocator;
    ///
    /// const SAMPLE_RATE: u32 = 44100;
    ///
    /// let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
    /// let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
    /// let mut allocator = VoiceAllocator::<SAMPLE_RATE, 4, _, _>::new(osc, env);
    ///
    /// allocator.note_on(60, 0.8);
    /// allocator.note_off(60);
    /// ```
    pub fn note_off(&mut self, note: u8) {
        // Find the first voice playing this note
        if let Some(state) = self.voices.iter_mut().find(|v| v.note == Some(note)) {
            state.voice.note_off();
            state.note = None;
        }
    }

    /// Releases all currently playing notes.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{ADSR, SineOscillator};
    /// use earworm::music::VoiceAllocator;
    ///
    /// const SAMPLE_RATE: u32 = 44100;
    ///
    /// let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
    /// let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
    /// let mut allocator = VoiceAllocator::<SAMPLE_RATE, 4, _, _>::new(osc, env);
    ///
    /// allocator.note_on(60, 0.8);
    /// allocator.note_on(64, 0.8);
    /// allocator.all_notes_off();
    /// ```
    pub fn all_notes_off(&mut self) {
        for state in self.voices.iter_mut() {
            state.voice.note_off();
            state.note = None;
        }
    }

    /// Returns true if the given note is currently playing.
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{ADSR, SineOscillator};
    /// use earworm::music::VoiceAllocator;
    ///
    /// const SAMPLE_RATE: u32 = 44100;
    ///
    /// let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
    /// let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
    /// let mut allocator = VoiceAllocator::<SAMPLE_RATE, 4, _, _>::new(osc, env);
    ///
    /// assert!(!allocator.is_note_playing(60));
    /// allocator.note_on(60, 0.8);
    /// assert!(allocator.is_note_playing(60));
    /// ```
    pub fn is_note_playing(&self, note: u8) -> bool {
        self.voices.iter().any(|v| v.note == Some(note))
    }

    /// Returns the number of currently active voices.
    ///
    /// A voice is considered active if its envelope is active (not idle).
    ///
    /// # Examples
    ///
    /// ```
    /// use earworm::{ADSR, SineOscillator};
    /// use earworm::music::VoiceAllocator;
    ///
    /// const SAMPLE_RATE: u32 = 44100;
    ///
    /// let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
    /// let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
    /// let mut allocator = VoiceAllocator::<SAMPLE_RATE, 4, _, _>::new(osc, env);
    ///
    /// assert_eq!(allocator.active_voice_count(), 0);
    /// allocator.note_on(60, 0.8);
    /// assert_eq!(allocator.active_voice_count(), 1);
    /// ```
    pub fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|v| v.voice.is_active()).count()
    }

    /// Finds a voice to use for a new note.
    ///
    /// Priority:
    /// 1. Inactive voice (envelope idle)
    /// 2. Voice to steal based on strategy
    fn find_voice_to_use(&self) -> usize {
        // First, try to find an inactive voice
        if let Some((idx, _)) = self
            .voices
            .iter()
            .enumerate()
            .find(|(_, v)| !v.voice.is_active())
        {
            return idx;
        }

        // All voices are active, need to steal one based on strategy
        self.find_voice_to_steal()
    }

    /// Finds a voice to steal based on the current stealing strategy.
    ///
    /// This is only called when all voices are active.
    fn find_voice_to_steal(&self) -> usize {
        match self.strategy {
            StealingStrategy::Oldest => self.find_oldest_voice(),
            StealingStrategy::Quietest => self.find_quietest_voice(),
            StealingStrategy::Released => self.find_released_or_oldest_voice(),
        }
    }

    /// Finds the oldest voice (lowest age counter).
    fn find_oldest_voice(&self) -> usize {
        self.voices
            .iter()
            .enumerate()
            .min_by_key(|(_, v)| v.age)
            .map(|(idx, _)| idx)
            .unwrap() // Safe because VOICES > 0
    }

    /// Finds the quietest voice (lowest envelope level).
    fn find_quietest_voice(&self) -> usize {
        self.voices
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                a.voice
                    .envelope_level()
                    .partial_cmp(&b.voice.envelope_level())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(idx, _)| idx)
            .unwrap() // Safe because VOICES > 0
    }

    /// Finds a voice in release phase, or falls back to oldest.
    fn find_released_or_oldest_voice(&self) -> usize {
        // Find all voices in their final decay/release phase
        let released_voices: Vec<(usize, &VoiceState<SAMPLE_RATE, S, E>)> = self
            .voices
            .iter()
            .enumerate()
            .filter(|(_, v)| v.voice.is_releasing())
            .collect();

        if !released_voices.is_empty() {
            // Steal the oldest voice in release/decay phase
            released_voices
                .iter()
                .min_by_key(|(_, v)| v.age)
                .map(|(idx, _)| *idx)
                .unwrap()
        } else {
            // No voices releasing, fall back to oldest
            self.find_oldest_voice()
        }
    }
}

impl<const SAMPLE_RATE: u32, const VOICES: usize, S, E> Signal
    for VoiceAllocator<SAMPLE_RATE, VOICES, S, E>
where
    S: AudioSignal<SAMPLE_RATE> + Pitched + Clone,
    E: Envelope + Clone,
{
    fn next_sample(&mut self) -> f64 {
        // Sum all voice outputs
        let sum: f64 = self.voices.iter_mut().map(|v| v.voice.next_sample()).sum();

        // Normalize by sqrt(VOICES) to prevent clipping
        // This assumes some phase cancellation between voices
        sum / (VOICES as f64).sqrt()
    }

    fn process(&mut self, buffer: &mut [f64]) {
        // Clear buffer
        buffer.fill(0.0);

        // Mix each voice into the buffer
        let mut voice_buffer = vec![0.0; buffer.len()];
        for voice_state in self.voices.iter_mut() {
            voice_state.voice.process(&mut voice_buffer);
            for (out, &voice_sample) in buffer.iter_mut().zip(voice_buffer.iter()) {
                *out += voice_sample;
            }
        }

        // Normalize
        let scale = 1.0 / (VOICES as f64).sqrt();
        for sample in buffer.iter_mut() {
            *sample *= scale;
        }
    }
}

impl<const SAMPLE_RATE: u32, const VOICES: usize, S, E> AudioSignal<SAMPLE_RATE>
    for VoiceAllocator<SAMPLE_RATE, VOICES, S, E>
where
    S: AudioSignal<SAMPLE_RATE> + Pitched + Clone,
    E: Envelope + Clone,
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ADSR, Signal, SineOscillator};

    const SAMPLE_RATE: u32 = 44100;

    #[test]
    fn test_creation() {
        let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
        let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
        let allocator = VoiceAllocator::<SAMPLE_RATE, 4, _, _>::new(osc, env);

        assert_eq!(allocator.active_voice_count(), 0);
    }

    #[test]
    fn test_basic_note_on_off() {
        let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
        let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
        let mut allocator = VoiceAllocator::<SAMPLE_RATE, 4, _, _>::new(osc, env);

        // Initially no notes playing
        assert!(!allocator.is_note_playing(60));
        assert_eq!(allocator.active_voice_count(), 0);

        // Play a note
        allocator.note_on(60, 0.8);
        assert!(allocator.is_note_playing(60));
        assert_eq!(allocator.active_voice_count(), 1);

        // Release the note
        allocator.note_off(60);
        assert!(!allocator.is_note_playing(60));
        // Voice is still active during release
        assert_eq!(allocator.active_voice_count(), 1);
    }

    #[test]
    fn test_multiple_simultaneous_notes() {
        let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
        let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
        let mut allocator = VoiceAllocator::<SAMPLE_RATE, 8, _, _>::new(osc, env);

        // Play a chord (C major)
        allocator.note_on(60, 0.8); // C
        allocator.note_on(64, 0.8); // E
        allocator.note_on(67, 0.8); // G

        assert!(allocator.is_note_playing(60));
        assert!(allocator.is_note_playing(64));
        assert!(allocator.is_note_playing(67));
        assert_eq!(allocator.active_voice_count(), 3);

        // Release one note
        allocator.note_off(64);
        assert!(!allocator.is_note_playing(64));
        assert!(allocator.is_note_playing(60));
        assert!(allocator.is_note_playing(67));
    }

    #[test]
    fn test_voice_stealing_when_exceeding_limit() {
        let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
        let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
        let mut allocator = VoiceAllocator::<SAMPLE_RATE, 4, _, _>::new(osc, env);

        // Play 4 notes (fill all voices)
        allocator.note_on(60, 0.8);
        allocator.note_on(62, 0.8);
        allocator.note_on(64, 0.8);
        allocator.note_on(65, 0.8);

        assert_eq!(allocator.active_voice_count(), 4);

        // Play a 5th note - should steal the oldest (first) voice
        allocator.note_on(67, 0.8);

        // Should still have 4 active voices
        assert_eq!(allocator.active_voice_count(), 4);

        // The newest note should be playing
        assert!(allocator.is_note_playing(67));

        // The oldest note (60) should have been stolen
        assert!(!allocator.is_note_playing(60));
    }

    #[test]
    fn test_all_notes_off() {
        let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
        let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
        let mut allocator = VoiceAllocator::<SAMPLE_RATE, 4, _, _>::new(osc, env);

        // Play multiple notes
        allocator.note_on(60, 0.8);
        allocator.note_on(64, 0.8);
        allocator.note_on(67, 0.8);

        assert_eq!(allocator.active_voice_count(), 3);

        // Release all
        allocator.all_notes_off();

        assert!(!allocator.is_note_playing(60));
        assert!(!allocator.is_note_playing(64));
        assert!(!allocator.is_note_playing(67));
    }

    #[test]
    fn test_voice_recycling() {
        let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
        // Very short envelope for quick recycling
        let env = ADSR::new(0.001, 0.001, 0.7, 0.001, SAMPLE_RATE as f64);
        let mut allocator = VoiceAllocator::<SAMPLE_RATE, 2, _, _>::new(osc, env);

        // Play and release a note
        allocator.note_on(60, 0.8);
        allocator.note_off(60);

        // Generate samples until voice becomes inactive
        for _ in 0..1000 {
            allocator.next_sample();
        }

        // Voice should be inactive now and available for reuse
        assert_eq!(allocator.active_voice_count(), 0);

        // Play a new note - should reuse the inactive voice
        allocator.note_on(64, 0.8);
        assert_eq!(allocator.active_voice_count(), 1);
    }

    #[test]
    fn test_rapid_note_changes() {
        let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
        let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
        let mut allocator = VoiceAllocator::<SAMPLE_RATE, 4, _, _>::new(osc, env);

        // Rapidly trigger and release notes
        for note in 60..80 {
            allocator.note_on(note, 0.8);
            allocator.note_off(note);

            // Generate a few samples
            for _ in 0..10 {
                allocator.next_sample();
            }
        }

        // Should not panic or produce invalid state
        assert!(allocator.active_voice_count() <= 4);
    }

    #[test]
    fn test_signal_generation() {
        let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
        let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
        let mut allocator = VoiceAllocator::<SAMPLE_RATE, 4, _, _>::new(osc, env);

        // Play a note
        allocator.note_on(60, 0.8);

        // Generate samples
        for _ in 0..100 {
            let sample = allocator.next_sample();
            // Should produce valid audio samples
            assert!(sample.abs() <= 2.0); // Allow some headroom above 1.0
        }
    }

    #[test]
    fn test_process_buffer() {
        let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
        let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
        let mut allocator = VoiceAllocator::<SAMPLE_RATE, 4, _, _>::new(osc, env);

        allocator.note_on(60, 0.8);
        allocator.note_on(64, 0.8);

        let mut buffer = vec![0.0; 128];
        allocator.process(&mut buffer);

        // Should produce non-zero samples
        assert!(buffer.iter().any(|&s| s.abs() > 0.01));
    }

    #[test]
    fn test_stealing_strategy_oldest() {
        let osc = SineOscillator::<SAMPLE_RATE>::new(440.0);
        let env = ADSR::new(0.01, 0.1, 0.7, 0.3, SAMPLE_RATE as f64);
        let mut allocator = VoiceAllocator::<SAMPLE_RATE, 3, _, _>::new(osc, env)
            .with_strategy(StealingStrategy::Oldest);

        // Fill all voices
        allocator.note_on(60, 0.8);
        allocator.note_on(62, 0.8);
        allocator.note_on(64, 0.8);

        // Trigger another - should steal the oldest (60)
        allocator.note_on(65, 0.8);

        assert!(!allocator.is_note_playing(60));
        assert!(allocator.is_note_playing(62));
        assert!(allocator.is_note_playing(64));
        assert!(allocator.is_note_playing(65));
    }
}
