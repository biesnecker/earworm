# Earworm Examples

This folder contains example programs demonstrating the earworm audio library.

## Available Examples

### play_sine
Real-time audio playback of a sine wave (requires audio output device).

```bash
cargo run --example play_sine
```

Plays a 440 Hz (A4 note) sine wave through your speakers for 5 seconds.

**Note:** This example uses the `cpal` library for cross-platform audio output.
