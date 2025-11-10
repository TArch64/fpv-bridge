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

mod ps5;
mod mapper;

pub use ps5::{Ps5Controller, ControllerState};
pub use mapper::ChannelMapper;

use crate::config::{ControllerConfig, ChannelConfig};
use crate::error::Result;

/// Controller handler that manages PS5 input and channel mapping
pub struct ControllerHandler {
    controller: Ps5Controller,
    mapper: ChannelMapper,
}

impl ControllerHandler {
    /// Create a new controller handler
    ///
    /// # Arguments
    ///
    /// * `controller_config` - Controller configuration
    /// * `channel_config` - Channel mapping configuration
    ///
    /// # Returns
    ///
    /// * `Result<ControllerHandler>` - Handler if successful
    ///
    /// # Errors
    ///
    /// Returns error if controller cannot be found or opened
    pub fn new(
        controller_config: &ControllerConfig,
        channel_config: &ChannelConfig,
    ) -> Result<Self> {
        let controller = Ps5Controller::new(&controller_config.device_path)?;
        let mapper = ChannelMapper::new(controller_config, channel_config);

        Ok(Self { controller, mapper })
    }

    /// Read controller state and map to RC channels
    ///
    /// # Returns
    ///
    /// * `Result<[u16; 16]>` - 16 RC channel values (0-2047)
    ///
    /// # Errors
    ///
    /// Returns error if controller read fails
    pub fn read_channels(&mut self) -> Result<[u16; 16]> {
        let state = self.controller.read_state()?;
        Ok(self.mapper.map_to_channels(&state))
    }

    /// Get the current controller state without mapping
    ///
    /// # Returns
    ///
    /// * `Result<ControllerState>` - Current controller state
    ///
    /// # Errors
    ///
    /// Returns error if controller read fails
    pub fn read_state(&mut self) -> Result<ControllerState> {
        self.controller.read_state()
    }
}
