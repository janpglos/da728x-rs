//! Simple RTWM (Register-Triggered Waveform Memory) example on nRF54L15.
//!
//! This example demonstrates the basics of waveform memory:
//! - Creating a simple snippet (waveform shape)
//! - Creating a sequence that plays the snippet
//! - Uploading to the DA7280's waveform memory
//! - Triggering playback via register writes
//!
//! # Hardware Setup
//!
//! - DA7280 connected via TWIM (SERIAL20, SDA: P1.10, SCL: P1.11)
//! - I2C address: 0x4A (default)
//!
//! # Waveform Memory Concepts
//!
//! - **Snippet**: A piecewise-linear (PWL) waveform shape defined by amplitude
//!   points over time. Snippet ID 0 is reserved for built-in silence.
//! - **Frame**: References a snippet with gain, timebase, and optional looping.
//! - **Sequence**: A series of frames played in order.
//!
//! # Running
//!
//! ```bash
//! cargo run --release --example simple_waveform
//! ```

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{bind_interrupts, peripherals};
use embassy_time::Timer;
use static_cell::ConstStaticCell;
use {defmt_rtt as _, panic_probe as _};

use da728x::waveform::{
    FrameBuilder, Gain, SequenceBuilder, SnippetBuilder, Timebase, WaveformMemoryBuilder,
};
use da728x::{Variant, DA728x};
use da728x_examples_common as common;

bind_interrupts!(struct Irqs {
    SERIAL20 => twim::InterruptHandler<peripherals::SERIAL20>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    info!("=== Simple Waveform (RTWM) Example (nRF54L15) ===");
    info!("Make sure the actuator is loaded (pressed between two surfaces)!");

    info!("Initializing TWI...");
    static RAM_BUFFER: ConstStaticCell<[u8; 16]> = ConstStaticCell::new([0; 16]);
    let twi = Twim::new(
        p.SERIAL20,
        Irqs,
        p.P1_10,
        p.P1_11,
        twim::Config::default(),
        RAM_BUFFER.take(),
    );

    info!("Setting up DA7280 haptics driver...");
    let mut haptics = DA728x::new(twi, 0x4A, Variant::DA7280)
        .await
        .unwrap();
    info!("DA7280 initialized successfully.");

    let actuator_config = common::config::sparkfun_lra_config();

    // Build a simple click waveform
    info!("Building waveform memory...");

    let click_snippet = SnippetBuilder::new()
        .ramp(1, 15).unwrap()
        .ramp(2, 0).unwrap()
        .build()
        .unwrap();

    let click_frame = FrameBuilder::new(1).unwrap()
        .gain(Gain::Full)
        .timebase(Timebase::Ms21_76)
        .build()
        .unwrap();

    let click_sequence = SequenceBuilder::new()
        .add_frame(click_frame).unwrap()
        .build()
        .unwrap();

    let memory = WaveformMemoryBuilder::new(false)
        .add_snippet(click_snippet).unwrap()
        .add_sequence(click_sequence).unwrap()
        .build()
        .unwrap();

    info!(
        "Memory built: {} bytes, {} snippet(s), {} sequence(s)",
        memory.len(),
        memory.num_snippets(),
        memory.num_sequences()
    );

    let device_config = common::config::rtwm_frequency_track();

    haptics.configure(actuator_config, device_config).await.unwrap();
    info!("Configured for RTWM mode.");

    // Upload and verify waveform memory
    info!("Uploading waveform memory...");
    haptics.upload_waveform_memory(&memory, false).await.unwrap();

    let mut readback = [0u8; 16];
    haptics.read_waveform_memory(memory.len(), &mut readback).await.unwrap();
    let expected = memory.as_bytes();
    let verified = readback[..memory.len()] == expected[..memory.len()];
    if verified {
        info!("Memory verification: PASSED");
    } else {
        error!("Memory verification: FAILED");
    }

    haptics.lock_waveform_memory().await.unwrap();
    haptics.enable().await.unwrap();
    info!("RTWM enabled. Playing clicks...");

    loop {
        info!("Click!");
        haptics.play_sequence(0, 0).await.unwrap();
        Timer::after_millis(500).await;

        common::demo::handle_faults(&mut haptics).await;
    }
}
