#![no_std]
extern crate alloc;

use embedded_graphics_core::{pixelcolor::Rgb888, prelude::*};
use snafu::Snafu;

pub struct VexDisplay;

impl OriginDimensions for VexDisplay {
    fn size(&self) -> Size {
        Size::new(480, 272)
    }
}

impl DrawTarget for VexDisplay {
    type Color = Rgb888;
    type Error = Error;
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            let color: u32 =
                ((pixel.1.r() as u32) << 16) | (pixel.1.g() as u32) << 8 | pixel.1.b() as u32;
            if pixel.0.x < 0 || pixel.0.y < 0 {
                return Err(Error::OutOfRange);
            }
            unsafe {
                pros_sys::vexDisplayCopyRect(
                    pixel.0.x as u32,
                    pixel.0.y as u32,
                    pixel.0.x as u32 + 1,
                    pixel.0.y as u32 + 1,
                    (&color) as _,
                    2,
                );
            }
        }
        Ok(())
    }
}

#[derive(Snafu, Debug)]
pub enum Error {
    #[snafu(display("Pixel point out of range"))]
    OutOfRange,
}
