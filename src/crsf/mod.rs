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
