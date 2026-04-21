//! Wideband DRO example playing the Tetris melody on nRF54L15.
//!
//! This example demonstrates using WIDEBAND mode to play different frequencies,
//! creating a simple melody using the haptic actuator as a speaker/buzzer.
//!
//! # Hardware Setup
//!
//! - DA7280 connected via TWIM (SERIAL20, SDA: P1.10, SCL: P1.11)
//! - I2C address: 0x4A (default)
//!
//! # WIDEBAND vs FREQUENCY_TRACK
//!
//! - **FREQUENCY_TRACK**: Locks onto the LRA's resonant frequency (~170 Hz) for
//!   maximum efficiency. Cannot play arbitrary frequencies.
//! - **WIDEBAND**: Allows driving the actuator at any frequency, enabling melodies
//!   but with less efficient energy transfer.
//!
//! # Running
//!
//! ```bash
//! cargo run --release --example wideband_melody
//! ```

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{bind_interrupts, peripherals};
use static_cell::ConstStaticCell;
use {defmt_rtt as _, panic_probe as _};

use da728x::{Variant, DA728x};
use da728x_examples_common as common;

bind_interrupts!(struct Irqs {
    SERIAL20 => twim::InterruptHandler<peripherals::SERIAL20>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    info!("=== Wideband Melody Example (nRF54L15) ===");
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

    haptics.configure(
        common::config::sparkfun_lra_config(),
        common::config::dro_wideband(),
    ).await.unwrap();
    haptics.enable().await.unwrap();
    info!("Wideband DRO mode enabled.");

    common::demo::run_melody_demo(&mut haptics, common::melody::TETRIS_MELODY).await;
}
