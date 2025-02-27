use pros_sys::{ext_adi_encoder_t, PROS_ERR};

use super::{AdiDevice, AdiDeviceType, AdiError, AdiPort};
use crate::error::bail_on;

#[derive(Debug, Eq, PartialEq)]
pub struct AdiEncoder {
    raw: ext_adi_encoder_t,
    port_top: AdiPort,
    port_bottom: AdiPort,
}

impl AdiEncoder {
    /// Create a new encoder from a top and bottom [`AdiPort`].
    ///
    /// If using an [`AdiExpander`], both ports must be on the same expander module.
    pub fn new(ports: (AdiPort, AdiPort), reverse: bool) -> Result<Self, AdiError> {
        let port_top = ports.0;
        let port_bottom = ports.1;

        if port_top.internal_expander_index() != port_bottom.internal_expander_index() {
            return Err(AdiError::ExpanderPortMismatch);
        }

        let raw = bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_encoder_init(
                port_top.internal_expander_index(),
                port_top.index(),
                port_bottom.index(),
                reverse,
            )
        });

        Ok(Self {
            raw,
            port_top,
            port_bottom,
        })
    }

    /// Resets the encoder to zero.
    pub fn zero(&mut self) -> Result<(), AdiError> {
        bail_on!(PROS_ERR, unsafe { pros_sys::adi_encoder_reset(self.raw) });
        Ok(())
    }

    /// Gets the number of ticks recorded by the encoder.
    pub fn value(&self) -> Result<i32, AdiError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::adi_encoder_get(self.raw)
        }))
    }
}

impl AdiDevice for AdiEncoder {
    type PortIndexOutput = (u8, u8);

    fn port_index(&self) -> Self::PortIndexOutput {
        (self.port_top.index(), self.port_bottom.index())
    }

    fn expander_port_index(&self) -> Option<u8> {
        self.port_top.expander_index()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::LegacyEncoder
    }
}
