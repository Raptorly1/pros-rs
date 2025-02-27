//! GPS sensor device.
//!
//! A notable differenc between this API and that of PROS
//! is that [`GpsSensor::status`] returns acceleration along with other status data.

use pros_sys::{PROS_ERR, PROS_ERR_F};
use snafu::Snafu;

use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::error::{bail_on, map_errno, PortError};

/// Represents the data output from a GPS sensor.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct GpsStatus {
    pub x: f64,
    pub y: f64,
    pub pitch: f64,
    pub roll: f64,
    pub yaw: f64,
    pub heading: f64,

    pub accel_x: f64,
    pub accel_y: f64,
    pub accel_z: f64,
}

/// A physical GPS sensor plugged into a port.
#[derive(Debug, Eq, PartialEq)]
pub struct GpsSensor {
    port: SmartPort,
}

impl GpsSensor {
    /// Creates a new GPS sensor on the given port.
    pub fn new(port: SmartPort) -> Result<Self, GpsError> {
        unsafe {
            bail_on!(
                PROS_ERR,
                pros_sys::gps_initialize_full(port.index(), 0.0, 0.0, 0.0, 0.0, 0.0)
            );
        }

        Ok(Self { port })
    }

    /// Sets the offset of the GPS sensor, relative to the sensor of turning, in meters.
    pub fn set_offset(&mut self, x: f64, y: f64) -> Result<(), GpsError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::gps_set_offset(self.port.index(), x, y));
        }
        Ok(())
    }

    /// Gets the possible error of the GPS sensor, in meters.
    pub fn rms_error(&self) -> Result<f64, GpsError> {
        Ok(unsafe { bail_on!(PROS_ERR_F, pros_sys::gps_get_error(self.port.index())) })
    }

    /// Gets the status of the GPS sensor.
    pub fn status(&self) -> Result<GpsStatus, GpsError> {
        unsafe {
            let status = pros_sys::gps_get_status(self.port.index());
            bail_on!(PROS_ERR_F, status.x);
            let accel = pros_sys::gps_get_accel(self.port.index());
            bail_on!(PROS_ERR_F, accel.x);
            let heading = bail_on!(PROS_ERR_F, pros_sys::gps_get_heading(self.port.index()));

            Ok(GpsStatus {
                x: status.x,
                y: status.y,
                pitch: status.pitch,
                roll: status.roll,
                yaw: status.yaw,
                heading,

                accel_x: accel.x,
                accel_y: accel.y,
                accel_z: accel.z,
            })
        }
    }

    /// Zeroes the rotation of the GPS sensor.
    pub fn zero_rotation(&mut self) -> Result<(), GpsError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::gps_tare_rotation(self.port.index()));
        }
        Ok(())
    }
}

impl SmartDevice for GpsSensor {
    fn port_index(&self) -> u8 {
        self.port.index()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::Gps
    }
}

#[derive(Debug, Snafu)]
pub enum GpsError {
    #[snafu(display("GPS sensor is still calibrating."))]
    StillCalibrating,
    #[snafu(display("{source}"), context(false))]
    Port { source: PortError },
}

map_errno! {
    GpsError {
        EAGAIN => Self::StillCalibrating,
    }
    inherit PortError;
}
