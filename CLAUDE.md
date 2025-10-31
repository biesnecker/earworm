# CLAUDE.md

**Note**: This project uses [bd (beads)](https://github.com/steveyegge/beads) for issue tracking. Use `bd` commands instead of markdown TODOs. See AGENTS.md for workflow details.

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Earworm is a Rust audio synthesis library providing oscillators, filters, envelopes, effects, and signal processing combinators for building audio applications. The library uses trait-based composition to enable flexible signal graph construction.

## Build and Test Commands

```bash
# Run all tests
cargo test

# Run a specific test
cargo test test_name

# Build the library
cargo build

# Run an example
cargo run --example example_name

# Run examples with interactive controls (use Q/ESC to quit, SPACE for interactions)
cargo run --example play_oscillators
cargo run --example filter_demo_interactive
cargo run --example adsr_spacebar
```

## Architecture

### Core Trait Hierarchy

The library is built on a trait hierarchy that enables composable signal processing:

1. **Signal** (src/signals/core.rs): Base trait for anything that generates samples via `next_sample()` and `process(buffer)`
2. **AudioSignal** (src/signals/audio.rs): Extends Signal with sample rate awareness
3. **Oscillator** (src/oscillators/traits.rs): Extends AudioSignal with frequency control

### Parameter Modulation System

The **Param** enum (src/signals/core.rs) is central to the library's design:
- `Param::Fixed(f64)`: Static constant value
- `Param::Signal(Box<dyn Signal>)`: Dynamic modulation by another signal (LFO, envelope, etc.)

This enables any parameter to be either fixed or modulated without generic complexity. Use `.into()` to convert `f64` or any `Signal` to `Param`.

### Module Organization

- **oscillators/**: Waveform generators (sine, triangle, sawtooth, square, pulse)
- **filters/**: BiquadFilter with multiple FilterType variants (lowpass, highpass, bandpass, notch, allpass)
- **envelopes/**: ADSR envelope and Curve interpolation types
- **effects/**: Time-based effects (Delay, bitcrusher coming)
- **noise/**: Noise generators (WhiteNoise, PinkNoise)
- **combinators/**: Signal transformation and combination (Add, Multiply, Gain, Offset, Mix, Clamp, Crossfade, etc.)

### Signal Composition Patterns

The library provides two approaches for combining signals:

1. **Direct struct construction**:
```rust
let signal = Multiply { a: osc1, b: osc2 };
```

2. **Fluent API via SignalExt and AudioSignalExt traits**:
```rust
let signal = osc1.multiply(osc2).gain(0.5).lowpass_filter(1000.0, 0.707);
```

The extension traits are blanket-implemented, so all Signals get SignalExt methods and all AudioSignals get AudioSignalExt methods.

### Important Implementation Details

- All signals must implement `Signal + Send` to be used in `Param` or `Mix`
- Sample rate is immutable once set (passed during construction)
- Filters use biquad IIR implementation for efficiency
- The `process()` method can be overridden for batch optimization but defaults to calling `next_sample()` per sample
- Ring modulation is just `Multiply` between two audio-rate signals
- LFO effects use `Param::modulated()` or convert a low-frequency oscillator to Param

## Development Notes

- Examples use `cpal` for cross-platform audio output
- Interactive examples use `crossterm` for keyboard input (space toggles/triggers, Q/ESC quits)
- Edition is set to "2024" in Cargo.toml (uses latest Rust edition)
- No unsafe code is used in the core library
