//! # PS5 DualSense Controller Handler
//!
//! Handles PS5 DualSense controller input via evdev.

use evdev::{Device, InputEventKind, AbsoluteAxisType, Key};
use crate::error::{FpvBridgeError, Result};

/// PS5 DualSense controller state
#[derive(Debug, Clone, Default)]
pub struct ControllerState {
    // Left stick (normalized -1.0 to 1.0)
    pub left_stick_x: f32,
    pub left_stick_y: f32,

    // Right stick (normalized -1.0 to 1.0)
    pub right_stick_x: f32,
    pub right_stick_y: f32,

    // Triggers (normalized 0.0 to 1.0)
    pub left_trigger: f32,
    pub right_trigger: f32,

    // D-Pad
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,

    // Face buttons
    pub button_cross: bool,
    pub button_circle: bool,
    pub button_square: bool,
    pub button_triangle: bool,

    // Shoulder buttons
    pub button_l1: bool,
    pub button_r1: bool,
    pub button_l2: bool,
    pub button_r2: bool,

    // System buttons
    pub button_share: bool,
    pub button_options: bool,
    pub button_ps: bool,
    pub button_touchpad: bool,
}

/// PS5 controller handler using evdev
pub struct Ps5Controller {
    device: Device,
    state: ControllerState,
}

impl Ps5Controller {
    /// Create a new PS5 controller handler
    ///
    /// # Arguments
    ///
    /// * `device_path` - Optional path to device (e.g. "/dev/input/event0"). If empty, auto-detect.
    ///
    /// # Returns
    ///
    /// * `Result<Ps5Controller>` - Controller if found and opened successfully
    ///
    /// # Errors
    ///
    /// Returns error if controller cannot be found or opened
    pub fn new(device_path: &str) -> Result<Self> {
        let device = if device_path.is_empty() {
            Self::find_ps5_controller()?
        } else {
            Device::open(device_path)
                .map_err(|e| FpvBridgeError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Failed to open device at {}: {}", device_path, e)
                )))?
        };

        Ok(Self {
            device,
            state: ControllerState::default(),
        })
    }

    /// Find PS5 DualSense controller automatically
    fn find_ps5_controller() -> Result<Device> {
        let devices = evdev::enumerate()
            .filter(|(_, dev)| Self::is_ps5_controller(dev))
            .collect::<Vec<_>>();

        if devices.is_empty() {
            return Err(FpvBridgeError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No PS5 DualSense controller found"
            )));
        }

        if devices.len() > 1 {
            tracing::warn!("Multiple PS5 controllers found, using first one");
        }

        let (_, device) = devices.into_iter().next().unwrap();
        Ok(device)
    }

    /// Check if device is a PS5 DualSense controller
    fn is_ps5_controller(device: &Device) -> bool {
        let name = device.name().unwrap_or("");

        // PS5 DualSense controller identifiers
        name.contains("DualSense") ||
        name.contains("Wireless Controller") &&
            device.input_id().vendor() == 0x054c &&  // Sony
            device.input_id().product() == 0x0ce6     // DualSense
    }

    /// Read the current controller state
    ///
    /// # Returns
    ///
    /// * `Result<ControllerState>` - Current state
    ///
    /// # Errors
    ///
    /// Returns error if device read fails
    pub fn read_state(&mut self) -> Result<ControllerState> {
        // Fetch all pending events and collect them to avoid borrow checker issues
        let events: Vec<_> = self.device.fetch_events()
            .map_err(|e| FpvBridgeError::Io(e))?
            .collect();

        for ev in events {
            self.process_event(ev);
        }

        Ok(self.state.clone())
    }

    /// Process a single input event
    fn process_event(&mut self, event: evdev::InputEvent) {
        match event.kind() {
            InputEventKind::AbsAxis(axis) => {
                self.process_axis_event(axis, event.value());
            }
            InputEventKind::Key(key) => {
                self.process_button_event(key, event.value() != 0);
            }
            _ => {}
        }
    }

    /// Process axis (stick/trigger) event
    fn process_axis_event(&mut self, axis: AbsoluteAxisType, value: i32) {
        // PS5 controller axis ranges: 0-255 for sticks, 0-255 for triggers
        match axis {
            // Left stick
            AbsoluteAxisType::ABS_X => {
                self.state.left_stick_x = Self::normalize_stick(value);
            }
            AbsoluteAxisType::ABS_Y => {
                // Invert Y axis (down is positive in hardware, we want up positive)
                self.state.left_stick_y = -Self::normalize_stick(value);
            }

            // Right stick
            AbsoluteAxisType::ABS_RX => {
                self.state.right_stick_x = Self::normalize_stick(value);
            }
            AbsoluteAxisType::ABS_RY => {
                self.state.right_stick_y = -Self::normalize_stick(value);
            }

            // Triggers
            AbsoluteAxisType::ABS_Z => {
                self.state.left_trigger = Self::normalize_trigger(value);
            }
            AbsoluteAxisType::ABS_RZ => {
                self.state.right_trigger = Self::normalize_trigger(value);
            }

            // D-Pad (reported as HAT on some systems)
            AbsoluteAxisType::ABS_HAT0X => {
                self.state.dpad_left = value < 0;
                self.state.dpad_right = value > 0;
            }
            AbsoluteAxisType::ABS_HAT0Y => {
                self.state.dpad_up = value < 0;
                self.state.dpad_down = value > 0;
            }

            _ => {}
        }
    }

    /// Process button event
    fn process_button_event(&mut self, key: evdev::Key, pressed: bool) {
        match key {
            // Face buttons
            Key::BTN_SOUTH => self.state.button_cross = pressed,
            Key::BTN_EAST => self.state.button_circle = pressed,
            Key::BTN_WEST => self.state.button_square = pressed,
            Key::BTN_NORTH => self.state.button_triangle = pressed,

            // Shoulder buttons
            Key::BTN_TL => self.state.button_l1 = pressed,
            Key::BTN_TR => self.state.button_r1 = pressed,
            Key::BTN_TL2 => self.state.button_l2 = pressed,
            Key::BTN_TR2 => self.state.button_r2 = pressed,

            // System buttons
            Key::BTN_SELECT => self.state.button_share = pressed,
            Key::BTN_START => self.state.button_options = pressed,
            Key::BTN_MODE => self.state.button_ps = pressed,
            Key::BTN_THUMBL => self.state.button_touchpad = pressed,

            _ => {}
        }
    }

    /// Normalize stick value from hardware range (0-255) to -1.0 to 1.0
    fn normalize_stick(value: i32) -> f32 {
        const CENTER: f32 = 127.5;
        const RANGE: f32 = 127.5;

        ((value as f32) - CENTER) / RANGE
    }

    /// Normalize trigger value from hardware range (0-255) to 0.0 to 1.0
    fn normalize_trigger(value: i32) -> f32 {
        (value as f32) / 255.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_stick_center() {
        assert!((Ps5Controller::normalize_stick(128) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_normalize_stick_min() {
        assert!((Ps5Controller::normalize_stick(0) + 1.0).abs() < 0.01);
    }

    #[test]
    fn test_normalize_stick_max() {
        assert!((Ps5Controller::normalize_stick(255) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_normalize_trigger_min() {
        assert_eq!(Ps5Controller::normalize_trigger(0), 0.0);
    }

    #[test]
    fn test_normalize_trigger_max() {
        assert_eq!(Ps5Controller::normalize_trigger(255), 1.0);
    }

    #[test]
    fn test_normalize_trigger_mid() {
        assert!((Ps5Controller::normalize_trigger(128) - 0.502).abs() < 0.01);
    }
}
