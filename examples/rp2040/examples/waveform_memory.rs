//! Upload exported waveform memory on RP2040.
//!
//! This example shows how to upload the waveform as exported from the waveform builder
//! See: tools/example_waveform_memory.txt
//!
//! # Hardware Setup
//!
//! - DA7280 connected via I2C0 (SDA: GP16, SCL: GP17)
//! - I2C address: 0x4A (default)
//!
//! # Running
//!
//! ```bash
//! cargo run --release --example waveform_memory
//! ```

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::i2c::{self, Config, I2c};
use embassy_rp::peripherals::I2C0;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use da728x::waveform::WaveformMemory;
use da728x::{DA728x, Variant};
use da728x_examples_common as common;

// Auto-generated DA7280 waveform memory
pub const DA7280_WAVEFORM_MEMORY: WaveformMemory = WaveformMemory::from_bytes(
    [
        0x07, 0x0B, 0x14, 0x15, 0x16, 0x1A, 0x1E, 0x1F, 0x20, 0x23, 0x25, 0x27, 0x2D, 0x38, 0x39,
        0x40, 0x43, 0x47, 0x48, 0x49, 0x77, 0x29, 0x47, 0xF7, 0xF0, 0xF9, 0xF0, 0x77, 0x77, 0x70,
        0x70, 0xF7, 0xF0, 0x01, 0x88, 0x18, 0x01, 0x18, 0x03, 0x18, 0x03, 0x10, 0xB8, 0x31, 0x88,
        0x18, 0x01, 0x88, 0x10, 0x90, 0x01, 0x88, 0x10, 0x90, 0x29, 0xA8, 0x18, 0x14, 0x05, 0xA8,
        0x18, 0x0D, 0x88, 0x05, 0xA8, 0x01, 0x02, 0x18, 0x1E, 0x11, 0x1F, 0x00, 0x19, 0x39, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ],
    74,
    7,
    11,
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("=== Waveform Memory Example (RP2040) ===");

    info!("Initializing I2C...");
    let sda = p.PIN_16;
    let scl = p.PIN_17;
    let mut config = Config::default();
    config.frequency = 400_000;

    let i2c = I2c::new_async(p.I2C0, scl, sda, Irqs, config);

    info!("Setting up DA7280 haptics driver...");
    let mut haptics = DA728x::new(i2c, 0x4A, Variant::DA7280).await.unwrap();
    info!("DA7280 initialized successfully.");

    let actuator_config = common::config::sparkfun_lra_config();

    info!(
        "Waveform memory: {} bytes, {} snippets, {} sequences",
        DA7280_WAVEFORM_MEMORY.len(),
        DA7280_WAVEFORM_MEMORY.num_snippets(),
        DA7280_WAVEFORM_MEMORY.num_sequences()
    );

    let device_config = common::config::rtwm_frequency_track();

    haptics
        .configure(actuator_config, device_config)
        .await
        .unwrap();
    info!("Configured for RTWM mode.");

    // Upload and verify waveform memory
    info!("Uploading waveform memory...");
    haptics
        .upload_waveform_memory(&DA7280_WAVEFORM_MEMORY, false)
        .await
        .unwrap();

    haptics.lock_waveform_memory().await.unwrap();
    haptics.enable().await.unwrap();

    // Clear any stale events
    let _ = haptics.get_events().await;

    info!("RTWM enabled. Playing effects...");

    loop {
        for n in 0..DA7280_WAVEFORM_MEMORY.num_sequences() {
            info!("Sequence: {}", n);
            haptics.play_sequence(n, 0).await.unwrap();
            Timer::after_millis(5_000).await;
        }

        if !common::demo::handle_faults(&mut haptics).await {
            info!("All effects OK");
        }

        Timer::after_millis(1000).await;
    }
}

embassy_rp::bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
});
