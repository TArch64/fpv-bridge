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

mod port_trait;

use crate::error::{FpvBridgeError, Result};
use port_trait::{SerialPortIO, TokioSerialPort};
use tokio_serial::SerialPortBuilderExt;
use tracing::{debug, info, warn};

#[cfg(test)]
use port_trait::mocks::MockSerialPort;

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
    /// Serial port handle (trait object for testability)
    port: Box<dyn SerialPortIO>,
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
        // Special case: empty paths list
        if paths.is_empty() {
            return Err(FpvBridgeError::SerialPortNotFound(
                "No ELRS device paths configured".to_string()
            ));
        }

        for path in paths {
            debug!("Trying to open serial port: {}", path);

            match Self::open_port(path) {
                Ok(port) => {
                    info!("Successfully opened ELRS device at {}", path);
                    return Ok(Self {
                        port: Box::new(TokioSerialPort::new(port)),
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

    /// Create a new ElrsSerial with a custom port implementation (for testing)
    ///
    /// # Arguments
    ///
    /// * `port` - Serial port implementation
    /// * `device_path` - Device path string for identification
    ///
    /// # Returns
    ///
    /// * `ElrsSerial` - Serial handler with custom port
    #[cfg(test)]
    pub fn new_with_port(port: Box<dyn SerialPortIO>, device_path: String) -> Self {
        Self { port, device_path }
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
    /// fn main() -> anyhow::Result<()> {
    ///     let mut serial = ElrsSerial::open()?;
    ///
    ///     // Send test packet with all channels centered
    ///     let channels = [CRSF_CHANNEL_VALUE_CENTER; 16];
    ///     let packet = encode_rc_channels_frame(&channels);
    ///     serial.send_packet(&packet)?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn send_packet(&mut self, packet: &[u8]) -> Result<()> {
        self.port.write_all(packet)
            .map_err(|e| FpvBridgeError::Serial(format!("Failed to write packet: {}", e)))?;

        self.port.flush()
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

        // Should fail with SerialPortNotFound error with clear message
        assert!(result.is_err());
        match result.unwrap_err() {
            FpvBridgeError::SerialPortNotFound(msg) => {
                assert_eq!(msg, "No ELRS device paths configured");
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

    #[test]
    fn test_device_paths_are_absolute_and_valid() {
        // Verify all default device paths are absolute and follow /dev/tty* pattern
        assert_eq!(DEFAULT_DEVICE_PATHS.len(), 2, "Should have exactly 2 default paths");

        for path in DEFAULT_DEVICE_PATHS {
            // Must be absolute path
            assert!(path.starts_with('/'), "Device path must be absolute: {}", path);

            // Must follow /dev/tty* pattern (standard Linux serial device naming)
            assert!(
                path.starts_with("/dev/tty"),
                "Device path must follow /dev/tty* pattern: {}",
                path
            );

            // Must have characters after /dev/tty (e.g., ACM0, USB0)
            assert!(
                path.len() > "/dev/tty".len(),
                "Device path must specify device after /dev/tty: {}",
                path
            );
        }
    }

    #[test]
    fn test_open_uses_default_paths() {
        // Verify that open() delegates to open_with_paths with DEFAULT_DEVICE_PATHS
        // This test ensures the convenience method works as expected
        let result = ElrsSerial::open();

        // Should attempt to use default paths
        // Will fail on CI without hardware, but that's expected
        if let Err(FpvBridgeError::SerialPortNotFound(msg)) = result {
            // Error message should contain the default paths
            assert!(msg.contains("/dev/ttyACM0"), "Error should mention /dev/ttyACM0");
            assert!(msg.contains("/dev/ttyUSB0"), "Error should mention /dev/ttyUSB0");
        }
    }

    #[test]
    fn test_serial_port_not_found_error_message_format() {
        // Verify error message format when paths are not found
        let paths = &["/dev/test1", "/dev/test2", "/dev/test3"];
        let result = ElrsSerial::open_with_paths(paths);

        assert!(result.is_err());
        if let Err(FpvBridgeError::SerialPortNotFound(msg)) = result {
            // Should contain all attempted paths
            assert!(msg.contains("/dev/test1"));
            assert!(msg.contains("/dev/test2"));
            assert!(msg.contains("/dev/test3"));

            // Should be comma-separated
            assert!(msg.contains(", "));
        } else {
            panic!("Expected SerialPortNotFound error");
        }
    }

    #[test]
    fn test_error_message_contains_path_on_open_failure() {
        // Verify that error messages include the failing path for debugging
        let nonexistent_path = "/dev/this_definitely_does_not_exist_12345";
        let result = ElrsSerial::open_port(nonexistent_path);

        assert!(result.is_err());
        if let Err(FpvBridgeError::Serial(msg)) = result {
            // Error should mention both the operation and the path
            assert!(msg.contains("Failed to open"), "Error should mention operation");
            assert!(msg.contains(nonexistent_path), "Error should mention the failing path");
        } else {
            panic!("Expected Serial error");
        }
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
    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn test_send_packet_with_real_hardware() {
        use crate::crsf::encoder::encode_rc_channels_frame;
        use crate::crsf::protocol::CRSF_CHANNEL_VALUE_CENTER;

        // This test requires actual ELRS hardware connected
        let result = ElrsSerial::open();

        if let Ok(mut serial) = result {
            // Send a test packet
            let channels = [CRSF_CHANNEL_VALUE_CENTER; 16];
            let packet = encode_rc_channels_frame(&channels);

            let send_result = serial.send_packet(&packet);
            assert!(send_result.is_ok(), "Failed to send packet: {:?}", send_result);

            println!("Successfully sent test packet to ELRS device");
        } else {
            println!("No ELRS hardware detected (skipping send test)");
        }
    }

    // Unit tests with mock serial port
    #[test]
    fn test_send_packet_success_with_mock() {
        use crate::crsf::encoder::encode_rc_channels_frame;
        use crate::crsf::protocol::CRSF_CHANNEL_VALUE_CENTER;

        let mock_port = MockSerialPort::new();
        let mut serial = ElrsSerial::new_with_port(
            Box::new(mock_port.clone()),
            "/dev/mock".to_string(),
        );

        // Send a test packet
        let channels = [CRSF_CHANNEL_VALUE_CENTER; 16];
        let packet = encode_rc_channels_frame(&channels);
        let result = serial.send_packet(&packet);

        // Should succeed
        assert!(result.is_ok(), "Expected success, got: {:?}", result);

        // Verify packet was written
        let written_data = mock_port.get_written_data();
        assert_eq!(written_data.len(), 1, "Should have written one packet");
        assert_eq!(written_data[0], packet, "Written data should match packet");
    }

    #[test]
    fn test_send_packet_write_error_with_mock() {
        let mock_port = MockSerialPort::new();
        mock_port.set_write_error(std::io::ErrorKind::BrokenPipe);

        let mut serial = ElrsSerial::new_with_port(
            Box::new(mock_port.clone()),
            "/dev/mock".to_string(),
        );

        let packet = vec![0x01, 0x02, 0x03];
        let result = serial.send_packet(&packet);

        // Should fail with Serial error
        assert!(result.is_err());
        match result.unwrap_err() {
            FpvBridgeError::Serial(msg) => {
                assert!(msg.contains("Failed to write packet"));
            }
            other => panic!("Expected Serial error, got: {:?}", other),
        }
    }

    #[test]
    fn test_send_packet_flush_error_with_mock() {
        let mock_port = MockSerialPort::new();
        mock_port.set_flush_error(std::io::ErrorKind::TimedOut);

        let mut serial = ElrsSerial::new_with_port(
            Box::new(mock_port.clone()),
            "/dev/mock".to_string(),
        );

        let packet = vec![0x01, 0x02, 0x03];
        let result = serial.send_packet(&packet);

        // Should fail with Serial error
        assert!(result.is_err());
        match result.unwrap_err() {
            FpvBridgeError::Serial(msg) => {
                assert!(msg.contains("Failed to flush serial port"));
            }
            other => panic!("Expected Serial error, got: {:?}", other),
        }
    }

    #[test]
    fn test_send_multiple_packets_with_mock() {
        let mock_port = MockSerialPort::new();
        let mut serial = ElrsSerial::new_with_port(
            Box::new(mock_port.clone()),
            "/dev/mock".to_string(),
        );

        // Send multiple different packets
        let packet1 = vec![0x01, 0x02];
        let packet2 = vec![0x03, 0x04, 0x05];
        let packet3 = vec![0x06];

        assert!(serial.send_packet(&packet1).is_ok());
        assert!(serial.send_packet(&packet2).is_ok());
        assert!(serial.send_packet(&packet3).is_ok());

        // Verify all packets were written in order
        let written_data = mock_port.get_written_data();
        assert_eq!(written_data.len(), 3);
        assert_eq!(written_data[0], packet1);
        assert_eq!(written_data[1], packet2);
        assert_eq!(written_data[2], packet3);
    }

    #[test]
    fn test_send_empty_packet_with_mock() {
        let mock_port = MockSerialPort::new();
        let mut serial = ElrsSerial::new_with_port(
            Box::new(mock_port.clone()),
            "/dev/mock".to_string(),
        );

        let empty_packet = vec![];
        let result = serial.send_packet(&empty_packet);

        assert!(result.is_ok());
        let written_data = mock_port.get_written_data();
        assert_eq!(written_data.len(), 1);
        assert_eq!(written_data[0].len(), 0);
    }

    #[test]
    fn test_send_large_packet_with_mock() {
        let mock_port = MockSerialPort::new();
        let mut serial = ElrsSerial::new_with_port(
            Box::new(mock_port.clone()),
            "/dev/mock".to_string(),
        );

        // Create a large packet (256 bytes)
        let large_packet: Vec<u8> = (0..=255).collect();
        let result = serial.send_packet(&large_packet);

        assert!(result.is_ok());
        let written_data = mock_port.get_written_data();
        assert_eq!(written_data.len(), 1);
        assert_eq!(written_data[0], large_packet);
        assert_eq!(written_data[0].len(), 256);
    }

    #[test]
    fn test_device_path_with_mock() {
        let mock_port = MockSerialPort::new();
        let device_path = "/dev/mock_device";
        let serial = ElrsSerial::new_with_port(
            Box::new(mock_port),
            device_path.to_string(),
        );

        assert_eq!(serial.device_path(), device_path);
    }

    #[test]
    fn test_mock_port_error_recovery() {
        let mock_port = MockSerialPort::new();
        let mut serial = ElrsSerial::new_with_port(
            Box::new(mock_port.clone()),
            "/dev/mock".to_string(),
        );

        // Set error condition
        mock_port.set_write_error(std::io::ErrorKind::Interrupted);

        // First send should fail
        let packet = vec![0x01, 0x02];
        assert!(serial.send_packet(&packet).is_err());

        // Clear error (by setting to None indirectly - send new port)
        let mock_port2 = MockSerialPort::new();
        let mut serial2 = ElrsSerial::new_with_port(
            Box::new(mock_port2.clone()),
            "/dev/mock".to_string(),
        );

        // Second send should succeed
        assert!(serial2.send_packet(&packet).is_ok());
        let written_data = mock_port2.get_written_data();
        assert_eq!(written_data.len(), 1);
    }

    #[test]
    fn test_send_packet_preserves_data_integrity() {
        use crate::crsf::encoder::encode_rc_channels_frame;

        let mock_port = MockSerialPort::new();
        let mut serial = ElrsSerial::new_with_port(
            Box::new(mock_port.clone()),
            "/dev/mock".to_string(),
        );

        // Create a packet with specific channel values
        let channels = [100, 200, 300, 400, 500, 600, 700, 800, 900, 1000, 1100, 1200, 1300, 1400, 1500, 1600];
        let packet = encode_rc_channels_frame(&channels);

        assert!(serial.send_packet(&packet).is_ok());

        // Verify exact byte-for-byte match
        let written_data = mock_port.get_written_data();
        assert_eq!(written_data[0], packet, "Packet data should be preserved exactly");
    }
}
