✅ DO use the common code in common/mod.rs to reduce boilerplate and improve consistency.
✅ DO look at existing demos to see how the simple UI works.
   - play_oscillators and play_noise are good examples
   - voice_demo shows keyboard-based musical input
❌ DO NOT make your own weird UI that doesn't follow common patterns.

## Common Module Features

### Keyboard Input for Musical Examples

For examples that use keyboard input to play notes, use the built-in keyboard utilities:

```rust
use common::{
    draw_keyboard_ui, key_to_midi_note, midi_note_to_name,
    KeyboardConfig, run_interactive_example
};

// In your initial UI function:
fn draw_ui() -> Result<()> {
    draw_keyboard_ui("My Synth Demo", None)
    // Or with extra controls:
    // draw_keyboard_ui("My Synth Demo", Some("SPACE = Toggle effect"))
}

// In your key handler:
if let Some(midi_note) = key_to_midi_note(key_event.code) {
    // Handle note on/off
}

// Use KeyboardConfig::with_enhancements() to get press/release events
run_interactive_example(
    state,
    KeyboardConfig::with_enhancements(),
    |_| draw_ui(),
    |state, key| { /* handle keys */ }
)
```

**Important**: The `draw_keyboard_ui()` function reserves line 1 for status updates from `output_info()`. Make sure your `ExampleAudioState` implementation provides meaningful status information.
