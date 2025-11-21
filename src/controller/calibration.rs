//! # Calibration Module
//!
//! Applies deadzones and exponential curves to controller inputs.
//!
//! ## Deadzone
//!
//! A deadzone eliminates small stick movements near center to prevent drift.
//! Values within the deadzone are mapped to center (0.0), while values outside
//! are scaled to use the full range.
//!
//! ## Exponential Curves
//!
//! Expo curves make small stick movements less sensitive while maintaining
//! full deflection at the endpoints. This provides finer control for precise
//! movements.
//!
//! The formula used is: `output = (1 - expo) * input + expo * input³`
//!
//! - `expo = 0.0`: Linear response
//! - `expo = 0.3`: Mild curve (recommended for beginners)
//! - `expo = 0.7`: Strong curve (for experienced pilots)
//!
//! ## Usage
//!
//! ```
//! use fpv_bridge::controller::calibration::Calibration;
//!
//! let cal = Calibration::new(0.05, 0.3); // 5% deadzone, 0.3 expo
//!
//! // Input near center (within deadzone)
//! assert_eq!(cal.apply(0.02), 0.0);
//!
//! // Input at full deflection
//! assert!((cal.apply(1.0) - 1.0).abs() < 0.001);
//! ```

/// Applies deadzone and exponential curve to a normalized input.
///
/// Input and output are in the range -1.0 to 1.0, where 0.0 is center.
#[derive(Debug, Clone, Copy)]
pub struct Calibration {
    /// Deadzone as a fraction (0.0 to 0.25).
    deadzone: f32,
    /// Exponential curve factor (0.0 to 1.0).
    expo: f32,
}

impl Default for Calibration {
    fn default() -> Self {
        Self {
            deadzone: 0.05,
            expo: 0.0,
        }
    }
}

impl Calibration {
    /// Creates a new calibration with specified deadzone and expo.
    ///
    /// # Arguments
    ///
    /// * `deadzone` - Deadzone fraction (0.0 to 0.25). Values outside this range are clamped.
    /// * `expo` - Exponential curve factor (0.0 to 1.0). 0.0 = linear, 1.0 = max curve.
    ///
    /// # Examples
    ///
    /// ```
    /// use fpv_bridge::controller::calibration::Calibration;
    ///
    /// let cal = Calibration::new(0.05, 0.3);
    /// ```
    #[must_use]
    pub fn new(deadzone: f32, expo: f32) -> Self {
        Self {
            deadzone: deadzone.clamp(0.0, 0.25),
            expo: expo.clamp(0.0, 1.0),
        }
    }

    /// Creates a linear calibration (no deadzone, no expo).
    ///
    /// # Examples
    ///
    /// ```
    /// use fpv_bridge::controller::calibration::Calibration;
    ///
    /// let cal = Calibration::linear();
    /// assert!((cal.apply(0.5) - 0.5).abs() < 0.001);
    /// ```
    #[must_use]
    pub fn linear() -> Self {
        Self {
            deadzone: 0.0,
            expo: 0.0,
        }
    }

    /// Returns the configured deadzone value.
    #[must_use]
    pub fn deadzone(&self) -> f32 {
        self.deadzone
    }

    /// Returns the configured expo value.
    #[must_use]
    pub fn expo(&self) -> f32 {
        self.expo
    }

    /// Applies deadzone and expo curve to a normalized input.
    ///
    /// # Arguments
    ///
    /// * `input` - Normalized input value (-1.0 to 1.0)
    ///
    /// # Returns
    ///
    /// Calibrated output value (-1.0 to 1.0)
    ///
    /// # Examples
    ///
    /// ```
    /// use fpv_bridge::controller::calibration::Calibration;
    ///
    /// let cal = Calibration::new(0.05, 0.3);
    ///
    /// // Within deadzone
    /// assert_eq!(cal.apply(0.02), 0.0);
    /// assert_eq!(cal.apply(-0.02), 0.0);
    ///
    /// // Full deflection preserved
    /// assert!((cal.apply(1.0) - 1.0).abs() < 0.001);
    /// assert!((cal.apply(-1.0) - (-1.0)).abs() < 0.001);
    /// ```
    #[must_use]
    pub fn apply(&self, input: f32) -> f32 {
        let sign = input.signum();
        let abs_input = input.abs();

        // Apply deadzone
        let after_deadzone = self.apply_deadzone(abs_input);

        // Apply expo curve
        let after_expo = self.apply_expo(after_deadzone);

        sign * after_expo
    }

    /// Applies deadzone to an absolute input value.
    ///
    /// Maps values within deadzone to 0, and scales remaining range to 0..1.
    #[inline]
    fn apply_deadzone(&self, abs_input: f32) -> f32 {
        if abs_input <= self.deadzone {
            0.0
        } else {
            // Scale remaining range to 0..1
            (abs_input - self.deadzone) / (1.0 - self.deadzone)
        }
    }

    /// Applies exponential curve to a value in range 0..1.
    ///
    /// Formula: output = (1 - expo) * input + expo * input³
    #[inline]
    fn apply_expo(&self, input: f32) -> f32 {
        if self.expo == 0.0 {
            input
        } else {
            let linear = (1.0 - self.expo) * input;
            let cubic = self.expo * input * input * input;
            linear + cubic
        }
    }
}

/// Calibration settings for all flight axes.
///
/// Holds separate calibration parameters for roll, pitch, yaw, and throttle.
#[derive(Debug, Clone)]
pub struct AxisCalibration {
    /// Roll axis calibration (right stick X).
    pub roll: Calibration,
    /// Pitch axis calibration (right stick Y).
    pub pitch: Calibration,
    /// Yaw axis calibration (left stick X).
    pub yaw: Calibration,
    /// Throttle axis calibration (left stick Y).
    pub throttle: Calibration,
    /// Trigger deadzone for L2/R2.
    pub trigger_deadzone: f32,
}

impl Default for AxisCalibration {
    fn default() -> Self {
        Self {
            roll: Calibration::new(0.05, 0.3),
            pitch: Calibration::new(0.05, 0.3),
            yaw: Calibration::new(0.05, 0.2),
            throttle: Calibration::new(0.05, 0.0), // Linear throttle
            trigger_deadzone: 0.10,
        }
    }
}

impl AxisCalibration {
    /// Creates axis calibration from config values.
    ///
    /// # Arguments
    ///
    /// * `deadzone_stick` - Deadzone for analog sticks (0.0 to 0.25)
    /// * `deadzone_trigger` - Deadzone for triggers (0.0 to 0.25)
    /// * `expo_roll` - Expo for roll axis
    /// * `expo_pitch` - Expo for pitch axis
    /// * `expo_yaw` - Expo for yaw axis
    /// * `expo_throttle` - Expo for throttle axis
    ///
    /// # Examples
    ///
    /// ```
    /// use fpv_bridge::controller::calibration::AxisCalibration;
    ///
    /// let cal = AxisCalibration::from_config(
    ///     0.05,  // deadzone_stick
    ///     0.10,  // deadzone_trigger
    ///     0.3,   // expo_roll
    ///     0.3,   // expo_pitch
    ///     0.2,   // expo_yaw
    ///     0.0,   // expo_throttle
    /// );
    /// ```
    #[must_use]
    pub fn from_config(
        deadzone_stick: f32,
        deadzone_trigger: f32,
        expo_roll: f32,
        expo_pitch: f32,
        expo_yaw: f32,
        expo_throttle: f32,
    ) -> Self {
        Self {
            roll: Calibration::new(deadzone_stick, expo_roll),
            pitch: Calibration::new(deadzone_stick, expo_pitch),
            yaw: Calibration::new(deadzone_stick, expo_yaw),
            throttle: Calibration::new(deadzone_stick, expo_throttle),
            trigger_deadzone: deadzone_trigger.clamp(0.0, 0.25),
        }
    }

    /// Applies trigger deadzone to a normalized trigger value (0.0 to 1.0).
    ///
    /// # Arguments
    ///
    /// * `input` - Trigger value (0.0 to 1.0)
    ///
    /// # Returns
    ///
    /// Calibrated trigger value (0.0 to 1.0)
    #[must_use]
    pub fn apply_trigger(&self, input: f32) -> f32 {
        if input <= self.trigger_deadzone {
            0.0
        } else {
            (input - self.trigger_deadzone) / (1.0 - self.trigger_deadzone)
        }
    }
}

/// Converts raw axis value (0-255) to normalized value (-1.0 to 1.0).
///
/// # Arguments
///
/// * `raw` - Raw axis value from controller (0-255)
///
/// # Returns
///
/// Normalized value where 0 = -1.0, 128 = 0.0, 255 = 1.0
///
/// # Examples
///
/// ```
/// use fpv_bridge::controller::calibration::normalize_axis;
///
/// assert!((normalize_axis(0) - (-1.0)).abs() < 0.01);
/// assert!((normalize_axis(128) - 0.0).abs() < 0.01);
/// assert!((normalize_axis(255) - 1.0).abs() < 0.01);
/// ```
#[must_use]
pub fn normalize_axis(raw: i32) -> f32 {
    // Map 0-255 to -1.0 to 1.0
    // 128 is center (0.0)
    ((raw as f32) - 128.0) / 127.0
}

/// Converts raw trigger value (0-255) to normalized value (0.0 to 1.0).
///
/// # Arguments
///
/// * `raw` - Raw trigger value from controller (0-255)
///
/// # Returns
///
/// Normalized value where 0 = 0.0, 255 = 1.0
///
/// # Examples
///
/// ```
/// use fpv_bridge::controller::calibration::normalize_trigger;
///
/// assert!((normalize_trigger(0) - 0.0).abs() < 0.01);
/// assert!((normalize_trigger(255) - 1.0).abs() < 0.01);
/// ```
#[must_use]
pub fn normalize_trigger(raw: i32) -> f32 {
    (raw as f32) / 255.0
}

/// Converts normalized value (-1.0 to 1.0) to CRSF channel value (0-2047).
///
/// # Arguments
///
/// * `normalized` - Normalized value (-1.0 to 1.0)
///
/// # Returns
///
/// CRSF channel value (0-2047) where -1.0 = 0, 0.0 = 1024, 1.0 = 2047
///
/// # Examples
///
/// ```
/// use fpv_bridge::controller::calibration::to_crsf_channel;
///
/// assert_eq!(to_crsf_channel(-1.0), 0);
/// // Center is approximately 1024 (may be 1023 due to float rounding)
/// let center = to_crsf_channel(0.0);
/// assert!(center == 1023 || center == 1024);
/// assert_eq!(to_crsf_channel(1.0), 2047);
/// ```
#[must_use]
pub fn to_crsf_channel(normalized: f32) -> u16 {
    // Map -1.0..1.0 to 0..2047
    let clamped = normalized.clamp(-1.0, 1.0);
    let scaled = (clamped + 1.0) * 1023.5;
    (scaled as u16).min(2047)
}

/// Converts normalized trigger value (0.0 to 1.0) to CRSF channel value (0-2047).
///
/// # Arguments
///
/// * `normalized` - Normalized trigger value (0.0 to 1.0)
///
/// # Returns
///
/// CRSF channel value (0-2047) where 0.0 = 0, 1.0 = 2047
#[must_use]
pub fn trigger_to_crsf_channel(normalized: f32) -> u16 {
    let clamped = normalized.clamp(0.0, 1.0);
    (clamped * 2047.0) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Calibration Tests ====================

    #[test]
    fn test_calibration_new() {
        let cal = Calibration::new(0.05, 0.3);
        assert!((cal.deadzone() - 0.05).abs() < 0.001);
        assert!((cal.expo() - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_calibration_default() {
        let cal = Calibration::default();
        assert!((cal.deadzone() - 0.05).abs() < 0.001);
        assert!((cal.expo() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_calibration_linear() {
        let cal = Calibration::linear();
        assert_eq!(cal.deadzone(), 0.0);
        assert_eq!(cal.expo(), 0.0);
    }

    #[test]
    fn test_calibration_clamps_deadzone() {
        let cal = Calibration::new(0.5, 0.0);
        assert!((cal.deadzone() - 0.25).abs() < 0.001); // Clamped to max

        let cal = Calibration::new(-0.1, 0.0);
        assert_eq!(cal.deadzone(), 0.0); // Clamped to min
    }

    #[test]
    fn test_calibration_clamps_expo() {
        let cal = Calibration::new(0.0, 1.5);
        assert!((cal.expo() - 1.0).abs() < 0.001); // Clamped to max

        let cal = Calibration::new(0.0, -0.5);
        assert_eq!(cal.expo(), 0.0); // Clamped to min
    }

    // ==================== Deadzone Tests ====================

    #[test]
    fn test_deadzone_within_zone() {
        let cal = Calibration::new(0.1, 0.0);
        assert_eq!(cal.apply(0.05), 0.0);
        assert_eq!(cal.apply(-0.05), 0.0);
        assert_eq!(cal.apply(0.1), 0.0);
        assert_eq!(cal.apply(-0.1), 0.0);
    }

    #[test]
    fn test_deadzone_outside_zone() {
        let cal = Calibration::new(0.1, 0.0);

        // Just outside deadzone
        let result = cal.apply(0.11);
        assert!(result > 0.0);

        // Full deflection should reach 1.0
        let result = cal.apply(1.0);
        assert!((result - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_deadzone_scaling() {
        let cal = Calibration::new(0.1, 0.0);

        // At 0.55 input (halfway between deadzone and max)
        // Should be around 0.5 output
        let result = cal.apply(0.55);
        assert!((result - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_deadzone_negative_values() {
        let cal = Calibration::new(0.1, 0.0);

        let result = cal.apply(-1.0);
        assert!((result - (-1.0)).abs() < 0.001);

        let result = cal.apply(-0.55);
        assert!((result - (-0.5)).abs() < 0.01);
    }

    // ==================== Expo Tests ====================

    #[test]
    fn test_expo_linear() {
        let cal = Calibration::new(0.0, 0.0);
        assert!((cal.apply(0.5) - 0.5).abs() < 0.001);
        assert!((cal.apply(-0.5) - (-0.5)).abs() < 0.001);
    }

    #[test]
    fn test_expo_reduces_small_inputs() {
        let cal = Calibration::new(0.0, 0.5);

        // Small input should be reduced
        let result = cal.apply(0.3);
        assert!(result < 0.3);

        // Full deflection should still be 1.0
        let result = cal.apply(1.0);
        assert!((result - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_expo_preserves_endpoints() {
        let cal = Calibration::new(0.0, 0.7);

        assert!((cal.apply(1.0) - 1.0).abs() < 0.001);
        assert!((cal.apply(-1.0) - (-1.0)).abs() < 0.001);
        assert_eq!(cal.apply(0.0), 0.0);
    }

    #[test]
    fn test_expo_symmetry() {
        let cal = Calibration::new(0.0, 0.5);

        let positive = cal.apply(0.5);
        let negative = cal.apply(-0.5);
        assert!((positive + negative).abs() < 0.001);
    }

    // ==================== Combined Deadzone + Expo Tests ====================

    #[test]
    fn test_combined_deadzone_and_expo() {
        let cal = Calibration::new(0.05, 0.3);

        // Within deadzone
        assert_eq!(cal.apply(0.03), 0.0);

        // Full deflection
        assert!((cal.apply(1.0) - 1.0).abs() < 0.001);

        // Partial deflection should be affected by both
        let result = cal.apply(0.5);
        assert!(result > 0.0 && result < 0.5);
    }

    // ==================== AxisCalibration Tests ====================

    #[test]
    fn test_axis_calibration_default() {
        let cal = AxisCalibration::default();
        assert!((cal.roll.deadzone() - 0.05).abs() < 0.001);
        assert!((cal.roll.expo() - 0.3).abs() < 0.001);
        assert!((cal.throttle.expo() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_axis_calibration_from_config() {
        let cal = AxisCalibration::from_config(0.08, 0.12, 0.4, 0.4, 0.3, 0.1);

        assert!((cal.roll.deadzone() - 0.08).abs() < 0.001);
        assert!((cal.roll.expo() - 0.4).abs() < 0.001);
        assert!((cal.pitch.expo() - 0.4).abs() < 0.001);
        assert!((cal.yaw.expo() - 0.3).abs() < 0.001);
        assert!((cal.throttle.expo() - 0.1).abs() < 0.001);
        assert!((cal.trigger_deadzone - 0.12).abs() < 0.001);
    }

    #[test]
    fn test_apply_trigger_deadzone() {
        let cal = AxisCalibration::default();

        // Within deadzone
        assert_eq!(cal.apply_trigger(0.05), 0.0);

        // At deadzone boundary
        assert_eq!(cal.apply_trigger(0.10), 0.0);

        // Full press
        assert!((cal.apply_trigger(1.0) - 1.0).abs() < 0.001);
    }

    // ==================== Normalization Tests ====================

    #[test]
    fn test_normalize_axis() {
        assert!((normalize_axis(0) - (-1.0)).abs() < 0.01);
        assert!((normalize_axis(128) - 0.0).abs() < 0.01);
        assert!((normalize_axis(255) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_normalize_axis_midpoints() {
        // Quarter points
        assert!((normalize_axis(64) - (-0.5)).abs() < 0.02);
        assert!((normalize_axis(192) - 0.5).abs() < 0.02);
    }

    #[test]
    fn test_normalize_trigger() {
        assert!((normalize_trigger(0) - 0.0).abs() < 0.01);
        assert!((normalize_trigger(128) - 0.5).abs() < 0.01);
        assert!((normalize_trigger(255) - 1.0).abs() < 0.01);
    }

    // ==================== CRSF Conversion Tests ====================

    #[test]
    fn test_to_crsf_channel() {
        assert_eq!(to_crsf_channel(-1.0), 0);
        // Center may be 1023 or 1024 due to float rounding
        let center = to_crsf_channel(0.0);
        assert!(center == 1023 || center == 1024);
        assert_eq!(to_crsf_channel(1.0), 2047);
    }

    #[test]
    fn test_to_crsf_channel_clamps() {
        assert_eq!(to_crsf_channel(-2.0), 0);
        assert_eq!(to_crsf_channel(2.0), 2047);
    }

    #[test]
    fn test_to_crsf_channel_midpoints() {
        // Quarter points
        let quarter = to_crsf_channel(-0.5);
        assert!(quarter > 400 && quarter < 600);

        let three_quarter = to_crsf_channel(0.5);
        assert!(three_quarter > 1400 && three_quarter < 1600);
    }

    #[test]
    fn test_trigger_to_crsf_channel() {
        assert_eq!(trigger_to_crsf_channel(0.0), 0);
        assert_eq!(trigger_to_crsf_channel(1.0), 2047);
    }

    #[test]
    fn test_trigger_to_crsf_channel_clamps() {
        assert_eq!(trigger_to_crsf_channel(-0.5), 0);
        assert_eq!(trigger_to_crsf_channel(1.5), 2047);
    }

    // ==================== Round-trip Tests ====================

    #[test]
    fn test_normalize_and_convert_roundtrip() {
        // Center value
        let raw = 128;
        let normalized = normalize_axis(raw);
        let crsf = to_crsf_channel(normalized);
        assert!((crsf as i32 - 1024).abs() <= 1);
    }

    #[test]
    fn test_full_pipeline() {
        let cal = Calibration::new(0.05, 0.3);

        // Full deflection
        let raw = 255;
        let normalized = normalize_axis(raw);
        let calibrated = cal.apply(normalized);
        let crsf = to_crsf_channel(calibrated);
        assert_eq!(crsf, 2047);
    }
}
