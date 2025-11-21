//! # Controller Module
//!
//! PS5 DualSense controller input handling.
//!
//! This module handles:
//! - PS5 controller detection and connection via evdev
//! - Reading analog stick and button inputs
//! - Applying deadzones and exponential curves
//! - Mapping inputs to RC channels
//! - Calibration and safety checks

pub mod channel_mapper;
pub mod mapper;
pub mod ps5;
