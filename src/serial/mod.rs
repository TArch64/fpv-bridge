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
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let serial = ElrsSerial::open().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn open() -> Result<Self> {
        Self::open_with_paths(DEFAULT_DEVICE_PATHS).await
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
    pub async fn open_with_paths(paths: &[&str]) -> Result<Self> {
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
    ///     let mut serial = ElrsSerial::open().await?;
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

    /// Get the device path
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
    }
}
