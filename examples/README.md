# Earworm Examples

This folder contains example programs demonstrating the earworm audio library.

## Interactive Examples

### play_oscillators

Interactive example to switch between different oscillator types in real-time.

```bash
cargo run --example play_oscillators
```

**Controls:**
- **SPACE** - Switch oscillator type (Sine → Triangle → Sawtooth → Square → Pulse → Pulse PWM → ...)
- **Q/ESC** - Quit

Plays a 440 Hz (A4 note) oscillator and lets you hear the differences between waveform types:
- **Pulse (25%)**: Fixed 25% duty cycle
- **Pulse (PWM)**: Duty cycle modulated by a 0.5 Hz LFO, creating a classic sweeping analog synthesizer effect

### play_noise

Interactive example to switch between different noise types in real-time.

```bash
cargo run --example play_noise
```

**Controls:**
- **SPACE** - Switch noise type (White → Pink → ...)
- **Q/ESC** - Quit

Lets you compare white noise (equal power across all frequencies) and pink noise (equal power per octave).

## Simple Playback Examples

The following examples play a single waveform for 5 seconds:

### play_sine
```bash
cargo run --example play_sine
```

### play_triangle
```bash
cargo run --example play_triangle
```

### play_sawtooth
```bash
cargo run --example play_sawtooth
```

### play_square
```bash
cargo run --example play_square
```

**Note:** All examples use the `cpal` library for cross-platform audio output.
