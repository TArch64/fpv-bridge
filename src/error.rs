//! # Error Types
//!
//! Custom error types for FPV Bridge using `thiserror`.

use thiserror::Error;

/// Main error type for FPV Bridge
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum FpvBridgeError {
    /// Serial port errors
    #[error("Serial port error: {0}")]
    Serial(#[from] tokio_serial::Error),

    /// Controller not found
    #[error("Controller not found: {0}")]
    ControllerNotFound(String),

    /// evdev errors
    #[error("Input device error: {0}")]
    InputDevice(#[from] evdev::Error),

    /// CRSF protocol errors
    #[error("CRSF protocol error: {0}")]
    CrsfProtocol(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] toml::de::Error),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Telemetry log errors
    #[error("Telemetry error: {0}")]
    Telemetry(String),

    /// Channel send errors
    #[error("Channel send error")]
    ChannelSend,

    /// Channel receive errors
    #[error("Channel receive error")]
    ChannelReceive,
}

/// Result type alias for FPV Bridge
pub type Result<T> = std::result::Result<T, FpvBridgeError>;
