//! # FPV Bridge
//!
//! Control your FPV drone with a PS5 DualSense controller via ExpressLRS.
//!
//! This application bridges PS5 controller inputs to CRSF (Crossfire) protocol
//! for controlling ExpressLRS-enabled drones.

use anyhow::Result;
use tokio::time::{interval, Duration};
use tracing::{debug, info, warn};
use tracing_subscriber;

mod config;
mod error;
mod crsf;
mod controller;
mod serial;
mod telemetry;

use crsf::encoder::encode_rc_channels_frame;
use crsf::protocol::CRSF_CHANNEL_VALUE_CENTER;
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

/// Main entry point for FPV Bridge application
///
/// Initializes serial communication with ELRS module and runs the main control loop
/// that continuously sends CRSF packets at 250Hz (ELRS standard rate).
///
/// # Current Implementation (Phase 2)
///
/// - Sends dummy channel values (all centered at 1024) at 250Hz
/// - Logs status every 1000 packets (~4 seconds)
/// - Handles Ctrl+C for graceful shutdown
/// - Tracks consecutive transmission failures with warning escalation
///
/// # Errors
///
/// Returns error if serial port cannot be opened (no ELRS device found)
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

    // TODO: Load configuration
    // TODO: Initialize controller handler

    // Initialize serial communication
    let mut serial = ElrsSerial::open()?;
    info!("ELRS serial port opened at: {}", serial.device_path());

    // Create dummy channel values (all centered)
    // In the next phase, these will be replaced with actual controller input
    let dummy_channels = [CRSF_CHANNEL_VALUE_CENTER; 16];

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
                // Encode and send CRSF packet with dummy values
                let packet = encode_rc_channels_frame(&dummy_channels);

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
                    info!("Sent {} packets ({}Hz, all channels centered at {})",
                        packet_count, PACKET_RATE_HZ, CRSF_CHANNEL_VALUE_CENTER);
                    last_log_count = packet_count;
                }
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
}
