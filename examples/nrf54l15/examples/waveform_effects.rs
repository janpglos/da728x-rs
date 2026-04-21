//! Complex waveform effects example on nRF54L15.
//!
//! This example shows how to create and play multiple haptic effects:
//! - Click: Sharp, quick feedback
//! - Double-click: Two clicks with a pause
//! - Buzz: Sustained vibration with looping
//!
//! # Hardware Setup
//!
//! - DA7280 connected via TWIM (SERIAL20, SDA: P1.10, SCL: P1.11)
//! - I2C address: 0x4A (default)
//!
//! # Waveform Memory Layout
//!
//! - Snippet 1: Click shape (quick rise, smooth fall)
//! - Snippet 2: Bump shape (gradual rise, hold, fall)
//! - Snippet 3: Buzz shape (rise, sustain, fall)
//! - Sequence 0: Single click
//! - Sequence 1: Double click (two clicks with silence between)
//! - Sequence 2: Buzz (sustained vibration with loop)
//!
//! # Running
//!
//! ```bash
//! cargo run --release --example waveform_effects
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

    info!("=== Waveform Effects Example (nRF54L15) ===");
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

    // Build waveform memory with multiple effects
    info!("Building waveform memory...");
    let memory = build_waveform_memory();
    info!(
        "Memory built: {} bytes, {} snippets, {} sequences",
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

    let mut readback = [0u8; 32];
    haptics.read_waveform_memory(memory.len(), &mut readback).await.unwrap();
    let expected = memory.as_bytes();
    let mut verified = true;
    for i in 0..memory.len() {
        if readback[i] != expected[i] {
            error!("Mismatch at byte {}: got {:02X}, expected {:02X}", i, readback[i], expected[i]);
            verified = false;
        }
    }
    if verified {
        info!("Memory verification: PASSED");
    } else {
        error!("Memory verification: FAILED");
    }

    haptics.lock_waveform_memory().await.unwrap();
    haptics.enable().await.unwrap();

    // Clear any stale events
    let _ = haptics.get_events().await;

    info!("RTWM enabled. Playing effects...");

    loop {
        // Effect 1: Single click
        info!("Effect: Click");
        haptics.play_sequence(0, 0).await.unwrap();
        Timer::after_millis(600).await;

        // Effect 2: Double click
        info!("Effect: Double click");
        haptics.play_sequence(1, 0).await.unwrap();
        Timer::after_millis(800).await;

        // Effect 3: Buzz
        info!("Effect: Buzz");
        haptics.play_sequence(2, 0).await.unwrap();
        Timer::after_millis(1500).await;

        if !common::demo::handle_faults(&mut haptics).await {
            info!("All effects OK");
        }

        Timer::after_millis(1000).await;
    }
}

fn build_waveform_memory() -> da728x::waveform::WaveformMemory {
    let click_snippet = SnippetBuilder::new()
        .ramp(1, 15).unwrap()
        .ramp(2, 0).unwrap()
        .build()
        .unwrap();

    let bump_snippet = SnippetBuilder::new()
        .ramp(2, 15).unwrap()
        .step(2, 15).unwrap()
        .ramp(2, 0).unwrap()
        .build()
        .unwrap();

    let buzz_snippet = SnippetBuilder::new()
        .ramp(1, 15).unwrap()
        .step(6, 15).unwrap()
        .ramp(1, 0).unwrap()
        .build()
        .unwrap();

    let click_frame = FrameBuilder::new(1).unwrap()
        .gain(Gain::Full)
        .timebase(Timebase::Ms21_76)
        .build()
        .unwrap();
    let click_seq = SequenceBuilder::new()
        .add_frame(click_frame).unwrap()
        .build()
        .unwrap();

    let frame1 = FrameBuilder::new(1).unwrap()
        .gain(Gain::Full)
        .timebase(Timebase::Ms21_76)
        .build()
        .unwrap();
    let silence = FrameBuilder::silence()
        .timebase(Timebase::Ms43_52)
        .build()
        .unwrap();
    let frame2 = FrameBuilder::new(1).unwrap()
        .gain(Gain::Full)
        .timebase(Timebase::Ms21_76)
        .build()
        .unwrap();
    let double_click_seq = SequenceBuilder::new()
        .add_frame(frame1).unwrap()
        .add_frame(silence).unwrap()
        .add_frame(frame2).unwrap()
        .build()
        .unwrap();

    let buzz_frame = FrameBuilder::new(3).unwrap()
        .gain(Gain::Full)
        .timebase(Timebase::Ms21_76)
        .loop_count(3).unwrap()
        .build()
        .unwrap();
    let buzz_seq = SequenceBuilder::new()
        .add_frame(buzz_frame).unwrap()
        .build()
        .unwrap();

    WaveformMemoryBuilder::new(false)
        .add_snippet(click_snippet).unwrap()
        .add_snippet(bump_snippet).unwrap()
        .add_snippet(buzz_snippet).unwrap()
        .add_sequence(click_seq).unwrap()
        .add_sequence(double_click_seq).unwrap()
        .add_sequence(buzz_seq).unwrap()
        .build()
        .unwrap()
}
