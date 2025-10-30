# Earworm Examples

This folder contains example programs demonstrating the earworm audio library.

## Interactive Examples

### play_deadmau5_filter

**NEW!** Recreates the iconic "deadmau5 filter" effect - a pulsing low-pass filter that creates a dramatic rhythmic sound.

```bash
cargo run --example play_deadmau5_filter
```

**Controls:**
- **Q/ESC** - Quit

This example demonstrates:
- A sawtooth oscillator (rich harmonic content)
- Low-pass filter with cutoff modulated by a square wave LFO
- Cutoff drops from 4000Hz to 50Hz on 8th notes at 120 BPM (4 Hz)
- Creates the characteristic "pulsing" effect heard in many electronic dance tracks

### filter_demo_interactive

Interactive demonstration of audio filters with real-time switching.

```bash
cargo run --example filter_demo_interactive
```

**Controls:**
- **SPACE** - Cycle through different filter types
- **Q/ESC** - Quit

Demonstrates various filter effects on a triangle wave:
1. **Raw Signal** - Unfiltered triangle wave
2. **Low-Pass** - Removes high frequencies (cutoff: 800Hz)
3. **High-Pass** - Removes low frequencies (cutoff: 600Hz)
4. **Band-Pass** - Isolates a frequency range
5. **Resonant Low-Pass** - Emphasizes the cutoff frequency
6. **Swept Filter** - LFO-modulated cutoff (300-1500Hz)
7. **Chained Filters** - Multiple filtering stages
8. **Notch Filter** - Removes a specific frequency

Press spacebar to cycle through each mode and hear the differences!

### adsr_spacebar

Interactive ADSR envelope demonstration.

```bash
cargo run --example adsr_spacebar
```

**Controls:**
- **SPACE** (hold) - Trigger note with envelope
- **SPACE** (release) - Release phase
- **Q/ESC** - Quit

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
