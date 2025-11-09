//! # Configuration Module
//!
//! Handles loading and validating configuration from TOML files.

use serde::Deserialize;
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
        let mut config = Config {
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
}
