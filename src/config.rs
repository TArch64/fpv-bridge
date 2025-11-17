//! # Configuration Module
//!
//! Handles loading and validating configuration from TOML files.

use serde::Deserialize;
use serde::de::Error;
use std::fs;
use std::path::Path;

use crate::error::Result;

/// Main configuration structure
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub serial: SerialConfig,
    pub controller: ControllerConfig,
    pub channels: ChannelConfig,
    pub telemetry: TelemetryConfig,
    pub safety: SafetyConfig,
    pub crsf: CrsfConfig,
}

/// Serial port configuration
#[derive(Debug, Deserialize, Clone)]
pub struct SerialConfig {
    #[serde(default = "default_serial_port")]
    pub port: String,

    #[serde(default = "default_baud_rate")]
    pub baud_rate: u32,

    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,

    #[serde(default = "default_reconnect_interval_ms")]
    pub reconnect_interval_ms: u64,
}

/// Controller configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ControllerConfig {
    #[serde(default)]
    pub device_path: String,

    #[serde(default = "default_deadzone_stick")]
    pub deadzone_stick: f32,

    #[serde(default = "default_deadzone_trigger")]
    pub deadzone_trigger: f32,

    #[serde(default = "default_expo_roll")]
    pub expo_roll: f32,

    #[serde(default = "default_expo_pitch")]
    pub expo_pitch: f32,

    #[serde(default = "default_expo_yaw")]
    pub expo_yaw: f32,

    #[serde(default = "default_expo_throttle")]
    pub expo_throttle: f32,
}

/// Channel configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ChannelConfig {
    #[serde(default = "default_throttle_min")]
    pub throttle_min: u16,

    #[serde(default = "default_throttle_max")]
    pub throttle_max: u16,

    #[serde(default = "default_center")]
    pub center: u16,

    #[serde(default)]
    pub channel_reverse: Vec<usize>,
}

/// Telemetry configuration
#[derive(Debug, Deserialize, Clone)]
pub struct TelemetryConfig {
    #[serde(default = "default_telemetry_enabled")]
    pub enabled: bool,

    #[serde(default = "default_log_dir")]
    pub log_dir: String,

    #[serde(default = "default_max_records_per_file")]
    pub max_records_per_file: usize,

    #[serde(default = "default_max_files_to_keep")]
    pub max_files_to_keep: usize,

    #[serde(default = "default_log_interval_ms")]
    pub log_interval_ms: u64,

    #[serde(default = "default_log_format")]
    pub format: String,
}

/// Safety configuration
#[derive(Debug, Deserialize, Clone)]
pub struct SafetyConfig {
    #[serde(default = "default_arm_button_hold_ms")]
    pub arm_button_hold_ms: u64,

    #[serde(default = "default_auto_disarm_timeout_s")]
    pub auto_disarm_timeout_s: u64,

    #[serde(default = "default_failsafe_timeout_ms")]
    pub failsafe_timeout_ms: u64,

    #[serde(default = "default_min_throttle_to_arm")]
    pub min_throttle_to_arm: u16,
}

/// CRSF protocol configuration
#[derive(Debug, Deserialize, Clone)]
pub struct CrsfConfig {
    #[serde(default = "default_packet_rate_hz")]
    pub packet_rate_hz: u32,

    #[serde(default = "default_link_stats_interval_ms")]
    pub link_stats_interval_ms: u64,
}

// Default value functions
fn default_serial_port() -> String { "/dev/ttyACM0".to_string() }
fn default_baud_rate() -> u32 { 420000 }
fn default_timeout_ms() -> u64 { 100 }
fn default_reconnect_interval_ms() -> u64 { 1000 }

fn default_deadzone_stick() -> f32 { 0.05 }
fn default_deadzone_trigger() -> f32 { 0.10 }
fn default_expo_roll() -> f32 { 0.3 }
fn default_expo_pitch() -> f32 { 0.3 }
fn default_expo_yaw() -> f32 { 0.2 }
fn default_expo_throttle() -> f32 { 0.0 }

fn default_throttle_min() -> u16 { 1000 }
fn default_throttle_max() -> u16 { 2000 }
fn default_center() -> u16 { 1500 }

fn default_telemetry_enabled() -> bool { true }
fn default_log_dir() -> String { "./logs".to_string() }
fn default_max_records_per_file() -> usize { 10000 }
fn default_max_files_to_keep() -> usize { 10 }
fn default_log_interval_ms() -> u64 { 100 }
fn default_log_format() -> String { "jsonl".to_string() }

fn default_arm_button_hold_ms() -> u64 { 1000 }
fn default_auto_disarm_timeout_s() -> u64 { 300 }
fn default_failsafe_timeout_ms() -> u64 { 500 }
fn default_min_throttle_to_arm() -> u16 { 1050 }

fn default_packet_rate_hz() -> u32 { 250 }
fn default_link_stats_interval_ms() -> u64 { 1000 }

impl Config {
    /// Load configuration from a TOML file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    ///
    /// * `Result<Config>` - Loaded and validated configuration
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - File cannot be read
    /// - TOML parsing fails
    /// - Validation fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fpv_bridge::config::Config;
    ///
    /// let config = Config::load("config/default.toml")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }

    /// Validate configuration values
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if valid, Err if invalid
    ///
    /// # Errors
    ///
    /// Returns error if any configuration value is out of valid range
    fn validate(&self) -> Result<()> {
        // Validate serial port configuration
        if self.serial.port.is_empty() {
            return Err(crate::error::FpvBridgeError::Config(
                toml::de::Error::custom("serial port cannot be empty")
            ));
        }

        // Validate telemetry configuration
        if self.telemetry.enabled && self.telemetry.log_dir.is_empty() {
            return Err(crate::error::FpvBridgeError::Config(
                toml::de::Error::custom("telemetry log_dir cannot be empty when enabled")
            ));
        }

        // Controller device_path can be empty (auto-detect)
        // Just accessing it to suppress dead code warning
        let _ = &self.controller.device_path;

        // Validate timing fields
        if self.serial.timeout_ms == 0 || self.serial.timeout_ms > 10000 {
            return Err(crate::error::FpvBridgeError::Config(
                toml::de::Error::custom("timeout_ms must be between 1 and 10000")
            ));
        }

        if self.serial.reconnect_interval_ms == 0 || self.serial.reconnect_interval_ms > 60000 {
            return Err(crate::error::FpvBridgeError::Config(
                toml::de::Error::custom("reconnect_interval_ms must be between 1 and 60000")
            ));
        }

        if self.telemetry.log_interval_ms == 0 || self.telemetry.log_interval_ms > 60000 {
            return Err(crate::error::FpvBridgeError::Config(
                toml::de::Error::custom("log_interval_ms must be between 1 and 60000")
            ));
        }

        if self.safety.failsafe_timeout_ms == 0 || self.safety.failsafe_timeout_ms > 60000 {
            return Err(crate::error::FpvBridgeError::Config(
                toml::de::Error::custom("failsafe_timeout_ms must be between 1 and 60000")
            ));
        }

        if self.safety.arm_button_hold_ms == 0 || self.safety.arm_button_hold_ms > 10000 {
            return Err(crate::error::FpvBridgeError::Config(
                toml::de::Error::custom("arm_button_hold_ms must be between 1 and 10000")
            ));
        }

        if self.safety.auto_disarm_timeout_s == 0 {
            return Err(crate::error::FpvBridgeError::Config(
                toml::de::Error::custom("auto_disarm_timeout_s must be greater than 0")
            ));
        }

        if self.crsf.link_stats_interval_ms == 0 || self.crsf.link_stats_interval_ms > 60000 {
            return Err(crate::error::FpvBridgeError::Config(
                toml::de::Error::custom("link_stats_interval_ms must be between 1 and 60000")
            ));
        }

        // Validate telemetry file limits
        if self.telemetry.max_records_per_file == 0 {
            return Err(crate::error::FpvBridgeError::Config(
                toml::de::Error::custom("max_records_per_file must be greater than 0")
            ));
        }

        if self.telemetry.max_files_to_keep == 0 {
            return Err(crate::error::FpvBridgeError::Config(
                toml::de::Error::custom("max_files_to_keep must be greater than 0")
            ));
        }

        // Validate deadzones
        if self.controller.deadzone_stick < 0.0 || self.controller.deadzone_stick > 0.25 {
            return Err(crate::error::FpvBridgeError::Config(
                toml::de::Error::custom("deadzone_stick must be between 0.0 and 0.25")
            ));
        }

        if self.controller.deadzone_trigger < 0.0 || self.controller.deadzone_trigger > 0.25 {
            return Err(crate::error::FpvBridgeError::Config(
                toml::de::Error::custom("deadzone_trigger must be between 0.0 and 0.25")
            ));
        }

        // Validate expo curves
        for (name, value) in [
            ("expo_roll", self.controller.expo_roll),
            ("expo_pitch", self.controller.expo_pitch),
            ("expo_yaw", self.controller.expo_yaw),
            ("expo_throttle", self.controller.expo_throttle),
        ] {
            if value < 0.0 || value > 1.0 {
                return Err(crate::error::FpvBridgeError::Config(
                    toml::de::Error::custom(format!("{} must be between 0.0 and 1.0", name))
                ));
            }
        }

        // Validate channel values
        if self.channels.throttle_min < 988 || self.channels.throttle_min > 1500 {
            return Err(crate::error::FpvBridgeError::Config(
                toml::de::Error::custom("throttle_min must be between 988 and 1500")
            ));
        }

        if self.channels.throttle_max < 1500 || self.channels.throttle_max > 2012 {
            return Err(crate::error::FpvBridgeError::Config(
                toml::de::Error::custom("throttle_max must be between 1500 and 2012")
            ));
        }

        if self.channels.throttle_min >= self.channels.throttle_max {
            return Err(crate::error::FpvBridgeError::Config(
                toml::de::Error::custom("throttle_min must be less than throttle_max")
            ));
        }

        if self.channels.center < self.channels.throttle_min
            || self.channels.center > self.channels.throttle_max {
            return Err(crate::error::FpvBridgeError::Config(
                toml::de::Error::custom("center must be within throttle range (throttle_min to throttle_max)")
            ));
        }

        // Validate channel_reverse indices (CRSF has 16 channels: 0-15)
        for &channel_idx in &self.channels.channel_reverse {
            if channel_idx > 15 {
                return Err(crate::error::FpvBridgeError::Config(
                    toml::de::Error::custom(format!("channel_reverse index {} is out of bounds (must be 0-15)", channel_idx))
                ));
            }
        }

        // Validate min_throttle_to_arm is within throttle range
        if self.safety.min_throttle_to_arm < self.channels.throttle_min
            || self.safety.min_throttle_to_arm > self.channels.throttle_max {
            return Err(crate::error::FpvBridgeError::Config(
                toml::de::Error::custom("min_throttle_to_arm must be within throttle range (throttle_min to throttle_max)")
            ));
        }

        // Validate baud rate
        if ![115200, 400000, 420000, 921600, 1870000, 3750000].contains(&self.serial.baud_rate) {
            return Err(crate::error::FpvBridgeError::Config(
                toml::de::Error::custom("baud_rate must be one of: 115200, 400000, 420000, 921600, 1870000, 3750000")
            ));
        }

        // Validate log format
        if self.telemetry.format != "jsonl" {
            return Err(crate::error::FpvBridgeError::Config(
                toml::de::Error::custom("log format must be 'jsonl' (only supported format)")
            ));
        }

        // Validate packet rate
        if ![50, 150, 250, 500].contains(&self.crsf.packet_rate_hz) {
            return Err(crate::error::FpvBridgeError::Config(
                toml::de::Error::custom("packet_rate_hz must be one of: 50, 150, 250, 500")
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config {
            serial: SerialConfig {
                port: default_serial_port(),
                baud_rate: default_baud_rate(),
                timeout_ms: default_timeout_ms(),
                reconnect_interval_ms: default_reconnect_interval_ms(),
            },
            controller: ControllerConfig {
                device_path: String::new(),
                deadzone_stick: default_deadzone_stick(),
                deadzone_trigger: default_deadzone_trigger(),
                expo_roll: default_expo_roll(),
                expo_pitch: default_expo_pitch(),
                expo_yaw: default_expo_yaw(),
                expo_throttle: default_expo_throttle(),
            },
            channels: ChannelConfig {
                throttle_min: default_throttle_min(),
                throttle_max: default_throttle_max(),
                center: default_center(),
                channel_reverse: vec![],
            },
            telemetry: TelemetryConfig {
                enabled: default_telemetry_enabled(),
                log_dir: default_log_dir(),
                max_records_per_file: default_max_records_per_file(),
                max_files_to_keep: default_max_files_to_keep(),
                log_interval_ms: default_log_interval_ms(),
                format: default_log_format(),
            },
            safety: SafetyConfig {
                arm_button_hold_ms: default_arm_button_hold_ms(),
                auto_disarm_timeout_s: default_auto_disarm_timeout_s(),
                failsafe_timeout_ms: default_failsafe_timeout_ms(),
                min_throttle_to_arm: default_min_throttle_to_arm(),
            },
            crsf: CrsfConfig {
                packet_rate_hz: default_packet_rate_hz(),
                link_stats_interval_ms: default_link_stats_interval_ms(),
            },
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_deadzone() {
        let config = Config {
            serial: SerialConfig {
                port: default_serial_port(),
                baud_rate: default_baud_rate(),
                timeout_ms: default_timeout_ms(),
                reconnect_interval_ms: default_reconnect_interval_ms(),
            },
            controller: ControllerConfig {
                device_path: String::new(),
                deadzone_stick: 0.5,  // Invalid: > 0.25
                deadzone_trigger: default_deadzone_trigger(),
                expo_roll: default_expo_roll(),
                expo_pitch: default_expo_pitch(),
                expo_yaw: default_expo_yaw(),
                expo_throttle: default_expo_throttle(),
            },
            channels: ChannelConfig {
                throttle_min: default_throttle_min(),
                throttle_max: default_throttle_max(),
                center: default_center(),
                channel_reverse: vec![],
            },
            telemetry: TelemetryConfig {
                enabled: default_telemetry_enabled(),
                log_dir: default_log_dir(),
                max_records_per_file: default_max_records_per_file(),
                max_files_to_keep: default_max_files_to_keep(),
                log_interval_ms: default_log_interval_ms(),
                format: default_log_format(),
            },
            safety: SafetyConfig {
                arm_button_hold_ms: default_arm_button_hold_ms(),
                auto_disarm_timeout_s: default_auto_disarm_timeout_s(),
                failsafe_timeout_ms: default_failsafe_timeout_ms(),
                min_throttle_to_arm: default_min_throttle_to_arm(),
            },
            crsf: CrsfConfig {
                packet_rate_hz: default_packet_rate_hz(),
                link_stats_interval_ms: default_link_stats_interval_ms(),
            },
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_load_config_from_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let toml_content = r#"
[serial]
port = "/dev/ttyUSB0"

[controller]

[channels]

[telemetry]

[safety]

[crsf]
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = Config::load(temp_file.path());
        assert!(result.is_ok());
    }

    fn create_valid_config() -> Config {
        Config {
            serial: SerialConfig {
                port: default_serial_port(),
                baud_rate: default_baud_rate(),
                timeout_ms: default_timeout_ms(),
                reconnect_interval_ms: default_reconnect_interval_ms(),
            },
            controller: ControllerConfig {
                device_path: String::new(),
                deadzone_stick: default_deadzone_stick(),
                deadzone_trigger: default_deadzone_trigger(),
                expo_roll: default_expo_roll(),
                expo_pitch: default_expo_pitch(),
                expo_yaw: default_expo_yaw(),
                expo_throttle: default_expo_throttle(),
            },
            channels: ChannelConfig {
                throttle_min: default_throttle_min(),
                throttle_max: default_throttle_max(),
                center: default_center(),
                channel_reverse: vec![],
            },
            telemetry: TelemetryConfig {
                enabled: default_telemetry_enabled(),
                log_dir: default_log_dir(),
                max_records_per_file: default_max_records_per_file(),
                max_files_to_keep: default_max_files_to_keep(),
                log_interval_ms: default_log_interval_ms(),
                format: default_log_format(),
            },
            safety: SafetyConfig {
                arm_button_hold_ms: default_arm_button_hold_ms(),
                auto_disarm_timeout_s: default_auto_disarm_timeout_s(),
                failsafe_timeout_ms: default_failsafe_timeout_ms(),
                min_throttle_to_arm: default_min_throttle_to_arm(),
            },
            crsf: CrsfConfig {
                packet_rate_hz: default_packet_rate_hz(),
                link_stats_interval_ms: default_link_stats_interval_ms(),
            },
        }
    }

    #[test]
    fn test_empty_serial_port() {
        let mut config = create_valid_config();
        config.serial.port = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_empty_log_dir_when_enabled() {
        let mut config = create_valid_config();
        config.telemetry.enabled = true;
        config.telemetry.log_dir = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_empty_log_dir_when_disabled() {
        let mut config = create_valid_config();
        config.telemetry.enabled = false;
        config.telemetry.log_dir = String::new();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_timeout_ms_zero() {
        let mut config = create_valid_config();
        config.serial.timeout_ms = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_timeout_ms_too_high() {
        let mut config = create_valid_config();
        config.serial.timeout_ms = 10001;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_reconnect_interval_zero() {
        let mut config = create_valid_config();
        config.serial.reconnect_interval_ms = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_reconnect_interval_too_high() {
        let mut config = create_valid_config();
        config.serial.reconnect_interval_ms = 60001;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_log_interval_zero() {
        let mut config = create_valid_config();
        config.telemetry.log_interval_ms = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_log_interval_too_high() {
        let mut config = create_valid_config();
        config.telemetry.log_interval_ms = 60001;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_failsafe_timeout_zero() {
        let mut config = create_valid_config();
        config.safety.failsafe_timeout_ms = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_failsafe_timeout_too_high() {
        let mut config = create_valid_config();
        config.safety.failsafe_timeout_ms = 60001;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_arm_button_hold_zero() {
        let mut config = create_valid_config();
        config.safety.arm_button_hold_ms = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_arm_button_hold_too_high() {
        let mut config = create_valid_config();
        config.safety.arm_button_hold_ms = 10001;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_auto_disarm_timeout_zero() {
        let mut config = create_valid_config();
        config.safety.auto_disarm_timeout_s = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_link_stats_interval_zero() {
        let mut config = create_valid_config();
        config.crsf.link_stats_interval_ms = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_link_stats_interval_too_high() {
        let mut config = create_valid_config();
        config.crsf.link_stats_interval_ms = 60001;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_max_records_per_file_zero() {
        let mut config = create_valid_config();
        config.telemetry.max_records_per_file = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_max_files_to_keep_zero() {
        let mut config = create_valid_config();
        config.telemetry.max_files_to_keep = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_deadzone_stick_negative() {
        let mut config = create_valid_config();
        config.controller.deadzone_stick = -0.1;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_deadzone_trigger_too_high() {
        let mut config = create_valid_config();
        config.controller.deadzone_trigger = 0.3;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_expo_roll_negative() {
        let mut config = create_valid_config();
        config.controller.expo_roll = -0.1;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_expo_pitch_too_high() {
        let mut config = create_valid_config();
        config.controller.expo_pitch = 1.1;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_expo_yaw_invalid() {
        let mut config = create_valid_config();
        config.controller.expo_yaw = 2.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_expo_throttle_negative() {
        let mut config = create_valid_config();
        config.controller.expo_throttle = -0.5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_throttle_min_too_low() {
        let mut config = create_valid_config();
        config.channels.throttle_min = 987;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_throttle_min_too_high() {
        let mut config = create_valid_config();
        config.channels.throttle_min = 1501;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_throttle_max_too_low() {
        let mut config = create_valid_config();
        config.channels.throttle_max = 1499;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_throttle_max_too_high() {
        let mut config = create_valid_config();
        config.channels.throttle_max = 2013;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_throttle_min_equals_max() {
        let mut config = create_valid_config();
        config.channels.throttle_min = 1500;
        config.channels.throttle_max = 1500;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_throttle_min_greater_than_max() {
        let mut config = create_valid_config();
        config.channels.throttle_min = 1800;
        config.channels.throttle_max = 1200;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_center_below_min() {
        let mut config = create_valid_config();
        config.channels.center = 900;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_center_above_max() {
        let mut config = create_valid_config();
        config.channels.center = 2100;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_channel_reverse_invalid_index() {
        let mut config = create_valid_config();
        config.channels.channel_reverse = vec![0, 5, 16]; // 16 is invalid
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_channel_reverse_valid_indices() {
        let mut config = create_valid_config();
        config.channels.channel_reverse = vec![0, 5, 10, 15]; // All valid (0-15)
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_min_throttle_to_arm_below_range() {
        let mut config = create_valid_config();
        config.safety.min_throttle_to_arm = 900;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_min_throttle_to_arm_above_range() {
        let mut config = create_valid_config();
        config.safety.min_throttle_to_arm = 2100;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_baud_rate() {
        let mut config = create_valid_config();
        config.serial.baud_rate = 9600; // Not in the allowed list
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_valid_baud_rates() {
        for &baud in &[115200, 400000, 420000, 921600, 1870000, 3750000] {
            let mut config = create_valid_config();
            config.serial.baud_rate = baud;
            assert!(config.validate().is_ok(), "Baud rate {} should be valid", baud);
        }
    }

    #[test]
    fn test_invalid_log_format() {
        let mut config = create_valid_config();
        config.telemetry.format = "csv".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_packet_rate() {
        let mut config = create_valid_config();
        config.crsf.packet_rate_hz = 100; // Not in the allowed list
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_valid_packet_rates() {
        for &rate in &[50, 150, 250, 500] {
            let mut config = create_valid_config();
            config.crsf.packet_rate_hz = rate;
            assert!(config.validate().is_ok(), "Packet rate {} should be valid", rate);
        }
    }

    #[test]
    fn test_default_functions() {
        assert_eq!(default_serial_port(), "/dev/ttyACM0");
        assert_eq!(default_baud_rate(), 420000);
        assert_eq!(default_timeout_ms(), 100);
        assert_eq!(default_reconnect_interval_ms(), 1000);
        assert_eq!(default_deadzone_stick(), 0.05);
        assert_eq!(default_deadzone_trigger(), 0.10);
        assert_eq!(default_expo_roll(), 0.3);
        assert_eq!(default_expo_pitch(), 0.3);
        assert_eq!(default_expo_yaw(), 0.2);
        assert_eq!(default_expo_throttle(), 0.0);
        assert_eq!(default_throttle_min(), 1000);
        assert_eq!(default_throttle_max(), 2000);
        assert_eq!(default_center(), 1500);
        assert_eq!(default_telemetry_enabled(), true);
        assert_eq!(default_log_dir(), "./logs");
        assert_eq!(default_max_records_per_file(), 10000);
        assert_eq!(default_max_files_to_keep(), 10);
        assert_eq!(default_log_interval_ms(), 100);
        assert_eq!(default_log_format(), "jsonl");
        assert_eq!(default_arm_button_hold_ms(), 1000);
        assert_eq!(default_auto_disarm_timeout_s(), 300);
        assert_eq!(default_failsafe_timeout_ms(), 500);
        assert_eq!(default_min_throttle_to_arm(), 1050);
        assert_eq!(default_packet_rate_hz(), 250);
        assert_eq!(default_link_stats_interval_ms(), 1000);
    }
}
