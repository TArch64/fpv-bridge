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

    /// Serial port errors
    #[error("Serial port error: {0}")]
    Serial(String),

    /// Serial port not found
    #[error("No ELRS device found. Tried: {0}")]
    SerialPortNotFound(String),
}

/// Result type alias for FPV Bridge
pub type Result<T> = std::result::Result<T, FpvBridgeError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_crsf_protocol_error_message() {
        let error = FpvBridgeError::CrsfProtocol("invalid sync byte".to_string());
        let message = error.to_string();
        assert!(message.contains("CRSF protocol error"));
        assert!(message.contains("invalid sync byte"));
    }

    #[test]
    fn test_serial_error_message() {
        let error = FpvBridgeError::Serial("write failed".to_string());
        let message = error.to_string();
        assert!(message.contains("Serial port error"));
        assert!(message.contains("write failed"));
    }

    #[test]
    fn test_serial_port_not_found_message() {
        let error = FpvBridgeError::SerialPortNotFound("/dev/ttyACM0, /dev/ttyUSB0".to_string());
        let message = error.to_string();
        assert!(message.contains("No ELRS device found"));
        assert!(message.contains("/dev/ttyACM0"));
        assert!(message.contains("/dev/ttyUSB0"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let error: FpvBridgeError = io_error.into();

        match error {
            FpvBridgeError::Io(_) => {
                let message = error.to_string();
                assert!(message.contains("I/O error"));
            }
            _ => panic!("Expected Io variant"),
        }
    }

    #[test]
    fn test_error_debug_format() {
        let error = FpvBridgeError::Serial("test error".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Serial"));
        assert!(debug_str.contains("test error"));
    }
}
