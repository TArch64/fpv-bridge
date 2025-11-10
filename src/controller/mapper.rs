//! # Channel Mapper
//!
//! Maps controller inputs to RC channels with deadzones and expo curves.

use super::ControllerState;
use crate::config::{ControllerConfig, ChannelConfig};
use crate::crsf::protocol::{CRSF_CHANNEL_VALUE_MIN, CRSF_CHANNEL_VALUE_MAX};

/// Channel mapper that applies deadzones, expo curves, and maps to RC channels
pub struct ChannelMapper {
    deadzone_stick: f32,
    deadzone_trigger: f32,
    expo_roll: f32,
    expo_pitch: f32,
    expo_yaw: f32,
    expo_throttle: f32,
    throttle_min: u16,
    throttle_max: u16,
    center: u16,
    channel_reverse: Vec<usize>,
}

impl ChannelMapper {
    /// Create a new channel mapper
    ///
    /// # Arguments
    ///
    /// * `controller_config` - Controller configuration (deadzones, expo)
    /// * `channel_config` - Channel configuration (ranges, reverse)
    pub fn new(controller_config: &ControllerConfig, channel_config: &ChannelConfig) -> Self {
        Self {
            deadzone_stick: controller_config.deadzone_stick,
            deadzone_trigger: controller_config.deadzone_trigger,
            expo_roll: controller_config.expo_roll,
            expo_pitch: controller_config.expo_pitch,
            expo_yaw: controller_config.expo_yaw,
            expo_throttle: controller_config.expo_throttle,
            throttle_min: channel_config.throttle_min,
            throttle_max: channel_config.throttle_max,
            center: channel_config.center,
            channel_reverse: channel_config.channel_reverse.clone(),
        }
    }

    /// Map controller state to 16 RC channels
    ///
    /// Channel mapping:
    /// - CH1 (Roll): Right stick X
    /// - CH2 (Pitch): Right stick Y
    /// - CH3 (Throttle): Left stick Y
    /// - CH4 (Yaw): Left stick X
    /// - CH5: L1 button (aux switch)
    /// - CH6: R1 button (aux switch)
    /// - CH7-16: Default center
    ///
    /// # Arguments
    ///
    /// * `state` - Controller state
    ///
    /// # Returns
    ///
    /// * `[u16; 16]` - 16 RC channel values (0-2047)
    pub fn map_to_channels(&self, state: &ControllerState) -> [u16; 16] {
        let mut channels = [self.center; 16];

        // CH1: Roll (Right stick X)
        channels[0] = self.map_stick_to_channel(
            state.right_stick_x,
            self.expo_roll,
            true,
        );

        // CH2: Pitch (Right stick Y)
        channels[1] = self.map_stick_to_channel(
            state.right_stick_y,
            self.expo_pitch,
            true,
        );

        // CH3: Throttle (Left stick Y) - special handling, no center
        channels[2] = self.map_throttle_to_channel(state.left_stick_y);

        // CH4: Yaw (Left stick X)
        channels[3] = self.map_stick_to_channel(
            state.left_stick_x,
            self.expo_yaw,
            true,
        );

        // CH5: L1 button (aux switch) - 2-position
        channels[4] = if state.button_l1 {
            CRSF_CHANNEL_VALUE_MAX
        } else {
            CRSF_CHANNEL_VALUE_MIN
        };

        // CH6: R1 button (aux switch) - 2-position
        channels[5] = if state.button_r1 {
            CRSF_CHANNEL_VALUE_MAX
        } else {
            CRSF_CHANNEL_VALUE_MIN
        };

        // Apply channel reverse
        for &ch_idx in &self.channel_reverse {
            if ch_idx < 16 {
                channels[ch_idx] = self.reverse_channel(channels[ch_idx]);
            }
        }

        channels
    }

    /// Map a stick axis to a channel value with expo curve
    ///
    /// # Arguments
    ///
    /// * `value` - Input value (-1.0 to 1.0)
    /// * `expo` - Expo curve factor (0.0 = linear, 1.0 = maximum expo)
    /// * `apply_deadzone` - Whether to apply deadzone
    ///
    /// # Returns
    ///
    /// * `u16` - Channel value (0-2047)
    fn map_stick_to_channel(&self, value: f32, expo: f32, apply_deadzone: bool) -> u16 {
        // Apply deadzone
        let mut val = value;
        if apply_deadzone {
            val = self.apply_deadzone(val, self.deadzone_stick);
        }

        // Apply expo curve
        val = self.apply_expo(val, expo);

        // Map to channel range centered at center value
        let range = (CRSF_CHANNEL_VALUE_MAX - CRSF_CHANNEL_VALUE_MIN) as f32 / 2.0;
        let channel = self.center as f32 + val * range;

        channel.clamp(CRSF_CHANNEL_VALUE_MIN as f32, CRSF_CHANNEL_VALUE_MAX as f32) as u16
    }

    /// Map throttle to channel value
    ///
    /// Throttle is special: -1.0 = throttle_min, +1.0 = throttle_max
    ///
    /// # Arguments
    ///
    /// * `value` - Input value (-1.0 to 1.0)
    ///
    /// # Returns
    ///
    /// * `u16` - Channel value (throttle_min to throttle_max)
    fn map_throttle_to_channel(&self, value: f32) -> u16 {
        // Apply deadzone
        let mut val = self.apply_deadzone(value, self.deadzone_stick);

        // Apply expo curve
        val = self.apply_expo(val, self.expo_throttle);

        // Map -1.0..1.0 to throttle_min..throttle_max
        let range = (self.throttle_max - self.throttle_min) as f32;
        let channel = self.throttle_min as f32 + (val + 1.0) / 2.0 * range;

        channel.clamp(self.throttle_min as f32, self.throttle_max as f32) as u16
    }

    /// Apply deadzone to input value
    ///
    /// # Arguments
    ///
    /// * `value` - Input value (-1.0 to 1.0)
    /// * `deadzone` - Deadzone threshold (0.0 to 1.0)
    ///
    /// # Returns
    ///
    /// * `f32` - Value with deadzone applied (-1.0 to 1.0)
    fn apply_deadzone(&self, value: f32, deadzone: f32) -> f32 {
        let abs_val = value.abs();

        if abs_val < deadzone {
            0.0
        } else {
            // Scale value so that deadzone..1.0 maps to 0.0..1.0
            let sign = value.signum();
            let scaled = (abs_val - deadzone) / (1.0 - deadzone);
            sign * scaled
        }
    }

    /// Apply exponential curve to input value
    ///
    /// Expo curves make center stick less sensitive while maintaining full range.
    /// Formula: output = input * (1 - expo) + input^3 * expo
    ///
    /// # Arguments
    ///
    /// * `value` - Input value (-1.0 to 1.0)
    /// * `expo` - Expo factor (0.0 = linear, 1.0 = maximum expo)
    ///
    /// # Returns
    ///
    /// * `f32` - Value with expo applied (-1.0 to 1.0)
    fn apply_expo(&self, value: f32, expo: f32) -> f32 {
        let linear = value;
        let cubic = value.powi(3);

        linear * (1.0 - expo) + cubic * expo
    }

    /// Reverse a channel value
    ///
    /// # Arguments
    ///
    /// * `value` - Channel value (0-2047)
    ///
    /// # Returns
    ///
    /// * `u16` - Reversed channel value (0-2047)
    fn reverse_channel(&self, value: u16) -> u16 {
        CRSF_CHANNEL_VALUE_MAX - value + CRSF_CHANNEL_VALUE_MIN
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ControllerConfig, ChannelConfig};

    fn test_mapper() -> ChannelMapper {
        let controller_config = ControllerConfig {
            device_path: String::new(),
            deadzone_stick: 0.1,
            deadzone_trigger: 0.05,
            expo_roll: 0.5,
            expo_pitch: 0.5,
            expo_yaw: 0.3,
            expo_throttle: 0.2,
        };

        let channel_config = ChannelConfig {
            throttle_min: 172,
            throttle_max: 1811,
            center: 992,
            channel_reverse: vec![],
        };

        ChannelMapper::new(&controller_config, &channel_config)
    }

    #[test]
    fn test_deadzone_within_threshold() {
        let mapper = test_mapper();
        assert_eq!(mapper.apply_deadzone(0.05, 0.1), 0.0);
        assert_eq!(mapper.apply_deadzone(-0.05, 0.1), 0.0);
    }

    #[test]
    fn test_deadzone_outside_threshold() {
        let mapper = test_mapper();
        let result = mapper.apply_deadzone(0.5, 0.1);
        assert!(result > 0.0);
        assert!(result <= 1.0);
    }

    #[test]
    fn test_expo_zero_is_linear() {
        let mapper = test_mapper();
        assert_eq!(mapper.apply_expo(0.5, 0.0), 0.5);
        assert_eq!(mapper.apply_expo(-0.5, 0.0), -0.5);
    }

    #[test]
    fn test_expo_reduces_center_sensitivity() {
        let mapper = test_mapper();
        let with_expo = mapper.apply_expo(0.5, 0.5);
        // With expo, center should be less sensitive (smaller output for same input)
        assert!(with_expo < 0.5);
        assert!(with_expo > 0.0);
    }

    #[test]
    fn test_map_channels_centered() {
        let mapper = test_mapper();
        let state = ControllerState::default(); // All zeros (sticks centered at 0.0)

        let channels = mapper.map_to_channels(&state);

        // Roll, pitch, yaw should be at center
        assert_eq!(channels[0], 992);
        assert_eq!(channels[1], 992);
        assert_eq!(channels[3], 992);

        // Throttle at mid-range (stick centered = 0.0 normalized = mid throttle)
        // Mid-range = (172 + 1811) / 2 = 991.5 ≈ 991
        assert_eq!(channels[2], 991);
    }

    #[test]
    fn test_reverse_channel() {
        let mapper = test_mapper();
        assert_eq!(mapper.reverse_channel(0), 2047);
        assert_eq!(mapper.reverse_channel(2047), 0);
        assert_eq!(mapper.reverse_channel(1024), 1023);
    }

    #[test]
    fn test_button_mapping() {
        let mapper = test_mapper();
        let mut state = ControllerState::default();

        state.button_l1 = true;
        state.button_r1 = false;

        let channels = mapper.map_to_channels(&state);

        assert_eq!(channels[4], CRSF_CHANNEL_VALUE_MAX); // L1 pressed
        assert_eq!(channels[5], CRSF_CHANNEL_VALUE_MIN); // R1 not pressed
    }
}
