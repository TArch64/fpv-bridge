//! # CRSF Protocol Module
//!
//! Implementation of the Crossfire (CRSF) protocol for ExpressLRS communication.
//!
//! This module handles:
//! - RC channels packet encoding (16 channels, 11-bit resolution)
//! - CRC8-DVB-S2 checksum calculation

pub mod protocol;
pub mod encoder;
pub mod crc;

// Decoder available only for tests until telemetry is implemented
#[cfg(test)]
pub mod decoder;
