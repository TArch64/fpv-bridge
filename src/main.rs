//! # FPV Bridge
//!
//! Control your FPV drone with a PS5 DualSense controller via ExpressLRS.
//!
//! This application bridges PS5 controller inputs to CRSF (Crossfire) protocol
//! for controlling ExpressLRS-enabled drones.

use anyhow::Result;
use tokio::time::{interval, Duration};
use tracing::info;
use tracing_subscriber;

mod config;
mod error;
mod crsf;
mod controller;
mod serial;
mod telemetry;

use crsf::encoder::encode_rc_channels_frame;
use crsf::protocol::{RcChannels, CRSF_CHANNEL_VALUE_CENTER};

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

    // Load configuration
    let config = match config::Config::load("config/default.toml") {
        Ok(cfg) => {
            info!("Loaded configuration from config/default.toml");
            cfg
        }
        Err(e) => {
            info!("Could not load config file ({}), using defaults", e);
            config::Config::default()
        }
    };

    info!("CRSF packet rate: {} Hz", config.crsf.packet_rate_hz);

    // Calculate interval between packets
    let packet_interval_ms = 1000 / config.crsf.packet_rate_hz;
    let mut ticker = interval(Duration::from_millis(packet_interval_ms as u64));

    // Generate dummy channel data (all centered)
    let channels: RcChannels = [CRSF_CHANNEL_VALUE_CENTER; 16];

    info!("Starting main loop with dummy channel data (all centered at {})", CRSF_CHANNEL_VALUE_CENTER);
    info!("Press Ctrl+C to exit");

    // Main loop
    let mut frame_count = 0u64;
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                // Encode CRSF frame
                let frame = encode_rc_channels_frame(&channels);

                // Print frame (in hex format)
                if frame_count % 50 == 0 {  // Print every 50th frame to avoid spam
                    info!("Frame {}: {} bytes - {:02X?}", frame_count, frame.len(), &frame[..8.min(frame.len())]);
                }

                frame_count += 1;
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Received Ctrl+C, shutting down...");
                break;
            }
        }
    }

    info!("Sent {} frames total", frame_count);
    info!("Shutdown complete");

    Ok(())
}
