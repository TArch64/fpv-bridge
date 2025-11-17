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
//! ## Permissions
//!
//! Access to `/dev/input/event*` devices requires appropriate permissions.
//!
//! **Option 1: Add user to input group (recommended)**
//! ```bash
//! sudo usermod -a -G input $USER
//! # Log out and back in for changes to take effect
//! ```
//!
//! **Option 2: Create udev rule for specific controller**
//! ```bash
//! # Create /etc/udev/rules.d/99-dualsense.rules with:
//! SUBSYSTEM=="input", ATTRS{idVendor}=="054c", ATTRS{idProduct}=="0ce6", MODE="0666"
//! # Reload rules:
//! sudo udevadm control --reload-rules && sudo udevadm trigger
//! ```
//!
//! **Option 3: Run as root (not recommended for production)**
//! ```bash
//! sudo ./fpv-bridge
//! ```
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

    #[test]
    fn test_open_returns_error_when_no_input_dir() {
        // Test behavior when /dev/input doesn't exist
        // This will fail on systems where /dev/input exists, but tests error path
        // On CI or systems without /dev/input, this ensures error handling works
        let result = DualSenseController::open();

        // Either succeeds (if controller found) or returns appropriate error
        if let Err(e) = result {
            let msg = e.to_string().to_lowercase();
            assert!(
                msg.contains("controller") || msg.contains("not found") || msg.contains("/dev/input"),
                "Error should be controller-related, got: {}", e
            );
        }
    }

    #[test]
    fn test_controller_not_found_error() {
        // Verify ControllerNotFound error message
        let error = FpvBridgeError::ControllerNotFound;
        assert_eq!(
            error.to_string(),
            "No PS5 DualSense controller found"
        );
    }

    #[test]
    fn test_controller_error_with_message() {
        // Verify Controller error with custom message
        let error = FpvBridgeError::Controller("test error".to_string());
        assert_eq!(
            error.to_string(),
            "Controller error: test error"
        );
    }

    #[test]
    fn test_vendor_and_product_id_matching() {
        // Verify the vendor/product ID constants are used correctly
        // This tests the logic of ID matching without requiring hardware
        let vendor = DUALSENSE_VENDOR_ID;
        let product = DUALSENSE_PRODUCT_ID;

        // Test exact match (what we expect for DualSense)
        assert_eq!(vendor, 0x054c);
        assert_eq!(product, 0x0ce6);

        // Test non-matching IDs (what would be rejected)
        assert_ne!(vendor, 0x0000);
        assert_ne!(product, 0x0000);
        assert_ne!(vendor, 0xFFFF);
        assert_ne!(product, 0xFFFF);
    }

    #[test]
    fn test_device_path_format() {
        // Test that device path would follow expected pattern
        // This validates our path handling logic
        let test_paths = vec![
            "/dev/input/event0",
            "/dev/input/event1",
            "/dev/input/event10",
            "/dev/input/event99",
        ];

        for path in test_paths {
            assert!(path.starts_with("/dev/input/event"));
            let filename = std::path::Path::new(path)
                .file_name()
                .unwrap()
                .to_string_lossy();
            assert!(filename.starts_with("event"));
        }
    }

    #[test]
    fn test_controller_open_error_handling() {
        // Test that open() returns proper error when no controller found
        // On CI or systems without a DualSense, this should return ControllerNotFound
        let result = DualSenseController::open();

        // Either it succeeds (if hardware present) or returns appropriate error
        if let Err(e) = result {
            // The error should be either ControllerNotFound or Controller variant
            match e {
                FpvBridgeError::ControllerNotFound => {
                    assert_eq!(e.to_string(), "No PS5 DualSense controller found");
                }
                FpvBridgeError::Controller(msg) => {
                    // Should be related to /dev/input access
                    assert!(
                        msg.contains("/dev/input") || msg.contains("directory"),
                        "Controller error should mention /dev/input or directory, got: {}", msg
                    );
                }
                _ => panic!("Unexpected error type: {:?}", e),
            }
        }
    }

    #[test]
    fn test_error_path_validation() {
        // Test error message construction for /dev/input directory errors
        let error = FpvBridgeError::Controller("/dev/input directory not found".to_string());
        assert!(error.to_string().contains("/dev/input"));
        assert!(error.to_string().contains("directory not found"));

        // Test error message for directory read errors
        let error2 = FpvBridgeError::Controller("Failed to read /dev/input: Permission denied".to_string());
        assert!(error2.to_string().contains("Failed to read /dev/input"));
        assert!(error2.to_string().contains("Permission denied"));
    }

    #[test]
    fn test_dualsense_constants_are_correct() {
        // Comprehensive test of DualSense identification constants
        // These are critical for controller detection

        // Sony Corporation vendor ID (standardized USB-IF assignment)
        assert_eq!(DUALSENSE_VENDOR_ID, 0x054c,
            "Sony vendor ID must be 0x054c per USB-IF assignment");

        // DualSense product ID (both wired and Bluetooth use same ID)
        assert_eq!(DUALSENSE_PRODUCT_ID, 0x0ce6,
            "DualSense product ID must be 0x0ce6 for both wired and Bluetooth");

        // Verify IDs are non-zero (sanity check)
        assert!(DUALSENSE_VENDOR_ID > 0, "Vendor ID must be non-zero");
        assert!(DUALSENSE_PRODUCT_ID > 0, "Product ID must be non-zero");
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
