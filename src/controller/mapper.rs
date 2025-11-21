//! # Controller Input Mapper Module
//!
//! This module handles parsing raw evdev events from the PS5 DualSense controller
//! and converting them into a structured [`ControllerState`].
//!
//! ## Event Types
//!
//! The DualSense controller emits the following event types:
//!
//! - **EV_ABS (Absolute Axis)**: Analog inputs like sticks and triggers
//! - **EV_KEY (Key/Button)**: Digital button presses
//!
//! ## Axis Codes (EV_ABS)
//!
//! | Axis | evdev Code | Range | Description |
//! |------|------------|-------|-------------|
//! | Left Stick X | ABS_X | 0-255 | Yaw control |
//! | Left Stick Y | ABS_Y | 0-255 | Throttle control |
//! | Right Stick X | ABS_Z | 0-255 | Roll control |
//! | Right Stick Y | ABS_RZ | 0-255 | Pitch control |
//! | L2 Trigger | ABS_RX | 0-255 | Beeper (analog) |
//! | R2 Trigger | ABS_RY | 0-255 | Turtle mode (analog) |
//! | D-Pad X | ABS_HAT0X | -1/0/1 | Left/Center/Right |
//! | D-Pad Y | ABS_HAT0Y | -1/0/1 | Up/Center/Down |
//!
//! ## Button Codes (EV_KEY)
//!
//! | Button | evdev Code | Description |
//! |--------|------------|-------------|
//! | Cross (×) | BTN_SOUTH | Reserved |
//! | Circle (○) | BTN_EAST | Reserved |
//! | Square (□) | BTN_WEST | Reserved |
//! | Triangle (△) | BTN_NORTH | Flip mode |
//! | L1 | BTN_TL | ARM switch |
//! | R1 | BTN_TR | Flight mode |
//! | L2 (click) | BTN_TL2 | L2 digital |
//! | R2 (click) | BTN_TR2 | R2 digital |
//! | Share | BTN_SELECT | Toggle logging |
//! | Options | BTN_START | Cycle modes |
//! | PS | BTN_MODE | Emergency disarm |
//! | L3 | BTN_THUMBL | Left stick click |
//! | R3 | BTN_THUMBR | Right stick click |
//! | Touchpad | BTN_TOUCH | Calibration |
//!
//! ## Usage
//!
//! ```no_run
//! use fpv_bridge::controller::mapper::{EventMapper, ControllerState};
//! use fpv_bridge::controller::ps5::DualSenseController;
//!
//! let mut controller = DualSenseController::open()?;
//! let mut mapper = EventMapper::new();
//!
//! loop {
//!     for event in controller.fetch_events()? {
//!         mapper.process_event(&event);
//!     }
//!     let state = mapper.state();
//!     // Use state for RC channel mapping...
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use evdev::{AbsoluteAxisType, InputEvent, Key};

/// Raw axis value range from DualSense controller.
pub const AXIS_MIN: i32 = 0;
/// Raw axis value range from DualSense controller.
pub const AXIS_MAX: i32 = 255;
/// Raw axis center value.
pub const AXIS_CENTER: i32 = 128;

/// D-Pad axis values.
pub const DPAD_RELEASED: i32 = 0;
/// D-Pad pressed negative direction (left or up).
pub const DPAD_NEGATIVE: i32 = -1;
/// D-Pad pressed positive direction (right or down).
pub const DPAD_POSITIVE: i32 = 1;

/// Represents the complete state of the PS5 DualSense controller.
///
/// All analog values are stored as raw evdev values (0-255 for sticks/triggers,
/// -1/0/1 for d-pad). Calibration and scaling to RC channel values is handled
/// separately by the calibration module.
///
/// # Examples
///
/// ```
/// use fpv_bridge::controller::mapper::ControllerState;
///
/// let state = ControllerState::default();
/// assert_eq!(state.left_stick_x, 128);  // Centered
/// assert!(!state.btn_l1);               // Not pressed
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControllerState {
    // Analog sticks (0-255, 128 = center)
    /// Left stick X axis (Yaw). 0 = full left, 255 = full right.
    pub left_stick_x: i32,
    /// Left stick Y axis (Throttle). 0 = full up, 255 = full down.
    pub left_stick_y: i32,
    /// Right stick X axis (Roll). 0 = full left, 255 = full right.
    pub right_stick_x: i32,
    /// Right stick Y axis (Pitch). 0 = full up, 255 = full down.
    pub right_stick_y: i32,

    // Triggers (0-255)
    /// L2 trigger analog value. 0 = released, 255 = fully pressed.
    pub trigger_l2: i32,
    /// R2 trigger analog value. 0 = released, 255 = fully pressed.
    pub trigger_r2: i32,

    // D-Pad (-1, 0, 1)
    /// D-Pad X axis. -1 = left, 0 = center, 1 = right.
    pub dpad_x: i32,
    /// D-Pad Y axis. -1 = up, 0 = center, 1 = down.
    pub dpad_y: i32,

    // Face buttons
    /// Cross button (×) - BTN_SOUTH.
    pub btn_cross: bool,
    /// Circle button (○) - BTN_EAST.
    pub btn_circle: bool,
    /// Square button (□) - BTN_WEST.
    pub btn_square: bool,
    /// Triangle button (△) - BTN_NORTH.
    pub btn_triangle: bool,

    // Shoulder buttons
    /// L1 button (ARM switch).
    pub btn_l1: bool,
    /// R1 button (Flight mode).
    pub btn_r1: bool,
    /// L2 button digital click.
    pub btn_l2: bool,
    /// R2 button digital click.
    pub btn_r2: bool,

    // System buttons
    /// Share button (toggle logging).
    pub btn_share: bool,
    /// Options button (cycle modes).
    pub btn_options: bool,
    /// PS button (emergency disarm).
    pub btn_ps: bool,

    // Stick clicks
    /// L3 button (left stick click).
    pub btn_l3: bool,
    /// R3 button (right stick click).
    pub btn_r3: bool,

    // Touchpad
    /// Touchpad click (calibration).
    pub btn_touchpad: bool,
}

impl Default for ControllerState {
    /// Creates a new controller state with all sticks centered and buttons released.
    fn default() -> Self {
        Self {
            // Sticks centered at 128
            left_stick_x: AXIS_CENTER,
            left_stick_y: AXIS_CENTER,
            right_stick_x: AXIS_CENTER,
            right_stick_y: AXIS_CENTER,

            // Triggers released
            trigger_l2: AXIS_MIN,
            trigger_r2: AXIS_MIN,

            // D-Pad centered
            dpad_x: DPAD_RELEASED,
            dpad_y: DPAD_RELEASED,

            // All buttons released
            btn_cross: false,
            btn_circle: false,
            btn_square: false,
            btn_triangle: false,
            btn_l1: false,
            btn_r1: false,
            btn_l2: false,
            btn_r2: false,
            btn_share: false,
            btn_options: false,
            btn_ps: false,
            btn_l3: false,
            btn_r3: false,
            btn_touchpad: false,
        }
    }
}

impl ControllerState {
    /// Creates a new controller state with default (centered/released) values.
    ///
    /// # Examples
    ///
    /// ```
    /// use fpv_bridge::controller::mapper::ControllerState;
    ///
    /// let state = ControllerState::new();
    /// assert_eq!(state.left_stick_x, 128);
    /// assert_eq!(state.trigger_l2, 0);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks if any stick has moved from center position.
    ///
    /// Useful for detecting controller activity for auto-disarm timeout.
    ///
    /// # Arguments
    ///
    /// * `threshold` - Minimum deviation from center to consider "moved" (0-127)
    ///
    /// # Examples
    ///
    /// ```
    /// use fpv_bridge::controller::mapper::ControllerState;
    ///
    /// let mut state = ControllerState::new();
    /// assert!(!state.any_stick_moved(10));
    ///
    /// state.left_stick_x = 150; // Moved right
    /// assert!(state.any_stick_moved(10));
    /// ```
    #[must_use]
    pub fn any_stick_moved(&self, threshold: i32) -> bool {
        (self.left_stick_x - AXIS_CENTER).abs() > threshold
            || (self.left_stick_y - AXIS_CENTER).abs() > threshold
            || (self.right_stick_x - AXIS_CENTER).abs() > threshold
            || (self.right_stick_y - AXIS_CENTER).abs() > threshold
    }

    /// Checks if any button is currently pressed.
    ///
    /// # Examples
    ///
    /// ```
    /// use fpv_bridge::controller::mapper::ControllerState;
    ///
    /// let mut state = ControllerState::new();
    /// assert!(!state.any_button_pressed());
    ///
    /// state.btn_l1 = true;
    /// assert!(state.any_button_pressed());
    /// ```
    #[must_use]
    pub fn any_button_pressed(&self) -> bool {
        self.btn_cross
            || self.btn_circle
            || self.btn_square
            || self.btn_triangle
            || self.btn_l1
            || self.btn_r1
            || self.btn_l2
            || self.btn_r2
            || self.btn_share
            || self.btn_options
            || self.btn_ps
            || self.btn_l3
            || self.btn_r3
            || self.btn_touchpad
    }

    /// Checks if any trigger is pressed beyond a threshold.
    ///
    /// # Arguments
    ///
    /// * `threshold` - Minimum value to consider "pressed" (0-255)
    ///
    /// # Examples
    ///
    /// ```
    /// use fpv_bridge::controller::mapper::ControllerState;
    ///
    /// let mut state = ControllerState::new();
    /// assert!(!state.any_trigger_pressed(10));
    ///
    /// state.trigger_l2 = 200;
    /// assert!(state.any_trigger_pressed(10));
    /// ```
    #[must_use]
    pub fn any_trigger_pressed(&self, threshold: i32) -> bool {
        self.trigger_l2 > threshold || self.trigger_r2 > threshold
    }
}

/// Parses raw evdev events and maintains controller state.
///
/// The `EventMapper` accumulates events from the controller and provides
/// a snapshot of the current state via [`EventMapper::state()`].
///
/// # Thread Safety
///
/// `EventMapper` is not thread-safe. Use from a single task/thread only.
///
/// # Examples
///
/// ```
/// use fpv_bridge::controller::mapper::EventMapper;
///
/// let mut mapper = EventMapper::new();
/// // Process events from controller...
/// let state = mapper.state();
/// println!("Left stick X: {}", state.left_stick_x);
/// ```
#[derive(Debug)]
pub struct EventMapper {
    state: ControllerState,
}

impl Default for EventMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl EventMapper {
    /// Creates a new event mapper with default controller state.
    ///
    /// # Examples
    ///
    /// ```
    /// use fpv_bridge::controller::mapper::EventMapper;
    ///
    /// let mapper = EventMapper::new();
    /// let state = mapper.state();
    /// assert_eq!(state.left_stick_x, 128);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: ControllerState::default(),
        }
    }

    /// Returns a reference to the current controller state.
    ///
    /// The state reflects all events processed so far.
    ///
    /// # Examples
    ///
    /// ```
    /// use fpv_bridge::controller::mapper::EventMapper;
    ///
    /// let mapper = EventMapper::new();
    /// let state = mapper.state();
    /// assert!(!state.btn_l1);
    /// ```
    #[must_use]
    pub fn state(&self) -> &ControllerState {
        &self.state
    }

    /// Returns a clone of the current controller state.
    ///
    /// Use this when you need an owned copy of the state.
    #[must_use]
    pub fn state_snapshot(&self) -> ControllerState {
        self.state.clone()
    }

    /// Processes a single evdev input event and updates internal state.
    ///
    /// Handles both absolute axis events (sticks, triggers, d-pad) and
    /// key events (buttons).
    ///
    /// # Arguments
    ///
    /// * `event` - The evdev input event to process
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fpv_bridge::controller::mapper::EventMapper;
    /// use fpv_bridge::controller::ps5::DualSenseController;
    ///
    /// let mut controller = DualSenseController::open()?;
    /// let mut mapper = EventMapper::new();
    ///
    /// for event in controller.fetch_events()? {
    ///     mapper.process_event(&event);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn process_event(&mut self, event: &InputEvent) {
        match event.kind() {
            evdev::InputEventKind::AbsAxis(axis) => {
                self.process_axis_event(axis, event.value());
            }
            evdev::InputEventKind::Key(key) => {
                self.process_key_event(key, event.value() != 0);
            }
            _ => {
                // Ignore sync events and other event types
            }
        }
    }

    /// Processes an absolute axis event.
    fn process_axis_event(&mut self, axis: AbsoluteAxisType, value: i32) {
        match axis {
            // Left stick
            AbsoluteAxisType::ABS_X => self.state.left_stick_x = value,
            AbsoluteAxisType::ABS_Y => self.state.left_stick_y = value,

            // Right stick (DualSense uses ABS_Z and ABS_RZ)
            AbsoluteAxisType::ABS_Z => self.state.right_stick_x = value,
            AbsoluteAxisType::ABS_RZ => self.state.right_stick_y = value,

            // Triggers (DualSense uses ABS_RX and ABS_RY for analog triggers)
            AbsoluteAxisType::ABS_RX => self.state.trigger_l2 = value,
            AbsoluteAxisType::ABS_RY => self.state.trigger_r2 = value,

            // D-Pad
            AbsoluteAxisType::ABS_HAT0X => self.state.dpad_x = value,
            AbsoluteAxisType::ABS_HAT0Y => self.state.dpad_y = value,

            _ => {
                // Ignore other axes (gyro, accelerometer, etc.)
            }
        }
    }

    /// Processes a key/button event.
    fn process_key_event(&mut self, key: Key, pressed: bool) {
        match key {
            // Face buttons
            Key::BTN_SOUTH => self.state.btn_cross = pressed,
            Key::BTN_EAST => self.state.btn_circle = pressed,
            Key::BTN_WEST => self.state.btn_square = pressed,
            Key::BTN_NORTH => self.state.btn_triangle = pressed,

            // Shoulder buttons
            Key::BTN_TL => self.state.btn_l1 = pressed,
            Key::BTN_TR => self.state.btn_r1 = pressed,
            Key::BTN_TL2 => self.state.btn_l2 = pressed,
            Key::BTN_TR2 => self.state.btn_r2 = pressed,

            // System buttons
            Key::BTN_SELECT => self.state.btn_share = pressed,
            Key::BTN_START => self.state.btn_options = pressed,
            Key::BTN_MODE => self.state.btn_ps = pressed,

            // Stick clicks
            Key::BTN_THUMBL => self.state.btn_l3 = pressed,
            Key::BTN_THUMBR => self.state.btn_r3 = pressed,

            // Touchpad (BTN_TOUCH for finger contact, we use click)
            Key::BTN_TOUCH => self.state.btn_touchpad = pressed,

            _ => {
                // Ignore unknown buttons
            }
        }
    }

    /// Resets all state to default (centered sticks, released buttons).
    ///
    /// Useful for calibration or when reconnecting a controller.
    ///
    /// # Examples
    ///
    /// ```
    /// use fpv_bridge::controller::mapper::EventMapper;
    ///
    /// let mut mapper = EventMapper::new();
    /// // ... process events ...
    /// mapper.reset();
    /// assert_eq!(mapper.state().left_stick_x, 128);
    /// ```
    pub fn reset(&mut self) {
        self.state = ControllerState::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use evdev::EventType;

    /// Helper to create an axis event for testing.
    fn make_axis_event(axis: AbsoluteAxisType, value: i32) -> InputEvent {
        InputEvent::new(EventType::ABSOLUTE, axis.0, value)
    }

    /// Helper to create a key event for testing.
    fn make_key_event(key: Key, pressed: bool) -> InputEvent {
        InputEvent::new(EventType::KEY, key.code(), if pressed { 1 } else { 0 })
    }

    // ==================== ControllerState Tests ====================

    #[test]
    fn test_controller_state_default() {
        let state = ControllerState::default();

        // Sticks should be centered
        assert_eq!(state.left_stick_x, AXIS_CENTER);
        assert_eq!(state.left_stick_y, AXIS_CENTER);
        assert_eq!(state.right_stick_x, AXIS_CENTER);
        assert_eq!(state.right_stick_y, AXIS_CENTER);

        // Triggers should be released
        assert_eq!(state.trigger_l2, AXIS_MIN);
        assert_eq!(state.trigger_r2, AXIS_MIN);

        // D-Pad should be centered
        assert_eq!(state.dpad_x, DPAD_RELEASED);
        assert_eq!(state.dpad_y, DPAD_RELEASED);

        // All buttons should be released
        assert!(!state.btn_cross);
        assert!(!state.btn_circle);
        assert!(!state.btn_square);
        assert!(!state.btn_triangle);
        assert!(!state.btn_l1);
        assert!(!state.btn_r1);
        assert!(!state.btn_l2);
        assert!(!state.btn_r2);
        assert!(!state.btn_share);
        assert!(!state.btn_options);
        assert!(!state.btn_ps);
        assert!(!state.btn_l3);
        assert!(!state.btn_r3);
        assert!(!state.btn_touchpad);
    }

    #[test]
    fn test_controller_state_new() {
        let state = ControllerState::new();
        assert_eq!(state, ControllerState::default());
    }

    #[test]
    fn test_any_stick_moved_centered() {
        let state = ControllerState::default();
        assert!(!state.any_stick_moved(10));
    }

    #[test]
    fn test_any_stick_moved_left_stick_x() {
        let mut state = ControllerState::default();
        state.left_stick_x = AXIS_CENTER + 20;
        assert!(state.any_stick_moved(10));
    }

    #[test]
    fn test_any_stick_moved_left_stick_y() {
        let mut state = ControllerState::default();
        state.left_stick_y = AXIS_CENTER - 20;
        assert!(state.any_stick_moved(10));
    }

    #[test]
    fn test_any_stick_moved_right_stick_x() {
        let mut state = ControllerState::default();
        state.right_stick_x = AXIS_MAX;
        assert!(state.any_stick_moved(10));
    }

    #[test]
    fn test_any_stick_moved_right_stick_y() {
        let mut state = ControllerState::default();
        state.right_stick_y = AXIS_MIN;
        assert!(state.any_stick_moved(10));
    }

    #[test]
    fn test_any_stick_moved_within_threshold() {
        let mut state = ControllerState::default();
        state.left_stick_x = AXIS_CENTER + 5;
        assert!(!state.any_stick_moved(10)); // Within threshold
        assert!(state.any_stick_moved(3));   // Exceeds threshold
    }

    #[test]
    fn test_any_button_pressed_none() {
        let state = ControllerState::default();
        assert!(!state.any_button_pressed());
    }

    #[test]
    fn test_any_button_pressed_face_buttons() {
        for button in ["cross", "circle", "square", "triangle"] {
            let mut state = ControllerState::default();
            match button {
                "cross" => state.btn_cross = true,
                "circle" => state.btn_circle = true,
                "square" => state.btn_square = true,
                "triangle" => state.btn_triangle = true,
                _ => unreachable!(),
            }
            assert!(state.any_button_pressed(), "Button {} should register", button);
        }
    }

    #[test]
    fn test_any_button_pressed_shoulder_buttons() {
        for button in ["l1", "r1", "l2", "r2"] {
            let mut state = ControllerState::default();
            match button {
                "l1" => state.btn_l1 = true,
                "r1" => state.btn_r1 = true,
                "l2" => state.btn_l2 = true,
                "r2" => state.btn_r2 = true,
                _ => unreachable!(),
            }
            assert!(state.any_button_pressed(), "Button {} should register", button);
        }
    }

    #[test]
    fn test_any_button_pressed_system_buttons() {
        for button in ["share", "options", "ps"] {
            let mut state = ControllerState::default();
            match button {
                "share" => state.btn_share = true,
                "options" => state.btn_options = true,
                "ps" => state.btn_ps = true,
                _ => unreachable!(),
            }
            assert!(state.any_button_pressed(), "Button {} should register", button);
        }
    }

    #[test]
    fn test_any_button_pressed_stick_clicks() {
        let mut state = ControllerState::default();
        state.btn_l3 = true;
        assert!(state.any_button_pressed());

        let mut state = ControllerState::default();
        state.btn_r3 = true;
        assert!(state.any_button_pressed());
    }

    #[test]
    fn test_any_button_pressed_touchpad() {
        let mut state = ControllerState::default();
        state.btn_touchpad = true;
        assert!(state.any_button_pressed());
    }

    #[test]
    fn test_any_trigger_pressed_none() {
        let state = ControllerState::default();
        assert!(!state.any_trigger_pressed(10));
    }

    #[test]
    fn test_any_trigger_pressed_l2() {
        let mut state = ControllerState::default();
        state.trigger_l2 = 200;
        assert!(state.any_trigger_pressed(10));
    }

    #[test]
    fn test_any_trigger_pressed_r2() {
        let mut state = ControllerState::default();
        state.trigger_r2 = 150;
        assert!(state.any_trigger_pressed(10));
    }

    #[test]
    fn test_any_trigger_pressed_threshold() {
        let mut state = ControllerState::default();
        state.trigger_l2 = 50;
        assert!(!state.any_trigger_pressed(100)); // Below threshold
        assert!(state.any_trigger_pressed(10));   // Above threshold
    }

    // ==================== EventMapper Tests ====================

    #[test]
    fn test_event_mapper_new() {
        let mapper = EventMapper::new();
        assert_eq!(*mapper.state(), ControllerState::default());
    }

    #[test]
    fn test_event_mapper_default() {
        let mapper = EventMapper::default();
        assert_eq!(*mapper.state(), ControllerState::default());
    }

    #[test]
    fn test_event_mapper_state_snapshot() {
        let mapper = EventMapper::new();
        let snapshot = mapper.state_snapshot();
        assert_eq!(snapshot, ControllerState::default());
    }

    #[test]
    fn test_event_mapper_reset() {
        let mut mapper = EventMapper::new();

        // Modify state
        let event = make_axis_event(AbsoluteAxisType::ABS_X, 200);
        mapper.process_event(&event);
        assert_eq!(mapper.state().left_stick_x, 200);

        // Reset
        mapper.reset();
        assert_eq!(mapper.state().left_stick_x, AXIS_CENTER);
    }

    // ==================== Axis Event Tests ====================

    #[test]
    fn test_process_left_stick_x() {
        let mut mapper = EventMapper::new();

        let event = make_axis_event(AbsoluteAxisType::ABS_X, 0);
        mapper.process_event(&event);
        assert_eq!(mapper.state().left_stick_x, 0);

        let event = make_axis_event(AbsoluteAxisType::ABS_X, 255);
        mapper.process_event(&event);
        assert_eq!(mapper.state().left_stick_x, 255);
    }

    #[test]
    fn test_process_left_stick_y() {
        let mut mapper = EventMapper::new();

        let event = make_axis_event(AbsoluteAxisType::ABS_Y, 50);
        mapper.process_event(&event);
        assert_eq!(mapper.state().left_stick_y, 50);
    }

    #[test]
    fn test_process_right_stick_x() {
        let mut mapper = EventMapper::new();

        let event = make_axis_event(AbsoluteAxisType::ABS_Z, 100);
        mapper.process_event(&event);
        assert_eq!(mapper.state().right_stick_x, 100);
    }

    #[test]
    fn test_process_right_stick_y() {
        let mut mapper = EventMapper::new();

        let event = make_axis_event(AbsoluteAxisType::ABS_RZ, 200);
        mapper.process_event(&event);
        assert_eq!(mapper.state().right_stick_y, 200);
    }

    #[test]
    fn test_process_trigger_l2() {
        let mut mapper = EventMapper::new();

        let event = make_axis_event(AbsoluteAxisType::ABS_RX, 128);
        mapper.process_event(&event);
        assert_eq!(mapper.state().trigger_l2, 128);
    }

    #[test]
    fn test_process_trigger_r2() {
        let mut mapper = EventMapper::new();

        let event = make_axis_event(AbsoluteAxisType::ABS_RY, 255);
        mapper.process_event(&event);
        assert_eq!(mapper.state().trigger_r2, 255);
    }

    #[test]
    fn test_process_dpad_x() {
        let mut mapper = EventMapper::new();

        // Press left
        let event = make_axis_event(AbsoluteAxisType::ABS_HAT0X, -1);
        mapper.process_event(&event);
        assert_eq!(mapper.state().dpad_x, -1);

        // Release
        let event = make_axis_event(AbsoluteAxisType::ABS_HAT0X, 0);
        mapper.process_event(&event);
        assert_eq!(mapper.state().dpad_x, 0);

        // Press right
        let event = make_axis_event(AbsoluteAxisType::ABS_HAT0X, 1);
        mapper.process_event(&event);
        assert_eq!(mapper.state().dpad_x, 1);
    }

    #[test]
    fn test_process_dpad_y() {
        let mut mapper = EventMapper::new();

        // Press up
        let event = make_axis_event(AbsoluteAxisType::ABS_HAT0Y, -1);
        mapper.process_event(&event);
        assert_eq!(mapper.state().dpad_y, -1);

        // Press down
        let event = make_axis_event(AbsoluteAxisType::ABS_HAT0Y, 1);
        mapper.process_event(&event);
        assert_eq!(mapper.state().dpad_y, 1);
    }

    // ==================== Key Event Tests ====================

    #[test]
    fn test_process_face_buttons() {
        let mut mapper = EventMapper::new();

        // Cross
        mapper.process_event(&make_key_event(Key::BTN_SOUTH, true));
        assert!(mapper.state().btn_cross);
        mapper.process_event(&make_key_event(Key::BTN_SOUTH, false));
        assert!(!mapper.state().btn_cross);

        // Circle
        mapper.process_event(&make_key_event(Key::BTN_EAST, true));
        assert!(mapper.state().btn_circle);

        // Square
        mapper.process_event(&make_key_event(Key::BTN_WEST, true));
        assert!(mapper.state().btn_square);

        // Triangle
        mapper.process_event(&make_key_event(Key::BTN_NORTH, true));
        assert!(mapper.state().btn_triangle);
    }

    #[test]
    fn test_process_shoulder_buttons() {
        let mut mapper = EventMapper::new();

        mapper.process_event(&make_key_event(Key::BTN_TL, true));
        assert!(mapper.state().btn_l1);

        mapper.process_event(&make_key_event(Key::BTN_TR, true));
        assert!(mapper.state().btn_r1);

        mapper.process_event(&make_key_event(Key::BTN_TL2, true));
        assert!(mapper.state().btn_l2);

        mapper.process_event(&make_key_event(Key::BTN_TR2, true));
        assert!(mapper.state().btn_r2);
    }

    #[test]
    fn test_process_system_buttons() {
        let mut mapper = EventMapper::new();

        mapper.process_event(&make_key_event(Key::BTN_SELECT, true));
        assert!(mapper.state().btn_share);

        mapper.process_event(&make_key_event(Key::BTN_START, true));
        assert!(mapper.state().btn_options);

        mapper.process_event(&make_key_event(Key::BTN_MODE, true));
        assert!(mapper.state().btn_ps);
    }

    #[test]
    fn test_process_stick_clicks() {
        let mut mapper = EventMapper::new();

        mapper.process_event(&make_key_event(Key::BTN_THUMBL, true));
        assert!(mapper.state().btn_l3);

        mapper.process_event(&make_key_event(Key::BTN_THUMBR, true));
        assert!(mapper.state().btn_r3);
    }

    #[test]
    fn test_process_touchpad() {
        let mut mapper = EventMapper::new();

        mapper.process_event(&make_key_event(Key::BTN_TOUCH, true));
        assert!(mapper.state().btn_touchpad);

        mapper.process_event(&make_key_event(Key::BTN_TOUCH, false));
        assert!(!mapper.state().btn_touchpad);
    }

    #[test]
    fn test_button_press_release_cycle() {
        let mut mapper = EventMapper::new();

        // Press L1
        mapper.process_event(&make_key_event(Key::BTN_TL, true));
        assert!(mapper.state().btn_l1);

        // Press R1 while L1 still held
        mapper.process_event(&make_key_event(Key::BTN_TR, true));
        assert!(mapper.state().btn_l1);
        assert!(mapper.state().btn_r1);

        // Release L1
        mapper.process_event(&make_key_event(Key::BTN_TL, false));
        assert!(!mapper.state().btn_l1);
        assert!(mapper.state().btn_r1);

        // Release R1
        mapper.process_event(&make_key_event(Key::BTN_TR, false));
        assert!(!mapper.state().btn_r1);
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_complex_input_sequence() {
        let mut mapper = EventMapper::new();

        // Simulate throttle up while pressing L1 (arming sequence)
        mapper.process_event(&make_axis_event(AbsoluteAxisType::ABS_Y, 0)); // Throttle up
        mapper.process_event(&make_key_event(Key::BTN_TL, true)); // L1 pressed

        assert_eq!(mapper.state().left_stick_y, 0);
        assert!(mapper.state().btn_l1);

        // Move right stick for roll
        mapper.process_event(&make_axis_event(AbsoluteAxisType::ABS_Z, 200));
        assert_eq!(mapper.state().right_stick_x, 200);

        // Press trigger
        mapper.process_event(&make_axis_event(AbsoluteAxisType::ABS_RY, 255));
        assert_eq!(mapper.state().trigger_r2, 255);
    }

    #[test]
    fn test_state_persistence_across_events() {
        let mut mapper = EventMapper::new();

        // Set multiple values
        mapper.process_event(&make_axis_event(AbsoluteAxisType::ABS_X, 100));
        mapper.process_event(&make_axis_event(AbsoluteAxisType::ABS_Y, 150));
        mapper.process_event(&make_key_event(Key::BTN_TL, true));

        // Verify all values persist
        let state = mapper.state();
        assert_eq!(state.left_stick_x, 100);
        assert_eq!(state.left_stick_y, 150);
        assert!(state.btn_l1);

        // Modify one value
        mapper.process_event(&make_axis_event(AbsoluteAxisType::ABS_X, 200));

        // Other values should remain unchanged
        let state = mapper.state();
        assert_eq!(state.left_stick_x, 200);
        assert_eq!(state.left_stick_y, 150);
        assert!(state.btn_l1);
    }

    #[test]
    fn test_unknown_axis_ignored() {
        let mut mapper = EventMapper::new();

        // ABS_MISC is not mapped
        let event = InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_MISC.0, 100);
        mapper.process_event(&event);

        // State should be unchanged
        assert_eq!(*mapper.state(), ControllerState::default());
    }

    #[test]
    fn test_sync_events_ignored() {
        let mut mapper = EventMapper::new();

        // SYN_REPORT events should be ignored
        let event = InputEvent::new(EventType::SYNCHRONIZATION, 0, 0);
        mapper.process_event(&event);

        assert_eq!(*mapper.state(), ControllerState::default());
    }

    // ==================== Constants Tests ====================

    #[test]
    fn test_axis_constants() {
        assert_eq!(AXIS_MIN, 0);
        assert_eq!(AXIS_MAX, 255);
        assert_eq!(AXIS_CENTER, 128);
    }

    #[test]
    fn test_dpad_constants() {
        assert_eq!(DPAD_RELEASED, 0);
        assert_eq!(DPAD_NEGATIVE, -1);
        assert_eq!(DPAD_POSITIVE, 1);
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_axis_boundary_values() {
        let mut mapper = EventMapper::new();

        // Test minimum
        mapper.process_event(&make_axis_event(AbsoluteAxisType::ABS_X, AXIS_MIN));
        assert_eq!(mapper.state().left_stick_x, AXIS_MIN);

        // Test maximum
        mapper.process_event(&make_axis_event(AbsoluteAxisType::ABS_X, AXIS_MAX));
        assert_eq!(mapper.state().left_stick_x, AXIS_MAX);

        // Test center
        mapper.process_event(&make_axis_event(AbsoluteAxisType::ABS_X, AXIS_CENTER));
        assert_eq!(mapper.state().left_stick_x, AXIS_CENTER);
    }

    #[test]
    fn test_controller_state_clone() {
        let mut state = ControllerState::default();
        state.btn_l1 = true;
        state.left_stick_x = 200;

        let cloned = state.clone();
        assert_eq!(state, cloned);
        assert!(cloned.btn_l1);
        assert_eq!(cloned.left_stick_x, 200);
    }

    #[test]
    fn test_controller_state_equality() {
        let state1 = ControllerState::default();
        let state2 = ControllerState::new();
        assert_eq!(state1, state2);

        let mut state3 = ControllerState::default();
        state3.btn_l1 = true;
        assert_ne!(state1, state3);
    }
}
