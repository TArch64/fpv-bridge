//! # FPV Bridge Library
//!
//! Control your FPV drone with a PS5 DualSense controller via ExpressLRS.
//!
//! This library provides the core functionality for bridging PS5 controller inputs
//! to CRSF (Crossfire) protocol for controlling ExpressLRS-enabled drones.

pub mod config;
pub mod error;
pub mod crsf;
pub mod controller;
pub mod serial;
pub mod telemetry;
