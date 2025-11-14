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
    let packet_rate_hz = 250;
    let period_ms = 1000 / packet_rate_hz;
    let mut packet_interval = interval(Duration::from_millis(period_ms));

    info!("Starting CRSF packet transmission loop at {}Hz", packet_rate_hz);
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

                // Log status every 1000 packets (~4 seconds at 250Hz)
                if packet_count - last_log_count >= 1000 {
                    info!("Sent {} packets ({}Hz, all channels centered at {})",
                        packet_count, packet_rate_hz, CRSF_CHANNEL_VALUE_CENTER);
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
