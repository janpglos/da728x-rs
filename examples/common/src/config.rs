use da728x::config::{ActuatorConfig, ActuatorType, DeviceConfig, DrivingMode, OperationMode};

/// Actuator config for the G1040003D LRA on the SparkFun Qwiic Haptic Driver (ROB-17590).
///
/// Values from the datasheet for the DA7280 evaluation setup.
pub fn sparkfun_lra_config() -> ActuatorConfig {
    ActuatorConfig {
        actuator_type: ActuatorType::LRA,
        nominal_max_mV: 2_106,
        absolute_max_mV: 2_260,
        max_current_mA: 165,
        impedance_mOhm: 13_800,
        inductance_uH: 50,
        frequency_Hz: 170,
        pid_Kp: None,
        pid_Ki: None,
    }
}

/// DRO mode with frequency tracking (locks to LRA resonant frequency).
pub fn dro_frequency_track() -> DeviceConfig {
    DeviceConfig {
        operation_mode: OperationMode::DRO_MODE,
        driving_mode: DrivingMode::FREQUENCY_TRACK,
        acceleration: false,
        rapid_stop: false,
    }
}

/// DRO mode with wideband (allows arbitrary frequencies for melodies).
pub fn dro_wideband() -> DeviceConfig {
    DeviceConfig {
        operation_mode: OperationMode::DRO_MODE,
        driving_mode: DrivingMode::WIDEBAND,
        acceleration: false,
        rapid_stop: false,
    }
}

/// RTWM (Register-Triggered Waveform Memory) mode with frequency tracking.
pub fn rtwm_frequency_track() -> DeviceConfig {
    DeviceConfig {
        operation_mode: OperationMode::RTWM_MODE,
        driving_mode: DrivingMode::FREQUENCY_TRACK,
        acceleration: false,
        rapid_stop: false,
    }
}
