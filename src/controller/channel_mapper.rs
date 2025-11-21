//! # RC Channel Mapper Module
//!
//! Maps PS5 DualSense controller inputs to CRSF RC channel values.
//!
//! ## Channel Assignments
//!
//! | Channel | Input | Function |
//! |---------|-------|----------|
//! | CH1 | Right Stick X | Roll |
//! | CH2 | Right Stick Y | Pitch |
//! | CH3 | Left Stick Y | Throttle |
//! | CH4 | Left Stick X | Yaw |
//! | CH5 | L1 | ARM switch |
//! | CH6 | R1 | Flight mode |
//! | CH7 | L2 | Beeper |
//! | CH8 | R2 | Turtle mode |
//!
//! ## Value Ranges
//!
//! - Raw controller input: 0-255 (8-bit)
//! - CRSF output: 0-2047 (11-bit)
//! - Center value: 1024
//!
//! ## Usage
//!
//! ```
//! use fpv_bridge::controller::mapper::ControllerState;
//! use fpv_bridge::controller::channel_mapper::ChannelMapper;
//!
//! let state = ControllerState::default();
//! let mapper = ChannelMapper::new();
//! let channels = mapper.map_to_channels(&state);
//!
//! // Roll centered (approximately 1024, may vary due to integer rounding)
//! assert!((channels[0] as i32 - 1024).abs() <= 5);
//! ```

use super::mapper::{ControllerState, AXIS_MAX, AXIS_MIN};
use crate::crsf::protocol::{
    RcChannels, CRSF_CHANNEL_VALUE_CENTER, CRSF_CHANNEL_VALUE_MAX, CRSF_CHANNEL_VALUE_MIN,
    CRSF_NUM_CHANNELS,
};

/// CRSF value for switch OFF state.
pub const SWITCH_OFF: u16 = CRSF_CHANNEL_VALUE_MIN;

/// CRSF value for switch ON state.
pub const SWITCH_ON: u16 = CRSF_CHANNEL_VALUE_MAX;

/// Channel indices for semantic access.
pub mod channels {
    /// Roll - Right Stick X
    pub const ROLL: usize = 0;
    /// Pitch - Right Stick Y
    pub const PITCH: usize = 1;
    /// Throttle - Left Stick Y
    pub const THROTTLE: usize = 2;
    /// Yaw - Left Stick X
    pub const YAW: usize = 3;
    /// ARM switch - L1
    pub const ARM: usize = 4;
    /// Flight mode - R1
    pub const FLIGHT_MODE: usize = 5;
    /// Beeper - L2
    pub const BEEPER: usize = 6;
    /// Turtle mode - R2
    pub const TURTLE: usize = 7;
}

/// Maps controller state to CRSF RC channels.
///
/// Converts raw controller inputs (0-255) to CRSF channel values (0-2047)
/// and maps buttons to switch states.
///
/// # Examples
///
/// ```
/// use fpv_bridge::controller::mapper::ControllerState;
/// use fpv_bridge::controller::channel_mapper::ChannelMapper;
///
/// let mapper = ChannelMapper::new();
/// let state = ControllerState::default();
/// let channels = mapper.map_to_channels(&state);
///
/// // All sticks centered (approximately 1024)
/// assert!((channels[0] as i32 - 1024).abs() <= 5);
/// ```
#[derive(Debug, Clone)]
pub struct ChannelMapper {
    /// Channels to reverse (invert direction).
    reversed_channels: [bool; CRSF_NUM_CHANNELS],
}

impl Default for ChannelMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl ChannelMapper {
    /// Creates a new channel mapper with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            reversed_channels: [false; CRSF_NUM_CHANNELS],
        }
    }

    /// Creates a channel mapper with specified reversed channels.
    ///
    /// # Arguments
    ///
    /// * `reversed` - Slice of channel indices to reverse (1-based for user config)
    ///
    /// # Examples
    ///
    /// ```
    /// use fpv_bridge::controller::channel_mapper::ChannelMapper;
    ///
    /// // Reverse channels 1 and 2 (roll and pitch)
    /// let mapper = ChannelMapper::with_reversed(&[1, 2]);
    /// ```
    #[must_use]
    pub fn with_reversed(reversed: &[usize]) -> Self {
        let mut reversed_channels = [false; CRSF_NUM_CHANNELS];
        for &ch in reversed {
            if (1..=CRSF_NUM_CHANNELS).contains(&ch) {
                reversed_channels[ch - 1] = true;
            }
        }
        Self { reversed_channels }
    }

    /// Maps controller state to 16 RC channels.
    ///
    /// # Arguments
    ///
    /// * `state` - Current controller state from [`EventMapper`](super::mapper::EventMapper)
    ///
    /// # Returns
    ///
    /// Array of 16 channel values (0-2047).
    ///
    /// # Examples
    ///
    /// ```
    /// use fpv_bridge::controller::mapper::ControllerState;
    /// use fpv_bridge::controller::channel_mapper::{ChannelMapper, channels};
    ///
    /// let mut state = ControllerState::default();
    /// state.btn_l1 = true; // ARM pressed
    ///
    /// let mapper = ChannelMapper::new();
    /// let channels = mapper.map_to_channels(&state);
    ///
    /// assert_eq!(channels[channels::ARM], 2047); // Armed
    /// ```
    #[must_use]
    pub fn map_to_channels(&self, state: &ControllerState) -> RcChannels {
        let mut channels = [CRSF_CHANNEL_VALUE_CENTER; CRSF_NUM_CHANNELS];

        // CH1: Roll (Right Stick X)
        channels[channels::ROLL] = self.map_axis(state.right_stick_x, channels::ROLL);

        // CH2: Pitch (Right Stick Y) - inverted (up = forward = high value)
        channels[channels::PITCH] = self.map_axis_inverted(state.right_stick_y, channels::PITCH);

        // CH3: Throttle (Left Stick Y) - inverted (up = high throttle)
        channels[channels::THROTTLE] =
            self.map_axis_inverted(state.left_stick_y, channels::THROTTLE);

        // CH4: Yaw (Left Stick X)
        channels[channels::YAW] = self.map_axis(state.left_stick_x, channels::YAW);

        // CH5: ARM (L1 button)
        channels[channels::ARM] = self.map_button(state.btn_l1, channels::ARM);

        // CH6: Flight Mode (R1 button)
        channels[channels::FLIGHT_MODE] = self.map_button(state.btn_r1, channels::FLIGHT_MODE);

        // CH7: Beeper (L2 trigger - use analog value)
        channels[channels::BEEPER] = self.map_trigger(state.trigger_l2, channels::BEEPER);

        // CH8: Turtle Mode (R2 trigger - use analog value)
        channels[channels::TURTLE] = self.map_trigger(state.trigger_r2, channels::TURTLE);

        channels
    }

    /// Maps an axis value (0-255) to CRSF range (0-2047).
    fn map_axis(&self, value: i32, channel: usize) -> u16 {
        let mapped = Self::scale_axis_to_crsf(value);
        self.apply_reverse(mapped, channel)
    }

    /// Maps an inverted axis value (0-255) to CRSF range (0-2047).
    /// Inverted means 0 -> 2047 and 255 -> 0.
    fn map_axis_inverted(&self, value: i32, channel: usize) -> u16 {
        let inverted = AXIS_MAX - value;
        let mapped = Self::scale_axis_to_crsf(inverted);
        self.apply_reverse(mapped, channel)
    }

    /// Maps a trigger value (0-255) to CRSF range (0-2047).
    fn map_trigger(&self, value: i32, channel: usize) -> u16 {
        let mapped = Self::scale_axis_to_crsf(value);
        self.apply_reverse(mapped, channel)
    }

    /// Maps a button state to switch value.
    fn map_button(&self, pressed: bool, channel: usize) -> u16 {
        let value = if pressed { SWITCH_ON } else { SWITCH_OFF };
        self.apply_reverse(value, channel)
    }

    /// Scales raw axis value (0-255) to CRSF range (0-2047).
    #[inline]
    fn scale_axis_to_crsf(value: i32) -> u16 {
        // Clamp input to valid range
        let clamped = value.clamp(AXIS_MIN, AXIS_MAX);

        // Scale: (value / 255) * 2047
        // Using integer math: (value * 2047 + 127) / 255 for rounding
        let scaled = ((clamped as u32 * CRSF_CHANNEL_VALUE_MAX as u32) + 127) / 255;

        scaled as u16
    }

    /// Applies channel reversal if configured.
    #[inline]
    fn apply_reverse(&self, value: u16, channel: usize) -> u16 {
        if self.reversed_channels[channel] {
            CRSF_CHANNEL_VALUE_MAX - value
        } else {
            value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::mapper::AXIS_CENTER;

    // ==================== Scaling Tests ====================

    #[test]
    fn test_scale_axis_min() {
        let result = ChannelMapper::scale_axis_to_crsf(AXIS_MIN);
        assert_eq!(result, CRSF_CHANNEL_VALUE_MIN);
    }

    #[test]
    fn test_scale_axis_max() {
        let result = ChannelMapper::scale_axis_to_crsf(AXIS_MAX);
        assert_eq!(result, CRSF_CHANNEL_VALUE_MAX);
    }

    #[test]
    fn test_scale_axis_center() {
        let result = ChannelMapper::scale_axis_to_crsf(AXIS_CENTER);
        // 128 * 2047 / 255 = 1027 (approximately center)
        assert!((result as i32 - CRSF_CHANNEL_VALUE_CENTER as i32).abs() <= 5);
    }

    #[test]
    fn test_scale_axis_clamps_negative() {
        let result = ChannelMapper::scale_axis_to_crsf(-10);
        assert_eq!(result, CRSF_CHANNEL_VALUE_MIN);
    }

    #[test]
    fn test_scale_axis_clamps_overflow() {
        let result = ChannelMapper::scale_axis_to_crsf(300);
        assert_eq!(result, CRSF_CHANNEL_VALUE_MAX);
    }

    // ==================== ChannelMapper Tests ====================

    #[test]
    fn test_new_mapper() {
        let mapper = ChannelMapper::new();
        assert!(!mapper.reversed_channels[0]);
    }

    #[test]
    fn test_default_mapper() {
        let mapper = ChannelMapper::default();
        assert!(!mapper.reversed_channels[0]);
    }

    #[test]
    fn test_with_reversed_channels() {
        let mapper = ChannelMapper::with_reversed(&[1, 2]);
        assert!(mapper.reversed_channels[0]); // CH1
        assert!(mapper.reversed_channels[1]); // CH2
        assert!(!mapper.reversed_channels[2]); // CH3
    }

    #[test]
    fn test_with_reversed_out_of_range() {
        let mapper = ChannelMapper::with_reversed(&[0, 17, 100]);
        // All should be false (0 is invalid, 17+ out of range)
        for ch in &mapper.reversed_channels {
            assert!(!ch);
        }
    }

    // ==================== Channel Mapping Tests ====================

    #[test]
    fn test_map_centered_state() {
        let mapper = ChannelMapper::new();
        let state = ControllerState::default();
        let channels = mapper.map_to_channels(&state);

        // Sticks should be near center
        let center = CRSF_CHANNEL_VALUE_CENTER as i32;
        assert!((channels[channels::ROLL] as i32 - center).abs() <= 5);
        assert!((channels[channels::PITCH] as i32 - center).abs() <= 5);
        assert!((channels[channels::YAW] as i32 - center).abs() <= 5);

        // Throttle centered (inverted from 128)
        assert!((channels[channels::THROTTLE] as i32 - center).abs() <= 5);

        // Buttons off
        assert_eq!(channels[channels::ARM], SWITCH_OFF);
        assert_eq!(channels[channels::FLIGHT_MODE], SWITCH_OFF);

        // Triggers at min
        assert_eq!(channels[channels::BEEPER], CRSF_CHANNEL_VALUE_MIN);
        assert_eq!(channels[channels::TURTLE], CRSF_CHANNEL_VALUE_MIN);
    }

    #[test]
    fn test_map_roll_full_right() {
        let mapper = ChannelMapper::new();
        let mut state = ControllerState::default();
        state.right_stick_x = AXIS_MAX;

        let channels = mapper.map_to_channels(&state);
        assert_eq!(channels[channels::ROLL], CRSF_CHANNEL_VALUE_MAX);
    }

    #[test]
    fn test_map_roll_full_left() {
        let mapper = ChannelMapper::new();
        let mut state = ControllerState::default();
        state.right_stick_x = AXIS_MIN;

        let channels = mapper.map_to_channels(&state);
        assert_eq!(channels[channels::ROLL], CRSF_CHANNEL_VALUE_MIN);
    }

    #[test]
    fn test_map_pitch_full_forward() {
        let mapper = ChannelMapper::new();
        let mut state = ControllerState::default();
        state.right_stick_y = AXIS_MIN; // Up on stick = forward

        let channels = mapper.map_to_channels(&state);
        assert_eq!(channels[channels::PITCH], CRSF_CHANNEL_VALUE_MAX);
    }

    #[test]
    fn test_map_pitch_full_back() {
        let mapper = ChannelMapper::new();
        let mut state = ControllerState::default();
        state.right_stick_y = AXIS_MAX; // Down on stick = back

        let channels = mapper.map_to_channels(&state);
        assert_eq!(channels[channels::PITCH], CRSF_CHANNEL_VALUE_MIN);
    }

    #[test]
    fn test_map_throttle_full() {
        let mapper = ChannelMapper::new();
        let mut state = ControllerState::default();
        state.left_stick_y = AXIS_MIN; // Up = full throttle

        let channels = mapper.map_to_channels(&state);
        assert_eq!(channels[channels::THROTTLE], CRSF_CHANNEL_VALUE_MAX);
    }

    #[test]
    fn test_map_throttle_min() {
        let mapper = ChannelMapper::new();
        let mut state = ControllerState::default();
        state.left_stick_y = AXIS_MAX; // Down = min throttle

        let channels = mapper.map_to_channels(&state);
        assert_eq!(channels[channels::THROTTLE], CRSF_CHANNEL_VALUE_MIN);
    }

    #[test]
    fn test_map_yaw_full_right() {
        let mapper = ChannelMapper::new();
        let mut state = ControllerState::default();
        state.left_stick_x = AXIS_MAX;

        let channels = mapper.map_to_channels(&state);
        assert_eq!(channels[channels::YAW], CRSF_CHANNEL_VALUE_MAX);
    }

    #[test]
    fn test_map_arm_button() {
        let mapper = ChannelMapper::new();
        let mut state = ControllerState::default();
        state.btn_l1 = true;

        let channels = mapper.map_to_channels(&state);
        assert_eq!(channels[channels::ARM], SWITCH_ON);
    }

    #[test]
    fn test_map_flight_mode_button() {
        let mapper = ChannelMapper::new();
        let mut state = ControllerState::default();
        state.btn_r1 = true;

        let channels = mapper.map_to_channels(&state);
        assert_eq!(channels[channels::FLIGHT_MODE], SWITCH_ON);
    }

    #[test]
    fn test_map_trigger_l2() {
        let mapper = ChannelMapper::new();
        let mut state = ControllerState::default();
        state.trigger_l2 = AXIS_MAX;

        let channels = mapper.map_to_channels(&state);
        assert_eq!(channels[channels::BEEPER], CRSF_CHANNEL_VALUE_MAX);
    }

    #[test]
    fn test_map_trigger_r2() {
        let mapper = ChannelMapper::new();
        let mut state = ControllerState::default();
        state.trigger_r2 = 128;

        let channels = mapper.map_to_channels(&state);
        // Should be approximately half
        let expected = CRSF_CHANNEL_VALUE_MAX / 2;
        assert!((channels[channels::TURTLE] as i32 - expected as i32).abs() <= 10);
    }

    // ==================== Reversal Tests ====================

    #[test]
    fn test_reversed_roll() {
        let mapper = ChannelMapper::with_reversed(&[1]);
        let mut state = ControllerState::default();
        state.right_stick_x = AXIS_MAX;

        let channels = mapper.map_to_channels(&state);
        assert_eq!(channels[channels::ROLL], CRSF_CHANNEL_VALUE_MIN);
    }

    #[test]
    fn test_reversed_pitch() {
        let mapper = ChannelMapper::with_reversed(&[2]);
        let mut state = ControllerState::default();
        state.right_stick_y = AXIS_MIN; // Forward

        let channels = mapper.map_to_channels(&state);
        assert_eq!(channels[channels::PITCH], CRSF_CHANNEL_VALUE_MIN);
    }

    #[test]
    fn test_reversed_arm_button() {
        let mapper = ChannelMapper::with_reversed(&[5]);
        let mut state = ControllerState::default();
        state.btn_l1 = true;

        let channels = mapper.map_to_channels(&state);
        assert_eq!(channels[channels::ARM], SWITCH_OFF);
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_stick_deflection() {
        let mapper = ChannelMapper::new();
        let mut state = ControllerState::default();

        // All sticks to max deflection
        state.left_stick_x = AXIS_MAX;
        state.left_stick_y = AXIS_MIN; // Throttle up
        state.right_stick_x = AXIS_MAX;
        state.right_stick_y = AXIS_MIN; // Pitch forward

        let channels = mapper.map_to_channels(&state);

        assert_eq!(channels[channels::ROLL], CRSF_CHANNEL_VALUE_MAX);
        assert_eq!(channels[channels::PITCH], CRSF_CHANNEL_VALUE_MAX);
        assert_eq!(channels[channels::THROTTLE], CRSF_CHANNEL_VALUE_MAX);
        assert_eq!(channels[channels::YAW], CRSF_CHANNEL_VALUE_MAX);
    }

    #[test]
    fn test_all_buttons_pressed() {
        let mapper = ChannelMapper::new();
        let mut state = ControllerState::default();

        state.btn_l1 = true;
        state.btn_r1 = true;
        state.trigger_l2 = AXIS_MAX;
        state.trigger_r2 = AXIS_MAX;

        let channels = mapper.map_to_channels(&state);

        assert_eq!(channels[channels::ARM], SWITCH_ON);
        assert_eq!(channels[channels::FLIGHT_MODE], SWITCH_ON);
        assert_eq!(channels[channels::BEEPER], CRSF_CHANNEL_VALUE_MAX);
        assert_eq!(channels[channels::TURTLE], CRSF_CHANNEL_VALUE_MAX);
    }

    // ==================== Constants Tests ====================

    #[test]
    fn test_switch_constants() {
        assert_eq!(SWITCH_OFF, 0);
        assert_eq!(SWITCH_ON, 2047);
    }

    #[test]
    fn test_channel_indices() {
        assert_eq!(channels::ROLL, 0);
        assert_eq!(channels::PITCH, 1);
        assert_eq!(channels::THROTTLE, 2);
        assert_eq!(channels::YAW, 3);
        assert_eq!(channels::ARM, 4);
        assert_eq!(channels::FLIGHT_MODE, 5);
        assert_eq!(channels::BEEPER, 6);
        assert_eq!(channels::TURTLE, 7);
    }
}
