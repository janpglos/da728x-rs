//! Simple DRO (Direct Register Override) example for DA7280 on nRF54L15.
//!
//! This example demonstrates basic haptic pulses using DRO mode with
//! frequency tracking. This is the simplest way to generate haptic feedback.
//!
//! # Hardware Setup
//!
//! - DA7280 connected via TWIM (SERIAL20, SDA: P1.10, SCL: P1.11)
//! - I2C address: 0x4A (default)
//!
//! # Important: Actuator Loading
//!
//! The LRA actuator **must be mechanically loaded** (compressed between two
//! surfaces) for proper operation.
//!
//! # Running
//!
//! ```bash
//! cargo run --release --example simple_dro
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

    info!("=== Simple DRO Example (nRF54L15) ===");
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
        common::config::dro_frequency_track(),
    ).await.unwrap();
    haptics.enable().await.unwrap();
    info!("DRO mode enabled.");

    common::demo::run_dro_demo(&mut haptics).await;
}
