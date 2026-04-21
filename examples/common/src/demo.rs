use defmt::{info, warn};
use embassy_time::Timer;
use embedded_hal_async::i2c::I2c;

use da728x::DA728x;

/// Play a melody on the haptic driver using DRO mode with wideband.
///
/// The device must be configured in DRO mode with `DrivingMode::WIDEBAND`
/// and enabled before calling this function.
pub async fn play_melody<I2C: I2c>(
    haptics: &mut DA728x<I2C>,
    melody: &[(u16, u64)],
    gap_ms: u64,
) {
    for &(freq, duration) in melody.iter() {
        haptics.set_frequency(freq).await.unwrap();
        haptics.set_override_value(127).await.unwrap();
        Timer::after_millis(duration).await;
        haptics.set_override_value(0).await.unwrap();
        Timer::after_millis(gap_ms).await;
    }
}

/// Check for faults and attempt recovery via EMBEDDED_MODE auto-clear.
///
/// Returns `true` if a fault was detected and recovery was performed.
/// EMBEDDED_MODE must be enabled (it is by default when using the shared configs).
pub async fn handle_faults<I2C: I2c>(haptics: &mut DA728x<I2C>) -> bool {
    let (events, warnings, seq_diag) = haptics.get_events().await.unwrap();
    let mut needs_recovery = false;

    if events.E_OC_FAULT() {
        warn!("OVERCURRENT FAULT!");
        needs_recovery = true;
    }
    if events.E_ACTUATOR_FAULT() {
        warn!("ACTUATOR FAULT - Is the actuator loaded?");
        needs_recovery = true;
    }
    if events.E_SEQ_FAULT() {
        warn!("Sequence fault: {:?}", seq_diag);
        needs_recovery = true;
    }
    if events.E_WARNING() {
        warn!("Warning: {:?}", warnings);
    }

    if needs_recovery {
        info!("Auto-recovering via EMBEDDED_MODE...");
        haptics.disable().await.unwrap();
        Timer::after_millis(50).await;
        haptics.enable().await.unwrap();
        info!("Recovery complete");
    }

    needs_recovery
}

/// Run a simple DRO demo loop: pulse on/off with fault recovery.
///
/// The device must be configured and enabled before calling this function.
pub async fn run_dro_demo<I2C: I2c>(haptics: &mut DA728x<I2C>) {
    loop {
        info!("Pulse!");
        haptics.set_override_value(127).await.unwrap();
        Timer::after_millis(100).await;
        haptics.set_override_value(0).await.unwrap();
        Timer::after_millis(400).await;
        handle_faults(haptics).await;
    }
}

/// Run a melody demo loop with fault recovery.
///
/// The device must be configured in DRO mode with `DrivingMode::WIDEBAND`
/// and enabled before calling this function.
pub async fn run_melody_demo<I2C: I2c>(haptics: &mut DA728x<I2C>, melody: &[(u16, u64)]) {
    loop {
        info!("Playing melody...");
        play_melody(haptics, melody, 50).await;
        info!("Melody complete!");
        handle_faults(haptics).await;
        Timer::after_millis(2000).await;
    }
}
