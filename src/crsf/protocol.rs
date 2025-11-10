//! # CRSF Protocol Constants and Types
//!
//! Core protocol definitions for CRSF (Crossfire) communication.

use crate::error::{FpvBridgeError, Result};

/// CRSF frame sync byte (always 0xC8)
pub const CRSF_SYNC_BYTE: u8 = 0xC8;

/// RC Channels packet type
pub const CRSF_FRAMETYPE_RC_CHANNELS_PACKED: u8 = 0x16;

/// Link Statistics packet type
pub const CRSF_FRAMETYPE_LINK_STATISTICS: u8 = 0x14;

/// Maximum CRSF payload size
/// Frame structure: sync(1) + length(1) + type(1) + payload(N) + crc(1)
/// Maximum frame size is 64 bytes, so max payload = 64 - 4 = 60 bytes
pub const CRSF_MAX_PAYLOAD_SIZE: usize = 60;

/// RC channels payload size (22 bytes for 16 channels × 11 bits)
pub const CRSF_RC_CHANNELS_PAYLOAD_SIZE: usize = 22;

/// RC channels frame length (type + payload + crc)
pub const CRSF_RC_CHANNELS_FRAME_LENGTH: u8 = 0x18; // 24 bytes

/// Number of RC channels
pub const CRSF_NUM_CHANNELS: usize = 16;

/// Channel value range (11-bit: 0-2047)
pub const CRSF_CHANNEL_VALUE_MIN: u16 = 0;
pub const CRSF_CHANNEL_VALUE_MAX: u16 = 2047;
pub const CRSF_CHANNEL_VALUE_CENTER: u16 = 1024;

/// Link Statistics payload size
pub const CRSF_LINK_STATS_PAYLOAD_SIZE: usize = 10;

/// Battery Sensor payload size
pub const CRSF_BATTERY_SENSOR_PAYLOAD_SIZE: usize = 8;

/// GPS payload size
pub const CRSF_GPS_PAYLOAD_SIZE: usize = 15;

/// RC channels array type (16 channels, 11-bit values)
pub type RcChannels = [u16; CRSF_NUM_CHANNELS];

/// Link statistics telemetry data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinkStatistics {
    /// Uplink RSSI (antenna 1) in -dBm
    pub uplink_rssi_1: u8,

    /// Uplink RSSI (antenna 2) in -dBm (diversity)
    pub uplink_rssi_2: u8,

    /// Uplink link quality (0-100%)
    pub uplink_lq: u8,

    /// Uplink SNR in dB
    pub uplink_snr: i8,

    /// Active antenna (0 or 1)
    pub active_antenna: u8,

    /// RF mode / packet rate
    pub rf_mode: u8,

    /// Uplink TX power in mW (encoded)
    pub uplink_tx_power: u8,

    /// Downlink RSSI in -dBm
    pub downlink_rssi: u8,

    /// Downlink link quality (0-100%)
    pub downlink_lq: u8,

    /// Downlink SNR in dB
    pub downlink_snr: i8,
}

/// Battery sensor telemetry data
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BatterySensor {
    /// Battery voltage in volts
    pub voltage: f32,

    /// Current draw in amperes
    pub current: f32,

    /// Capacity used in mAh
    pub capacity_used: u32,

    /// Battery remaining percentage (0-100%)
    pub remaining_percent: u8,
}

/// GPS telemetry data
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GpsData {
    /// Latitude in degrees
    pub latitude: f64,

    /// Longitude in degrees
    pub longitude: f64,

    /// Ground speed in km/h
    pub ground_speed: f32,

    /// Heading in degrees
    pub heading: f32,

    /// Altitude in meters
    pub altitude: i16,

    /// Number of satellites
    pub satellites: u8,
}

/// CRSF frame structure
#[derive(Debug, Clone)]
pub struct CrsfFrame {
    /// Frame type
    pub frame_type: u8,

    /// Payload data
    pub payload: Vec<u8>,
}

impl CrsfFrame {
    /// Create a new CRSF frame
    ///
    /// # Arguments
    ///
    /// * `frame_type` - Frame type byte
    /// * `payload` - Payload data (max 60 bytes)
    ///
    /// # Returns
    ///
    /// * `Result<CrsfFrame>` - Frame if valid, or error if payload too large
    ///
    /// # Errors
    ///
    /// Returns error if payload exceeds CRSF_MAX_PAYLOAD_SIZE (60 bytes)
    pub fn new(frame_type: u8, payload: Vec<u8>) -> Result<Self> {
        if payload.len() > CRSF_MAX_PAYLOAD_SIZE {
            return Err(FpvBridgeError::CrsfProtocol(
                format!("Payload size {} exceeds maximum {}", payload.len(), CRSF_MAX_PAYLOAD_SIZE)
            ));
        }

        Ok(Self {
            frame_type,
            payload,
        })
    }

    /// Get frame length (type + payload + crc)
    ///
    /// This is guaranteed not to overflow since payload is validated to be ≤ 60 bytes
    pub fn length(&self) -> u8 {
        (1 + self.payload.len() + 1) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_value_ranges() {
        assert_eq!(CRSF_CHANNEL_VALUE_MIN, 0);
        assert_eq!(CRSF_CHANNEL_VALUE_MAX, 2047);
        assert_eq!(CRSF_CHANNEL_VALUE_CENTER, 1024);
    }

    #[test]
    fn test_frame_constants() {
        assert_eq!(CRSF_SYNC_BYTE, 0xC8);
        assert_eq!(CRSF_FRAMETYPE_RC_CHANNELS_PACKED, 0x16);
        assert_eq!(CRSF_FRAMETYPE_LINK_STATISTICS, 0x14);
        assert_eq!(CRSF_NUM_CHANNELS, 16);
    }

    #[test]
    fn test_crsf_frame() {
        let frame = CrsfFrame::new(CRSF_FRAMETYPE_RC_CHANNELS_PACKED, vec![0u8; 22]).unwrap();
        assert_eq!(frame.frame_type, 0x16);
        assert_eq!(frame.payload.len(), 22);
        assert_eq!(frame.length(), 24); // 1 (type) + 22 (payload) + 1 (crc)
    }

    #[test]
    fn test_crsf_frame_payload_too_large() {
        // Payload of 61 bytes should fail (max is 60)
        let result = CrsfFrame::new(CRSF_FRAMETYPE_RC_CHANNELS_PACKED, vec![0u8; 61]);
        assert!(result.is_err());
    }

    #[test]
    fn test_crsf_frame_max_payload() {
        // Payload of exactly 60 bytes should succeed
        let frame = CrsfFrame::new(CRSF_FRAMETYPE_RC_CHANNELS_PACKED, vec![0u8; 60]).unwrap();
        assert_eq!(frame.payload.len(), 60);
        assert_eq!(frame.length(), 62); // 1 (type) + 60 (payload) + 1 (crc)
    }
}
