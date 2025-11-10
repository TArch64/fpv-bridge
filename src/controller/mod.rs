//! # Controller Module
//!
//! PS5 DualSense controller input handling.
//!
//! This module handles:
//! - PS5 controller detection and connection via evdev
//! - Reading analog stick and button inputs
//! - Applying deadzones and exponential curves
//! - Mapping inputs to RC channels
//! - Calibration and safety checks

// Module exports will be added as we implement submodules
// pub mod ps5;
// pub mod mapper;
// pub mod calibration;

// Placeholder types - will be implemented in future PRs
#[allow(dead_code)]
/// Controller state
#[derive(Debug, Clone, Default)]
pub struct ControllerState {
    // Sticks (normalized -1.0 to 1.0)
    pub left_stick_x: f32,
    pub left_stick_y: f32,
    pub right_stick_x: f32,
    pub right_stick_y: f32,

    // Triggers (0.0 to 1.0)
    pub left_trigger: f32,
    pub right_trigger: f32,

    // Buttons
    pub button_cross: bool,
    pub button_circle: bool,
    pub button_square: bool,
    pub button_triangle: bool,
    pub button_l1: bool,
    pub button_r1: bool,
    pub button_l2: bool,
    pub button_r2: bool,
    pub button_share: bool,
    pub button_options: bool,
    pub button_ps: bool,
    pub button_touchpad: bool,

    // D-Pad
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
}
