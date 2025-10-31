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

## Important Development Notes

### Pre-Commit Checklist (MUST complete ALL before claiming task is done)
1. **Format**: `cargo fmt --all`
2. **Lint**: `cargo clippy --all-targets --all-features -- -D warnings`
3. **Test**: `cargo test --all-features` (all tests must pass)
4. **Build**: `cargo build --all-features` (must compile cleanly)
5. **Build Examples with all features**: `cargo build --examples --all-features`
6. **Build Examples without features**: `cargo build --examples` (ensures examples handle missing features correctly)
7. **Run New Examples**: For ANY new example created, manually run `cargo run --example <name>` to verify it actually executes (not just compiles)

### Creating New Examples
When adding a new example, follow these steps to ensure it works correctly in all configurations:

1. **Create the example file** in `examples/`
2. **Declare feature dependencies** (if needed) in `Cargo.toml`:
   ```toml
   [[example]]
   name = "example_name"
   required-features = ["feature1", "feature2"]
   ```
3. **Test with all features**: `cargo run --example example_name --all-features` - verify it runs without crashing
4. **Test without features**: `cargo run --example example_name` - ensure it either works or fails gracefully with a clear error message
5. **Build verification**: Run both `cargo build --examples --all-features` and `cargo build --examples` to catch feature dependency issues
6. **Document usage** in README.md with any special instructions or controls

**Important**: Building examples with `--all-features` can hide feature dependency problems. Always test both with and without features to ensure correct `required-features` declarations.

### Why These Steps Matter
- `cargo build --examples --all-features` can hide feature dependency issues
- Examples must work standalone, not just when all features are enabled
- Running examples catches runtime errors that compilation misses
- DO NOT claim a task is complete without running ALL checks above

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
