//! # FPV Bridge
//!
//! Control your FPV drone with a PS5 DualSense controller via ExpressLRS.
//!
//! This application bridges PS5 controller inputs to CRSF (Crossfire) protocol
//! for controlling ExpressLRS-enabled drones.

use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber;

mod config;
mod error;
mod crsf;
mod controller;
mod serial;
mod telemetry;

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
    // TODO: Initialize serial communication
    // TODO: Initialize telemetry logger
    // TODO: Spawn async tasks
    // TODO: Wait for shutdown signal

    info!("FPV Bridge initialized successfully");

    // Placeholder: wait for Ctrl+C
    tokio::signal::ctrl_c().await?;

    info!("Shutting down...");

    Ok(())
}
