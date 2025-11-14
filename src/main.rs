//! # FPV Bridge
//!
//! Control your FPV drone with a PS5 DualSense controller via ExpressLRS.
//!
//! This application bridges PS5 controller inputs to CRSF (Crossfire) protocol
//! for controlling ExpressLRS-enabled drones.

use anyhow::Result;
use tracing::{info, warn};
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
    let mut serial = match ElrsSerial::open().await {
        Ok(s) => {
            info!("ELRS serial port opened at: {}", s.device_path());
            s
        }
        Err(e) => {
            warn!("Failed to open ELRS device: {}", e);
            warn!("Make sure your ELRS module is connected via USB");
            warn!("This is a test build - continuing without serial port");

            // Wait for Ctrl+C and exit
            tokio::signal::ctrl_c().await?;
            return Ok(());
        }
    };

    // Send a test packet with all channels centered
    info!("Sending test CRSF packet (all channels centered at 1024)...");
    let test_channels = [CRSF_CHANNEL_VALUE_CENTER; 16];
    let test_packet = encode_rc_channels_frame(&test_channels);

    serial.send_packet(&test_packet).await?;
    info!("Test packet sent successfully!");

    // TODO: Initialize telemetry logger
    // TODO: Spawn async tasks
    // TODO: Wait for shutdown signal

    info!("FPV Bridge initialized successfully");
    info!("Press Ctrl+C to exit");

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;

    info!("Shutting down...");

    Ok(())
}
