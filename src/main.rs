//! # FPV Bridge
//!
//! Control your FPV drone with a PS5 DualSense controller via ExpressLRS.
//!
//! This application bridges PS5 controller inputs to CRSF (Crossfire) protocol
//! for controlling ExpressLRS-enabled drones.

use anyhow::Result;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};
use tracing_subscriber;

mod config;
mod controller;
mod crsf;
mod error;
mod serial;
mod telemetry;

use config::Config;
use controller::calibration::{
    normalize_axis, normalize_trigger, to_crsf_channel, trigger_to_crsf_channel, AxisCalibration,
};
use controller::channel_mapper::{channels, ChannelMapper};
use controller::mapper::EventMapper;
use controller::ps5::DualSenseController;
use crsf::encoder::encode_rc_channels_frame;
use crsf::protocol::{RcChannels, CRSF_CHANNEL_VALUE_CENTER};
use serial::ElrsSerial;

/// Default packet transmission rate in Hz (ELRS standard)
///
/// ExpressLRS uses 250Hz packet rate for control commands, resulting in
/// a 4ms period between packets. This ensures responsive control with
/// low latency suitable for FPV drone racing and freestyle.
const PACKET_RATE_HZ: u32 = 250;

/// Number of packets between status log messages
///
/// At 250Hz, logging every 1000 packets results in status updates
/// approximately every 4 seconds, providing visibility without
/// flooding the logs.
const LOG_INTERVAL_PACKETS: u64 = 1000;

/// Consecutive failure threshold before escalating to warning level
///
/// When packet transmission fails 10 times consecutively, logging
/// escalates from debug to warning level to alert of persistent
/// connectivity issues that may require intervention.
const FAILURE_WARNING_THRESHOLD: u32 = 10;

/// Channel buffer size for controller state communication
///
/// Buffer holds the latest channel values from controller task.
/// Size of 1 means we only keep the most recent state, dropping
/// older values if main loop is slower than controller updates.
const CHANNEL_BUFFER_SIZE: usize = 1;

/// Controller task that reads PS5 input and sends calibrated RC channels
///
/// Runs in a separate async task, continuously reading controller events,
/// applying calibration and mapping to CRSF channels, then sending via mpsc.
///
/// # Arguments
///
/// * `tx` - Channel sender for transmitting RC channel values
/// * `calibration` - Axis calibration settings for deadzones and expo
/// * `mapper` - Channel mapper for reversals and button mapping
async fn controller_task(
    tx: mpsc::Sender<RcChannels>,
    calibration: AxisCalibration,
    _mapper: ChannelMapper,
) -> Result<()> {
    info!("Controller task starting");

    // Open PS5 controller
    let mut controller = DualSenseController::open()?;
    info!("PS5 controller connected: {}", controller.device_path());

    // Create event mapper
    let mut event_mapper = EventMapper::new();

    // Continuously read and process controller events
    loop {
        // Fetch events from controller
        match controller.fetch_events() {
            Ok(events) => {
                for event in events {
                    event_mapper.process_event(&event);
                }

                // Get current controller state
                let state = event_mapper.state();

                // Convert raw inputs to calibrated CRSF channels
                let mut channels = [CRSF_CHANNEL_VALUE_CENTER; 16];

                // Roll (right stick X)
                let roll_norm = normalize_axis(state.right_stick_x);
                let roll_cal = calibration.roll.apply(roll_norm);
                channels[channels::ROLL] = to_crsf_channel(roll_cal);

                // Pitch (right stick Y) - inverted
                let pitch_raw = 255 - state.right_stick_y; // Invert: up = forward
                let pitch_norm = normalize_axis(pitch_raw);
                let pitch_cal = calibration.pitch.apply(pitch_norm);
                channels[channels::PITCH] = to_crsf_channel(pitch_cal);

                // Throttle (left stick Y) - inverted
                let throttle_raw = 255 - state.left_stick_y; // Invert: up = high
                let throttle_norm = normalize_axis(throttle_raw);
                let throttle_cal = calibration.throttle.apply(throttle_norm);
                channels[channels::THROTTLE] = to_crsf_channel(throttle_cal);

                // Yaw (left stick X)
                let yaw_norm = normalize_axis(state.left_stick_x);
                let yaw_cal = calibration.yaw.apply(yaw_norm);
                channels[channels::YAW] = to_crsf_channel(yaw_cal);

                // ARM (L1 button)
                channels[channels::ARM] = if state.btn_l1 { 2047 } else { 0 };

                // Flight Mode (R1 button)
                channels[channels::FLIGHT_MODE] = if state.btn_r1 { 2047 } else { 0 };

                // Beeper (L2 trigger)
                let l2_norm = normalize_trigger(state.trigger_l2);
                let l2_cal = calibration.apply_trigger(l2_norm);
                channels[channels::BEEPER] = trigger_to_crsf_channel(l2_cal);

                // Turtle Mode (R2 trigger)
                let r2_norm = normalize_trigger(state.trigger_r2);
                let r2_cal = calibration.apply_trigger(r2_norm);
                channels[channels::TURTLE] = trigger_to_crsf_channel(r2_cal);

                // Apply channel reversals if configured
                // (mapper.map_to_channels would handle this, but we're doing manual mapping here)
                // For now, skip reversals - can be added later

                // Send channels to main loop (non-blocking)
                if tx.try_send(channels).is_err() {
                    // Channel full - main loop hasn't consumed previous value yet
                    // This is fine, we'll just drop this update and send the next one
                    debug!("Channel full, dropping update");
                }
            }
            Err(e) => {
                error!("Failed to fetch controller events: {}", e);
                // Return error to signal controller disconnection
                return Err(e.into());
            }
        }

        // Small yield to prevent busy-waiting
        tokio::task::yield_now().await;
    }
}

/// Main entry point for FPV Bridge application
///
/// Initializes serial communication with ELRS module and runs the main control loop
/// that continuously sends CRSF packets at 250Hz (ELRS standard rate).
///
/// # Current Implementation
///
/// - Reads PS5 DualSense controller input
/// - Applies calibration (deadzones and expo curves)
/// - Maps controller inputs to CRSF RC channels
/// - Sends CRSF packets at 250Hz to ELRS module
/// - Logs status every 1000 packets (~4 seconds)
/// - Handles Ctrl+C for graceful shutdown
/// - Tracks consecutive transmission failures with warning escalation
///
/// # Errors
///
/// Returns error if:
/// - Configuration file is invalid
/// - Serial port cannot be opened (no ELRS device found)
/// - PS5 controller cannot be opened
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    info!("FPV Bridge v{} starting...", env!("CARGO_PKG_VERSION"));

    // Load configuration (use defaults if file doesn't exist)
    let config = match Config::load("config/default.toml") {
        Ok(cfg) => {
            info!("Loaded configuration from config/default.toml");
            cfg
        }
        Err(e) => {
            warn!("Failed to load config file, using defaults: {}", e);
            // Create default config
            Config {
                serial: config::SerialConfig {
                    port: "/dev/ttyACM0".to_string(),
                    baud_rate: 420000,
                    timeout_ms: 100,
                    reconnect_interval_ms: 1000,
                },
                controller: config::ControllerConfig {
                    device_path: String::new(),
                    deadzone_stick: 0.05,
                    deadzone_trigger: 0.10,
                    expo_roll: 0.3,
                    expo_pitch: 0.3,
                    expo_yaw: 0.2,
                    expo_throttle: 0.0,
                },
                channels: config::ChannelConfig {
                    throttle_min: 1000,
                    throttle_max: 2000,
                    center: 1500,
                    channel_reverse: vec![],
                },
                telemetry: config::TelemetryConfig {
                    enabled: true,
                    log_dir: "./logs".to_string(),
                    max_records_per_file: 10000,
                    max_files_to_keep: 10,
                    log_interval_ms: 100,
                    format: "jsonl".to_string(),
                },
                safety: config::SafetyConfig {
                    arm_button_hold_ms: 1000,
                    auto_disarm_timeout_s: 300,
                    failsafe_timeout_ms: 500,
                    min_throttle_to_arm: 1050,
                },
                crsf: config::CrsfConfig {
                    packet_rate_hz: 250,
                    link_stats_interval_ms: 1000,
                },
            }
        }
    };

    // Create calibration from config
    let calibration = AxisCalibration::from_config(
        config.controller.deadzone_stick,
        config.controller.deadzone_trigger,
        config.controller.expo_roll,
        config.controller.expo_pitch,
        config.controller.expo_yaw,
        config.controller.expo_throttle,
    );
    info!(
        "Calibration: stick_deadzone={:.3}, trigger_deadzone={:.3}, expo=(roll={:.2}, pitch={:.2}, yaw={:.2}, throttle={:.2})",
        config.controller.deadzone_stick,
        config.controller.deadzone_trigger,
        config.controller.expo_roll,
        config.controller.expo_pitch,
        config.controller.expo_yaw,
        config.controller.expo_throttle,
    );

    // Create channel mapper with reversed channels
    let mapper = if config.channels.channel_reverse.is_empty() {
        ChannelMapper::new()
    } else {
        ChannelMapper::with_reversed(&config.channels.channel_reverse)
    };

    // Initialize serial communication
    let mut serial = ElrsSerial::open()?;
    info!("ELRS serial port opened at: {}", serial.device_path());

    // Create channel for controller → main loop communication
    let (tx, mut rx) = mpsc::channel::<RcChannels>(CHANNEL_BUFFER_SIZE);

    // Spawn controller task
    let mut controller_handle = tokio::spawn(controller_task(tx, calibration, mapper));

    // Initialize with centered channels
    let mut current_channels = [CRSF_CHANNEL_VALUE_CENTER; 16];

    // Create 250Hz interval (4ms period)
    let period_ms = 1000 / PACKET_RATE_HZ;
    let mut packet_interval = interval(Duration::from_millis(period_ms as u64));
    // Skip missed ticks to prevent burst sends after delays
    packet_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    info!("Starting CRSF packet transmission loop at {}Hz", PACKET_RATE_HZ);
    info!("Press Ctrl+C to exit");

    let mut packet_count: u64 = 0;
    let mut last_log_count: u64 = 0;
    let mut consecutive_failures: u32 = 0;

    // Main control loop
    loop {
        tokio::select! {
            // Send packet at regular interval
            _ = packet_interval.tick() => {
                // Try to receive latest channels from controller
                // (non-blocking - use most recent value if available)
                while let Ok(channels) = rx.try_recv() {
                    current_channels = channels;
                }

                // Encode and send CRSF packet
                let packet = encode_rc_channels_frame(&current_channels);

                if let Err(e) = serial.send_packet(&packet).await {
                    consecutive_failures += 1;

                    if consecutive_failures >= FAILURE_WARNING_THRESHOLD {
                        warn!("Failed to send packet (consecutive failures: {}): {}", consecutive_failures, e);
                    } else {
                        debug!("Failed to send packet: {}", e);
                    }
                    continue;
                }

                // Reset failure counter on successful transmission
                consecutive_failures = 0;
                packet_count += 1;

                // Log status every LOG_INTERVAL_PACKETS (~4 seconds at 250Hz)
                if packet_count - last_log_count >= LOG_INTERVAL_PACKETS {
                    info!("Sent {} packets ({}Hz) - Throttle={} Roll={} Pitch={} Yaw={} ARM={}",
                        packet_count,
                        PACKET_RATE_HZ,
                        current_channels[channels::THROTTLE],
                        current_channels[channels::ROLL],
                        current_channels[channels::PITCH],
                        current_channels[channels::YAW],
                        current_channels[channels::ARM],
                    );
                    last_log_count = packet_count;
                }
            }

            // Handle controller task completion (error or exit)
            result = &mut controller_handle => {
                match result {
                    Ok(Ok(())) => {
                        info!("Controller task exited normally");
                    }
                    Ok(Err(e)) => {
                        error!("Controller task failed: {}", e);
                        return Err(e);
                    }
                    Err(e) => {
                        error!("Controller task panicked: {}", e);
                        return Err(e.into());
                    }
                }
                break;
            }

            // Handle Ctrl+C for graceful shutdown
            _ = tokio::signal::ctrl_c() => {
                info!("Received Ctrl+C, shutting down...");
                info!("Total packets sent: {}", packet_count);
                break;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_rate_constant() {
        // Verify ELRS standard packet rate
        assert_eq!(PACKET_RATE_HZ, 250, "Packet rate should be 250Hz (ELRS standard)");
    }

    #[test]
    fn test_log_interval_constant() {
        // Verify log interval is reasonable
        assert_eq!(LOG_INTERVAL_PACKETS, 1000);

        // At 250Hz, 1000 packets = 4 seconds
        let seconds = LOG_INTERVAL_PACKETS as f64 / PACKET_RATE_HZ as f64;
        assert_eq!(seconds, 4.0, "Log interval should be 4 seconds at 250Hz");
    }

    #[test]
    fn test_packet_period_calculation() {
        // Verify period calculation is correct
        let period_ms = 1000 / PACKET_RATE_HZ;
        assert_eq!(period_ms, 4, "Period should be 4ms at 250Hz");
    }

    #[test]
    fn test_dummy_channels_are_centered() {
        // Verify dummy values match CRSF center position
        let dummy_channels = [CRSF_CHANNEL_VALUE_CENTER; 16];
        assert_eq!(dummy_channels.len(), 16, "Should have 16 channels");
        for &channel in &dummy_channels {
            assert_eq!(channel, CRSF_CHANNEL_VALUE_CENTER, "All channels should be centered");
        }
    }

    #[test]
    fn test_failure_warning_threshold() {
        // Verify failure threshold is reasonable
        assert_eq!(FAILURE_WARNING_THRESHOLD, 10);

        // At 250Hz, 10 failures = 40ms of consecutive failures
        // This is a reasonable threshold before escalating to warnings
        let failure_duration_ms = FAILURE_WARNING_THRESHOLD * 4; // 4ms per packet at 250Hz
        assert_eq!(failure_duration_ms, 40, "Should tolerate 40ms of failures before warning");
    }

    #[test]
    fn test_constants_are_consistent() {
        // Verify that constants work together logically

        // Packet rate and period
        let period_ms = 1000 / PACKET_RATE_HZ;
        assert_eq!(period_ms, 4, "250Hz rate should result in 4ms period");

        // Log interval timing
        let log_interval_seconds = LOG_INTERVAL_PACKETS as f64 / PACKET_RATE_HZ as f64;
        assert_eq!(log_interval_seconds, 4.0, "Should log every 4 seconds");

        // Failure threshold timing
        let failure_threshold_ms = FAILURE_WARNING_THRESHOLD * period_ms;
        assert_eq!(failure_threshold_ms, 40, "Should warn after 40ms of failures");

        // Sanity checks
        assert!(PACKET_RATE_HZ > 0, "Packet rate must be positive");
        assert!(LOG_INTERVAL_PACKETS > 0, "Log interval must be positive");
        assert!(FAILURE_WARNING_THRESHOLD > 0, "Failure threshold must be positive");
    }

    #[test]
    fn test_elrs_standard_packet_rate() {
        // ExpressLRS standard specifies 250Hz for RC channels
        // This is critical for proper operation
        assert_eq!(PACKET_RATE_HZ, 250,
            "ELRS requires 250Hz packet rate for RC channels");

        // Verify period calculation
        let period_ms = 1000 / PACKET_RATE_HZ;
        assert_eq!(period_ms, 4,
            "250Hz should result in exactly 4ms period per packet");
    }
}
