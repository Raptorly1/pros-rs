//! Devices
//!
//! This module provides functionality for accessing hardware and devices connected to the V5 brain.
//!
//! # Overview
//!
//! The V5 brain features 21 RJ11 4p4c connector ports (known as "Smart Ports") for communicating with
//! newer V5 peripherals, as well as six 3-wire ports with analog-to-digital conversion capability for
//! compatibility with legacy cortex devices. This module provides access for both smart devices and
//! ADI devices.
//!
//! # Organization
//!
//! - [`devices::smart`] contains abstractions and types for smart port connected devices.
//! - [`devices::adi`] contains abstractions for three wire ADI connected devices.
//! - [`devices::battery`] provides functions for getting information about the currently connected
//!   battery.
//! - [`devices::controller`] provides types for interacting with the V5 controller.

pub mod adi;
pub mod smart;

pub mod battery;
pub mod controller;
pub mod peripherals;
pub mod position;

pub use controller::Controller;
pub use position::Position;
