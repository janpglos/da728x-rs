# AGENTS.md

## Project Overview

`da728x` is an async, `no_std` Rust driver library for the Renesas DA7280/DA7281/DA7282 wide-bandwidth haptic driver ICs. It communicates over I2C using `embedded-hal-async` and is designed for embedded targets (RP2040, nRF54L15, etc.) running the Embassy async runtime.

Datasheets for all three ICs live in `support/` — consult them when working on register-level details.

## Commands

```bash
# Build the library (host target, for checking compilation)
cargo build

# Build with debug logging enabled
cargo build --features debug

# Run unit tests (host target only — waveform encoding logic)
cargo test

# Build an RP2040 example (from examples/rp2040/)
cd examples/rp2040
cargo build --release --example simple_dro

# Flash and run on RP2040 via probe-rs (runner configured in .cargo/config.toml)
cargo run --release --example simple_dro

# Build the nRF54L15 example (from examples/nrf54l15/)
cd examples/nrf54l15
cargo build --release
```

There is no linter or formatter configured. No CI pipeline exists.

## Architecture

```
src/
├── lib.rs          # DA728x<I2C> driver struct — all async I2C register operations
├── config.rs       # ActuatorConfig, DeviceConfig, enums (OperationMode, DrivingMode, ActuatorType)
├── errors.rs       # Error enum (I2c, Gpio, VariantMismatch, InvalidValue, NotConfigured, WrongMode, waveform errors)
├── registers.rs    # Register address enum + bitfield structs via bitfield-struct crate
└── waveform/
    ├── mod.rs      # Re-exports: SnippetBuilder, FrameBuilder, SequenceBuilder, WaveformMemoryBuilder
    ├── snippet.rs  # PwlPoint (single byte: RMP + TIME + AMP), Snippet/SnippetBuilder
    ├── frame.rs    # Frame (1-3 bytes), FrameBuilder, Gain enum, Timebase enum
    ├── sequence.rs # Sequence of frames, SequenceBuilder
    └── memory.rs   # WaveformMemory + WaveformMemoryBuilder — assembles final 100-byte memory image
```

### Data Flow

1. User creates `ActuatorConfig` + `DeviceConfig` and calls `DA728x::configure()` which validates and writes all relevant registers.
2. For waveform playback, user builds a `WaveformMemory` using the builder chain: `SnippetBuilder` → `FrameBuilder` → `SequenceBuilder` → `WaveformMemoryBuilder`.
3. `WaveformMemory` is uploaded via `DA728x::upload_waveform_memory()` which unlocks memory, writes bytes (in 32-byte chunks), and optionally re-locks.
4. Playback is triggered via `DA728x::play_sequence(sequence_id, loops)`.

### Waveform Memory Layout

100 bytes total at registers `SNP_MEM_0..SNP_MEM_99` (0x84..0xE7):
- Byte 0: number of snippets (1-15, ID 0 is reserved silence)
- Byte 1: number of sequences (1-16)
- Bytes 2+: end pointers (absolute indices), then snippet data, then sequence data

## Key Conventions

- **Edition 2024** — uses the latest Rust edition.
- **`#![deny(unsafe_code)]`** — no unsafe code allowed.
- **`#![no_std]`** — no standard library; all types use `core::` imports.
- **Naming**: Register structs and bitfield accessors use SCREAMING_SNAKE_CASE matching the datasheet register/field names (e.g., `TOP_CFG1`, `with_OPERATION_MODE()`). The entire `registers.rs` module has `#[allow(non_camel_case_types)]` and `#[allow(non_snake_case)]`.
- **Config struct fields** use units as suffixes: `_mV`, `_mA`, `_mOhm`, `_uH`, `_Hz`.
- **Config enums** (OperationMode, DrivingMode, ActuatorType) use `#[allow(nonstandard_style)]` with ALL_CAPS variants.
- **Builder pattern**: all waveform builders are consuming (take `self`, return `Result<Self, Error>`).
- **Async everywhere**: the driver only exposes `async fn` methods. There is no blocking API.
- **`embedded-hal-async` v1.0**: uses `I2c::write`, `I2c::write_read` from the 1.0 traits.

## Testing

Unit tests exist only in `src/waveform/` modules (snippet, frame, sequence, memory). These test byte encoding, validation, and builder logic. Run with `cargo test`.

There are no integration tests — testing against hardware requires flashing examples to a target board.

## Gotchas
- **`ActuatorConfig` with old fields**: Some examples (e.g., `simple_dro.rs:77-84`) create `ActuatorConfig` without `inductance_uH`, `pid_Kp`, `pid_Ki` fields, while the struct definition in `config.rs` requires them. The examples may not compile with the current struct unless these fields are added.
- **Snippet ID 0 is reserved** for the built-in silence snippet — user snippets start at ID 1.
- **`WAV_MEM_LOCK` semantics are inverted**: `WAV_MEM_LOCK = 1` means **unlocked** (writable), `0` means **locked**. See `unlock_waveform_memory()` / `lock_waveform_memory()`.
- **`get_events()` is destructive**: reading IRQ events also clears them (writes 0xFF to `IRQ_EVENT1`).
- **Frequency range depends on driving mode**: 50-300 Hz for `FREQUENCY_TRACK`, 25-1024 Hz for `WIDEBAND`/`CUSTOM_WAVEFORM`.
- **`MEM_DATA_SIGNED` must be inverted relative to `ACCELERATION_EN`**: `MEM_DATA_SIGNED = !acceleration_en`. See `configure()` in lib.rs.
- **Waveform memory is limited to 100 bytes** — the builder enforces this at build time.
- **End pointers in waveform memory are absolute indices** (not relative offsets) pointing to the last byte of each snippet/sequence.

## Features

- `debug` — enables `defmt` logging throughout the driver. Adds `defmt` format to bitfield structs via `#[cfg_attr(feature = "debug", bitfield(u8, defmt = true))]`.

## Example Projects

- **`examples/rp2040/`** — targets `thumbv6m-none-eabi`, uses `embassy-rp` with `probe-rs` runner. Multiple examples: `simple_dro`, `simple_waveform`, `waveform_effects`, `wideband_melody`.
- **`examples/nrf54l15/`** — targets `thumbv8m.main-none-eabihf`, uses `embassy-nrf` with `nrf54l15-app-s`. Single binary example.

Both examples depend on the library with `features = ["debug"]` and use `defmt-rtt` for logging.

## Not Yet Implemented

- PWM mode testing
- GPI configuration and ETWM_MODE
- Script upload (list of registers and values as exported by GUI)
