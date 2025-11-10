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

// Module exports will be added as we implement submodules
// pub mod logger;
// pub mod types;
