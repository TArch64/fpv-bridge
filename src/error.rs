//! # Error Types
//!
//! Custom error types for FPV Bridge using `thiserror`.

use thiserror::Error;

/// Main error type for FPV Bridge
#[derive(Debug, Error)]
pub enum FpvBridgeError {
    /// CRSF protocol errors
    #[error("CRSF protocol error: {0}")]
    CrsfProtocol(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] toml::de::Error),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type alias for FPV Bridge
pub type Result<T> = std::result::Result<T, FpvBridgeError>;
