//! # FPV Bridge
//!
//! Control your FPV drone with a PS5 DualSense controller via ExpressLRS.
//!
//! This application bridges PS5 controller inputs to CRSF (Crossfire) protocol
//! for controlling ExpressLRS-enabled drones.

use anyhow::Result;
use tokio::time::{interval, Duration};
use tracing::{debug, info};
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
const PACKET_RATE_HZ: u32 = 250;

/// Number of packets between status log messages
const LOG_INTERVAL_PACKETS: u64 = 1000;

/// Main entry point for FPV Bridge application
///
/// Initializes the application and runs the main control loop that continuously
/// sends CRSF packets to the ELRS transmitter module at 250Hz.
///
/// # Control Flow
///
/// 1. **Initialization**
///    - Set up logging with tracing subscriber
///    - Open serial connection to ELRS module
///    - Configure 250Hz packet transmission interval
///
/// 2. **Main Loop**
///    - Send CRSF packets at 250Hz with dummy channel values (all centered)
///    - Log status every 1000 packets (~4 seconds)
///    - Handle Ctrl+C for graceful shutdown
///
/// 3. **Graceful Shutdown**
///    - Stop packet transmission
///    - Log total packet count
///    - Clean exit
///
/// # Current Behavior
///
/// In this phase, all 16 RC channels are set to center position (1024).
/// This provides a continuous, valid CRSF packet stream to the ELRS module
/// without requiring controller input. The drone will receive neutral
/// stick positions.
///
/// # Future Phases
///
/// - Phase 3: Replace dummy values with PS5 controller input
/// - Phase 4: Add telemetry reception
/// - Phase 5: Implement safety features and failsafe
///
/// # Errors
///
/// Returns error if:
/// - Serial port cannot be opened (no ELRS device found)
/// - Critical transmission failures occur
///
/// # Examples
///
/// Run the application:
/// ```bash
/// cargo run --release
/// ```
///
/// Expected output:
/// ```text
/// INFO fpv_bridge: FPV Bridge v0.1.0 starting...
/// INFO fpv_bridge::serial: Successfully opened ELRS device at /dev/ttyACM0
/// INFO fpv_bridge: Starting CRSF packet transmission loop at 250Hz
/// INFO fpv_bridge: Sent 1000 packets (250Hz, all channels centered at 1024)
/// ```
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

    info!("Starting CRSF packet transmission loop at {}Hz", PACKET_RATE_HZ);
    info!("Press Ctrl+C to exit");

    let mut packet_count: u64 = 0;
    let mut last_log_count: u64 = 0;

    // Main control loop
    loop {
        tokio::select! {
            // Send packet at regular interval
            _ = packet_interval.tick() => {
                // Encode and send CRSF packet with dummy values
                let packet = encode_rc_channels_frame(&dummy_channels);

                if let Err(e) = serial.send_packet(&packet).await {
                    debug!("Failed to send packet: {}", e);
                    continue;
                }

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
            assert_eq!(channel, 1024, "All channels should be centered at 1024");
        }
    }
}
