//! # CRSF Protocol Module
//!
//! Implementation of the Crossfire (CRSF) protocol for ExpressLRS communication.
//!
//! This module handles:
//! - RC channels packet encoding (16 channels, 11-bit resolution)
//! - Telemetry packet decoding (Link Stats, Battery, GPS, etc.)
//! - CRC8-DVB-S2 checksum calculation
//! - Frame synchronization and validation

// Module exports will be added as we implement submodules
// pub mod protocol;
// pub mod encoder;
// pub mod decoder;
// pub mod crc;

// Placeholder types
/// RC channels array (16 channels)
pub type RcChannels = [u16; 16];

/// CRSF frame sync byte
pub const CRSF_SYNC_BYTE: u8 = 0xC8;

/// RC Channels packet type
pub const CRSF_FRAMETYPE_RC_CHANNELS_PACKED: u8 = 0x16;

/// Link Statistics packet type
pub const CRSF_FRAMETYPE_LINK_STATISTICS: u8 = 0x14;
