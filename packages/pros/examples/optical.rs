#![no_std]
#![no_main]

use core::time::Duration;

use pros::prelude::*;

pub struct Robot {
    optical: OpticalSensor,
}
impl Robot {
    pub fn new(peripherals: Peripherals) -> Self {
        Self {
            optical: OpticalSensor::new(peripherals.port_1, true).unwrap(),
        }
    }
}

impl SyncRobot for Robot {
    fn opcontrol(&mut self) -> pros::Result {
        loop {
            println!(
				"-----\nHue: {}\nSaturation: {}\nBrightess: {}\nLast Gesture Direction: {:?}\n-----\n",
				self.optical.hue()?,
				self.optical.saturation()?,
				self.optical.brightness()?,
				self.optical.last_gesture_direction()?
			);

            pros::task::delay(Duration::from_millis(10));
        }
    }
}

sync_robot!(Robot, Robot::new(Peripherals::take().unwrap()));
