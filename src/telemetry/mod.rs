//! # Telemetry Module
//!
//! Handles telemetry logging to JSONL files with rotation.
//!
//! This module handles:
//! - Receiving telemetry data from ELRS
//! - Formatting as JSONL (JSON Lines)
//! - Writing to rotating log files
//! - Managing file rotation (max N records per file)
//! - Retaining only last M files

use serde::Serialize;

// Module exports will be added as we implement submodules
// pub mod logger;
// pub mod types;

/// Telemetry data entry
#[derive(Debug, Clone, Serialize)]
pub struct TelemetryEntry {
    /// Timestamp in ISO 8601 format
    pub timestamp: String,

    /// Battery voltage (volts)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub battery_voltage: Option<f32>,

    /// Current draw (amps)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<f32>,

    /// Capacity used (mAh)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity_used: Option<u32>,

    /// Battery remaining (%)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub battery_remaining: Option<u8>,

    /// RSSI (dBm)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rssi: Option<i8>,

    /// Link quality (%)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_quality: Option<u8>,

    /// SNR (dB)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snr: Option<i8>,

    /// Armed status
    pub armed: bool,

    /// Flight mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flight_mode: Option<String>,

    /// RC channels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<Vec<u16>>,

    /// GPS data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gps: Option<GpsData>,
}

/// GPS telemetry data
#[derive(Debug, Clone, Serialize)]
pub struct GpsData {
    /// Latitude (degrees)
    pub lat: f64,

    /// Longitude (degrees)
    pub lon: f64,

    /// Altitude (meters)
    pub alt: i32,

    /// Ground speed (km/h)
    pub speed: f32,

    /// Heading (degrees)
    pub heading: u16,

    /// Number of satellites
    pub sats: u8,
}
