//! # PS5 DualSense Controller Module
//!
//! This module handles PS5 DualSense controller detection, connection, and input reading
//! using the Linux evdev interface.
//!
//! ## Controller Detection
//!
//! The DualSense controller is identified by:
//! - Vendor ID: 0x054c (Sony)
//! - Product ID: 0x0ce6 (DualSense, both wired and Bluetooth)
//!
//! ## Input Axes
//!
//! - Left stick: ABS_X (0-255), ABS_Y (0-255)
//! - Right stick: ABS_Z (0-255), ABS_RZ (0-255)
//! - Triggers: ABS_RX (L2), ABS_RY (R2) (0-255)

use evdev::Device;
use std::path::Path;
use tracing::{debug, info};

use crate::error::{FpvBridgeError, Result};

/// PS5 DualSense vendor ID (Sony)
const DUALSENSE_VENDOR_ID: u16 = 0x054c;

/// PS5 DualSense product ID (wired and Bluetooth)
const DUALSENSE_PRODUCT_ID: u16 = 0x0ce6;

/// PS5 DualSense controller handle
///
/// Represents an active connection to a PS5 DualSense controller via evdev.
/// Provides methods for reading controller input events.
pub struct DualSenseController {
    device: Device,
    device_path: String,
}

impl DualSenseController {
    /// Detect and open the first available PS5 DualSense controller
    ///
    /// Scans all `/dev/input/event*` devices to find a connected DualSense controller
    /// by matching vendor and product IDs.
    ///
    /// # Returns
    ///
    /// Returns `Ok(DualSenseController)` if a controller is found and opened successfully.
    ///
    /// # Errors
    ///
    /// - `ControllerNotFound`: No DualSense controller found on the system
    /// - `Io`: Permission denied or other I/O errors when opening device
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fpv_bridge::controller::ps5::DualSenseController;
    ///
    /// let controller = DualSenseController::open()?;
    /// println!("Connected to controller at: {}", controller.device_path());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn open() -> Result<Self> {
        // Scan /dev/input for event devices
        let input_dir = Path::new("/dev/input");

        if !input_dir.exists() {
            return Err(FpvBridgeError::Controller(
                "/dev/input directory not found".to_string(),
            ));
        }

        let mut entries: Vec<_> = std::fs::read_dir(input_dir)
            .map_err(|e| FpvBridgeError::Controller(format!("Failed to read /dev/input: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| FpvBridgeError::Controller(format!("Failed to read directory entry: {}", e)))?;

        // Sort entries for deterministic device selection when multiple controllers are connected
        entries.sort_by_key(|entry| entry.path());

        for entry in entries {
            let path = entry.path();

            // Only check event* devices
            if let Some(filename) = path.file_name() {
                if !filename.to_string_lossy().starts_with("event") {
                    continue;
                }
            } else {
                continue;
            }

            // Try to open the device
            match Device::open(&path) {
                Ok(device) => {
                    // Check if this is a DualSense controller
                    let id = device.input_id();
                    debug!(
                        "Found input device: {} (vendor: 0x{:04x}, product: 0x{:04x})",
                        path.display(),
                        id.vendor(),
                        id.product()
                    );

                    if id.vendor() == DUALSENSE_VENDOR_ID
                        && id.product() == DUALSENSE_PRODUCT_ID
                    {
                        let device_path = path.to_string_lossy().to_string();
                        info!(
                            "Found PS5 DualSense controller at: {}",
                            device_path
                        );

                        return Ok(DualSenseController {
                            device,
                            device_path,
                        });
                    }
                }
                Err(e) => {
                    // Permission denied or other errors - skip device
                    debug!("Could not open {}: {}", path.display(), e);
                }
            }
        }

        Err(FpvBridgeError::ControllerNotFound)
    }

    /// Get the device path of this controller
    ///
    /// Returns the `/dev/input/eventX` path that was used to open this controller.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use fpv_bridge::controller::ps5::DualSenseController;
    /// # let controller = DualSenseController::open()?;
    /// println!("Controller path: {}", controller.device_path());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn device_path(&self) -> &str {
        &self.device_path
    }

    /// Fetch events from the controller
    ///
    /// Fetches available input events from the controller.
    /// Returns an iterator over events. This call may block if no events are available.
    ///
    /// # Returns
    ///
    /// Returns an iterator over `InputEvent` objects.
    ///
    /// # Errors
    ///
    /// Returns `Controller` error if fetching events fails (e.g., controller disconnected).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use fpv_bridge::controller::ps5::DualSenseController;
    /// # let mut controller = DualSenseController::open()?;
    /// loop {
    ///     for event in controller.fetch_events()? {
    ///         // Process event...
    ///         println!("Event: {:?}", event);
    ///     }
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn fetch_events(&mut self) -> Result<impl Iterator<Item = evdev::InputEvent> + '_> {
        self.device
            .fetch_events()
            .map_err(|e| FpvBridgeError::Controller(format!("Failed to fetch events: {}", e)))
    }

    /// Get controller name from evdev
    ///
    /// Returns the human-readable name of the controller device, typically
    /// "Wireless Controller" or "DualSense Wireless Controller".
    pub fn name(&self) -> Option<&str> {
        self.device.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dualsense_vendor_id() {
        // Verify Sony vendor ID
        assert_eq!(DUALSENSE_VENDOR_ID, 0x054c, "Sony vendor ID should be 0x054c");
    }

    #[test]
    fn test_dualsense_product_id() {
        // Verify DualSense product ID
        assert_eq!(
            DUALSENSE_PRODUCT_ID, 0x0ce6,
            "DualSense product ID should be 0x0ce6"
        );
    }

    // Integration test - only runs with real hardware
    #[test]
    #[ignore]
    fn test_open_with_real_hardware() {
        // This test requires a connected PS5 controller
        let result = DualSenseController::open();
        assert!(result.is_ok(), "Should detect connected PS5 controller");

        let controller = result.unwrap();
        assert!(controller.device_path().starts_with("/dev/input/event"));
        assert!(controller.name().is_some());
    }

    // Integration test - only runs with real hardware
    #[test]
    #[ignore]
    fn test_fetch_events_with_real_hardware() {
        // This test requires a connected PS5 controller
        let mut controller = DualSenseController::open().expect("Controller not found");

        println!("Move controller sticks or press buttons within 5 seconds...");

        // Try to read events over 5 seconds (100 iterations * 50ms)
        for _ in 0..100 {
            match controller.fetch_events() {
                Ok(events) => {
                    for event in events {
                        println!("Received event: {:?}", event);
                        return; // Test passed if we got at least one event
                    }
                }
                Err(_) => continue,
            }

            // Sleep 50ms between iterations to allow user interaction time
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        panic!("No events received from controller");
    }
}
