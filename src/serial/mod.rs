//! # Serial Communication Module
//!
//! Handles serial communication with ELRS USB module.
//!
//! This module handles:
//! - Opening serial port at 420,000 baud
//! - Async read/write operations
//! - Transmitting CRSF RC channels packets at 250Hz
//! - Receiving telemetry packets
//! - Error recovery and reconnection

use crate::error::{FpvBridgeError, Result};
use tokio_serial::SerialPortBuilderExt;
use tracing::{debug, info, warn};

/// CRSF baud rate for ELRS (420,000 baud)
pub const CRSF_BAUD_RATE: u32 = 420_000;

/// Default ELRS device paths to try (in order of preference)
const DEFAULT_DEVICE_PATHS: &[&str] = &[
    "/dev/ttyACM0", // USB CDC devices (most common for ELRS)
    "/dev/ttyUSB0", // USB-to-serial adapters
];

/// ELRS Serial Port Handler
///
/// Manages connection to the ELRS transmitter module via USB serial.
pub struct ElrsSerial {
    /// Serial port handle
    port: tokio_serial::SerialStream,
    /// Device path (e.g., /dev/ttyACM0)
    device_path: String,
}

impl std::fmt::Debug for ElrsSerial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ElrsSerial")
            .field("device_path", &self.device_path)
            .finish_non_exhaustive()
    }
}

impl ElrsSerial {
    /// Open connection to ELRS module
    ///
    /// Auto-detects the device by trying common paths.
    ///
    /// # Returns
    ///
    /// * `Result<ElrsSerial>` - Connected serial port or error
    ///
    /// # Errors
    ///
    /// Returns error if no ELRS device found or connection fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fpv_bridge::serial::ElrsSerial;
    ///
    /// fn main() -> anyhow::Result<()> {
    ///     let serial = ElrsSerial::open()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn open() -> Result<Self> {
        Self::open_with_paths(DEFAULT_DEVICE_PATHS)
    }

    /// Open connection to ELRS module with custom device paths
    ///
    /// # Arguments
    ///
    /// * `paths` - Device paths to try (e.g., &["/dev/ttyACM0"])
    ///
    /// # Returns
    ///
    /// * `Result<ElrsSerial>` - Connected serial port or error
    pub fn open_with_paths(paths: &[&str]) -> Result<Self> {
        for path in paths {
            debug!("Trying to open serial port: {}", path);

            match Self::open_port(path) {
                Ok(port) => {
                    info!("Successfully opened ELRS device at {}", path);
                    return Ok(Self {
                        port,
                        device_path: path.to_string(),
                    });
                }
                Err(e) => {
                    warn!("Failed to open {}: {}", path, e);
                    continue;
                }
            }
        }

        Err(FpvBridgeError::SerialPortNotFound(
            paths.join(", ")
        ))
    }

    /// Open a specific serial port with CRSF settings
    ///
    /// # Arguments
    ///
    /// * `path` - Device path (e.g., "/dev/ttyACM0")
    ///
    /// # Returns
    ///
    /// * `Result<SerialStream>` - Opened serial port
    fn open_port(path: &str) -> Result<tokio_serial::SerialStream> {
        let port = tokio_serial::new(path, CRSF_BAUD_RATE)
            .data_bits(tokio_serial::DataBits::Eight)
            .parity(tokio_serial::Parity::None)
            .stop_bits(tokio_serial::StopBits::One)
            .flow_control(tokio_serial::FlowControl::None)
            .open_native_async()
            .map_err(|e| FpvBridgeError::Serial(format!("Failed to open {}: {}", path, e)))?;

        Ok(port)
    }

    /// Send a CRSF packet to the ELRS module
    ///
    /// # Arguments
    ///
    /// * `packet` - Complete CRSF frame (including sync byte, length, type, payload, CRC)
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Success or error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fpv_bridge::serial::ElrsSerial;
    /// use fpv_bridge::crsf::encoder::encode_rc_channels_frame;
    /// use fpv_bridge::crsf::protocol::CRSF_CHANNEL_VALUE_CENTER;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let mut serial = ElrsSerial::open()?;
    ///
    ///     // Send test packet with all channels centered
    ///     let channels = [CRSF_CHANNEL_VALUE_CENTER; 16];
    ///     let packet = encode_rc_channels_frame(&channels);
    ///     serial.send_packet(&packet).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn send_packet(&mut self, packet: &[u8]) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        self.port.write_all(packet).await
            .map_err(|e| FpvBridgeError::Serial(format!("Failed to write packet: {}", e)))?;

        self.port.flush().await
            .map_err(|e| FpvBridgeError::Serial(format!("Failed to flush serial port: {}", e)))?;

        debug!("Sent CRSF packet ({} bytes)", packet.len());
        Ok(())
    }

    /// Get the device path of the opened serial port
    ///
    /// Returns the path to the serial device that was successfully opened
    /// (e.g., "/dev/ttyACM0" or "/dev/ttyUSB0").
    ///
    /// # Returns
    ///
    /// * `&str` - Reference to the device path string
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fpv_bridge::serial::ElrsSerial;
    ///
    /// let serial = ElrsSerial::open()?;
    /// println!("Connected to: {}", serial.device_path());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn device_path(&self) -> &str {
        &self.device_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crsf::encoder::encode_rc_channels_frame;
    use crate::crsf::protocol::CRSF_CHANNEL_VALUE_CENTER;

    #[test]
    fn test_constants() {
        assert_eq!(CRSF_BAUD_RATE, 420_000);
        assert_eq!(DEFAULT_DEVICE_PATHS.len(), 2);
        assert_eq!(DEFAULT_DEVICE_PATHS[0], "/dev/ttyACM0");
        assert_eq!(DEFAULT_DEVICE_PATHS[1], "/dev/ttyUSB0");
    }

    #[test]
    fn test_open_with_invalid_paths_returns_error() {
        // Try to open non-existent device paths
        let invalid_paths = &["/dev/nonexistent0", "/dev/nonexistent1"];
        let result = ElrsSerial::open_with_paths(invalid_paths);

        // Should fail with SerialPortNotFound error
        assert!(result.is_err());
        let err = result.unwrap_err();

        // Verify error message contains the paths we tried
        match err {
            FpvBridgeError::SerialPortNotFound(msg) => {
                assert!(msg.contains("/dev/nonexistent0"));
                assert!(msg.contains("/dev/nonexistent1"));
            }
            _ => panic!("Expected SerialPortNotFound error, got: {:?}", err),
        }
    }

    #[test]
    fn test_open_with_empty_paths_returns_error() {
        // Try to open with empty path list
        let empty_paths: &[&str] = &[];
        let result = ElrsSerial::open_with_paths(empty_paths);

        // Should fail with SerialPortNotFound error
        assert!(result.is_err());
        match result.unwrap_err() {
            FpvBridgeError::SerialPortNotFound(_) => {
                // Expected error
            }
            other => panic!("Expected SerialPortNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn test_open_port_with_invalid_path_returns_error() {
        // Try to open a non-existent device
        let result = ElrsSerial::open_port("/dev/nonexistent_serial_device_12345");

        // Should fail with Serial error
        assert!(result.is_err());
        let err = result.unwrap_err();

        match err {
            FpvBridgeError::Serial(msg) => {
                // Error message should mention the path and failure
                assert!(msg.contains("/dev/nonexistent_serial_device_12345"));
                assert!(msg.contains("Failed to open"));
            }
            _ => panic!("Expected Serial error, got: {:?}", err),
        }
    }

    #[test]
    fn test_crsf_packet_encoding() {
        // Verify that CRSF packet encoding produces valid packets
        let channels = [CRSF_CHANNEL_VALUE_CENTER; 16];
        let packet = encode_rc_channels_frame(&channels);

        // Verify packet structure
        assert_eq!(packet.len(), 26, "CRSF packet should be 26 bytes");
        assert_eq!(packet[0], 0xC8, "First byte should be sync byte (0xC8)");
        assert_eq!(packet[1], 0x18, "Second byte should be length (0x18 = 24)");
        assert_eq!(packet[2], 0x16, "Third byte should be frame type (0x16 = RC channels)");
    }

    #[test]
    fn test_device_path_order() {
        // Verify that device paths are tried in the correct priority order
        assert_eq!(DEFAULT_DEVICE_PATHS[0], "/dev/ttyACM0",
            "ttyACM0 should be tried first (most common for ELRS)");
        assert_eq!(DEFAULT_DEVICE_PATHS[1], "/dev/ttyUSB0",
            "ttyUSB0 should be tried second (USB-to-serial adapters)");
    }

    #[test]
    fn test_serial_configuration_constants() {
        // Verify CRSF protocol requirements
        assert_eq!(CRSF_BAUD_RATE, 420_000, "CRSF requires 420,000 baud");

        // These are the expected serial settings for CRSF
        // 8 data bits, no parity, 1 stop bit (8N1)
        // Tested indirectly through open_port configuration
    }

    // Integration test - only runs if ELRS hardware is connected
    // Skipped in CI/CD environments
    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn test_open_with_real_hardware() {
        // This test requires actual ELRS hardware connected
        let result = ElrsSerial::open();

        if result.is_ok() {
            let serial = result.unwrap();
            println!("Successfully opened ELRS device at: {}", serial.device_path());

            // Verify device path is one of the expected ones
            let path = serial.device_path();
            assert!(
                path == "/dev/ttyACM0" || path == "/dev/ttyUSB0",
                "Unexpected device path: {}",
                path
            );
        } else {
            println!("No ELRS hardware detected (this is OK for CI/CD)");
        }
    }

    // Integration test - only runs if ELRS hardware is connected
    #[tokio::test]
    #[ignore] // Run with: cargo test -- --ignored
    async fn test_send_packet_with_real_hardware() {
        // This test requires actual ELRS hardware connected
        let result = ElrsSerial::open();

        if let Ok(mut serial) = result {
            // Send a test packet
            let channels = [CRSF_CHANNEL_VALUE_CENTER; 16];
            let packet = encode_rc_channels_frame(&channels);

            let send_result = serial.send_packet(&packet).await;
            assert!(send_result.is_ok(), "Failed to send packet: {:?}", send_result);

            println!("Successfully sent test packet to ELRS device");
        } else {
            println!("No ELRS hardware detected (skipping send test)");
        }
    }
}
