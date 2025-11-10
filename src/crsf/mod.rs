//! # CRSF Protocol Module
//!
//! Implementation of the Crossfire (CRSF) protocol for ExpressLRS communication.
//!
//! This module handles:
//! - RC channels packet encoding (16 channels, 11-bit resolution)
//! - Telemetry packet decoding (Link Stats, Battery, GPS, etc.)
//! - CRC8-DVB-S2 checksum calculation
//! - Frame synchronization and validation

pub mod protocol;
pub mod encoder;
pub mod decoder;
pub mod crc;

// Re-export commonly used types and functions
pub use protocol::{
    RcChannels,
    CrsfFrame,
    LinkStatistics,
    BatterySensor,
    GpsData,
    CRSF_SYNC_BYTE,
    CRSF_FRAMETYPE_RC_CHANNELS_PACKED,
    CRSF_FRAMETYPE_LINK_STATISTICS,
    CRSF_FRAMETYPE_BATTERY_SENSOR,
    CRSF_FRAMETYPE_GPS,
    CRSF_NUM_CHANNELS,
    CRSF_CHANNEL_VALUE_MIN,
    CRSF_CHANNEL_VALUE_MAX,
    CRSF_CHANNEL_VALUE_CENTER,
};

pub use encoder::{
    encode_rc_channels_frame,
    encode_rc_channels_payload,
    clamp_channel_value,
};

pub use decoder::{
    decode_frame,
    decode_link_statistics,
    decode_battery_sensor,
    decode_gps,
};

pub use crc::crc8_dvb_s2;
