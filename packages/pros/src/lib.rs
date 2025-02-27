//! # Pros
//! Opinionated bindings for the PROS library and kernel.
//! Not everything in this library is one to one with the PROS API.
//!
//! Advantages over similar libraries or PROS itself:
//! - Pros-rs has an [`Async executor`](async_runtime) which allows for easy and performant asynchronous code.
//! - Simulation support with [`pros-simulator`](https://crates.io/crates/pros-simulator) and any interface with it (e.g. [`pros-simulator-gui`](https://github.com/pros-rs/pros-simulator-gui))
//! - Active development. Pros-rs is actively developed and maintained.
//! - Pros-rs is a real crate on crates.io instead of a template, or similar. This allows for dependency management with cargo.
//!
//! # Usage
//!
//! When using pros, you have a few options for how you want to get started.
//! You have two options: `async` and `sync`.
//! When using async, an async executor is started and you can use it to run code asynchronously without any FreeRTOS tasks.
//! When using sync, if you want to run code asynchronously you must create a FreeRTOS task.
//!
//! Here are some examples of both:
//!
//! ```rust
//! // Async
//! use pros::prelude::*;
//!
//! #[derive(Default)]
//! struct Robot;
//! impl AsyncRobot for Robot {
//!    async fn opcontrol(&mut self) -> Result {
//!       loop {
//!         // Do something
//!        sleep(Duration::from_millis(20)).await;
//!    }
//! }
//! async_robot!(Robot);
//! ```
//!
//!```rust
//! // Sync
//! use pros::prelude::*;
//!
//! #[derive(Default)]
//! struct Robot;
//! impl SyncRobot for Robot {
//!   fn opcontrol(&mut self) -> Result {
//!      loop {
//!       // Do something
//!      delay(Duration::from_millis(20));
//!    }
//! }
//! sync_robot!(Robot);
//! ```
//!
//! You may have noticed the `#[derive(Default)]` attribute on these Robot structs.
//! If you want to learn why, look at the docs for [`async_robot`] or [`sync_robot`].

#![feature(error_in_core, stdsimd, negative_impls)]
#![no_std]

extern crate alloc;

use core::future::Future;

pub mod async_runtime;
pub mod devices;
pub mod error;
pub mod pid;
pub mod sync;
#[macro_use]
pub mod task;

#[doc(hidden)]
pub use pros_sys as __pros_sys;
#[cfg(target_os = "vexos")]
mod vexos_env;
#[cfg(target_arch = "wasm32")]
mod wasm_env;
#[macro_use]
pub mod lcd;
pub mod competition;
pub mod io;
pub mod lvgl;
pub mod time;
pub mod usd;

pub type Result<T = ()> = core::result::Result<T, alloc::boxed::Box<dyn core::error::Error>>;

pub trait AsyncRobot {
    fn opcontrol(&mut self) -> impl Future<Output = Result> {
        async { Ok(()) }
    }
    fn auto(&mut self) -> impl Future<Output = Result> {
        async { Ok(()) }
    }
    fn disabled(&mut self) -> impl Future<Output = Result> {
        async { Ok(()) }
    }
    fn comp_init(&mut self) -> impl Future<Output = Result> {
        async { Ok(()) }
    }
}

pub trait SyncRobot {
    fn opcontrol(&mut self) -> Result {
        Ok(())
    }
    fn auto(&mut self) -> Result {
        Ok(())
    }
    fn disabled(&mut self) -> Result {
        Ok(())
    }
    fn comp_init(&mut self) -> Result {
        Ok(())
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __gen_sync_exports {
    ($rbt:ty) => {
        pub static mut ROBOT: Option<$rbt> = None;

        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn opcontrol() {
            <$rbt as $crate::SyncRobot>::opcontrol(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before opcontrol")
            })
            .unwrap();
        }

        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn autonomous() {
            <$rbt as $crate::SyncRobot>::auto(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before opcontrol")
            })
            .unwrap();
        }

        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn disabled() {
            <$rbt as $crate::SyncRobot>::disabled(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before opcontrol")
            })
            .unwrap();
        }

        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn competition_initialize() {
            <$rbt as $crate::SyncRobot>::comp_init(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before opcontrol")
            })
            .unwrap();
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __gen_async_exports {
    ($rbt:ty) => {
        pub static mut ROBOT: Option<$rbt> = None;

        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn opcontrol() {
            $crate::async_runtime::block_on(<$rbt as $crate::AsyncRobot>::opcontrol(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before opcontrol")
            }))
            .unwrap();
        }

        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn autonomous() {
            $crate::async_runtime::block_on(<$rbt as $crate::AsyncRobot>::opcontrol(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before auto")
            }))
            .unwrap();
        }

        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn disabled() {
            $crate::async_runtime::block_on(<$rbt as $crate::AsyncRobot>::opcontrol(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before disabled")
            }))
            .unwrap();
        }

        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn competition_initialize() {
            $crate::async_runtime::block_on(<$rbt as $crate::AsyncRobot>::opcontrol(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before comp_init")
            }))
            .unwrap();
        }
    };
}

/// Allows your async robot code to be executed by the pros kernel.
/// If your robot struct implements Default then you can just supply this macro with its type.
/// If not, you can supply an expression that returns your robot type to initialize your robot struct.
/// The code that runs to create your robot struct will run in the initialize function in PROS.
///
/// Example of using the macro with a struct that implements Default:
/// ```rust
/// use pros::prelude::*;
/// #[derive(Default)]
/// struct ExampleRobot;
/// #[async_trait]
/// impl AsyncRobot for ExampleRobot {
///    asnyc fn opcontrol(&mut self) -> pros::Result {
///       println!("Hello, world!");
///      Ok(())
///   }
/// }
/// async_robot!(ExampleRobot);
/// ```
///
/// Example of using the macro with a struct that does not implement Default:
/// ```rust
/// use pros::prelude::*;
/// struct ExampleRobot {
///    x: i32,
/// }
/// #[async_trait]
/// impl AsyncRobot for ExampleRobot {
///     async fn opcontrol(&mut self) -> pros::Result {
///         println!("Hello, world! {}", self.x);
///         Ok(())
///     }
/// }
/// impl ExampleRobot {
///     pub fn new() -> Self {
///        Self { x: 5 }
///    }
/// }
/// async_robot!(ExampleRobot, ExampleRobot::new());
#[macro_export]
macro_rules! async_robot {
    ($rbt:ty) => {
        $crate::__gen_async_exports!($rbt);

        #[no_mangle]
        extern "C" fn initialize() {
            ::pros::task::__init_entrypoint();
            unsafe {
                ROBOT = Some(Default::default());
            }
        }
    };
    ($rbt:ty, $init:expr) => {
        $crate::__gen_async_exports!($rbt);

        #[no_mangle]
        extern "C" fn initialize() {
            ::pros::task::__init_entrypoint();
            unsafe {
                ROBOT = Some($init);
            }
        }
    };
}

/// Allows your sync robot code to be executed by the pros kernel.
/// If your robot struct implements Default then you can just supply this macro with its type.
/// If not, you can supply an expression that returns your robot type to initialize your robot struct.
/// The code that runs to create your robot struct will run in the initialize function in PROS.
///
/// Example of using the macro with a struct that implements Default:
/// ```rust
/// use pros::prelude::*;
/// #[derive(Default)]
/// struct ExampleRobot;
/// impl SyncRobot for ExampleRobot {
///    asnyc fn opcontrol(&mut self) -> pros::Result {
///       println!("Hello, world!");
///      Ok(())
///   }
/// }
/// sync_robot!(ExampleRobot);
/// ```
///
/// Example of using the macro with a struct that does not implement Default:
/// ```rust
/// use pros::prelude::*;
/// struct ExampleRobot {
///    x: i32,
/// }
/// impl SyncRobot for ExampleRobot {
///     async fn opcontrol(&mut self) -> pros::Result {
///         println!("Hello, world! {}", self.x);
///         Ok(())
///     }
/// }
/// impl ExampleRobot {
///     pub fn new() -> Self {
///        Self { x: 5 }
///    }
/// }
/// sync_robot!(ExampleRobot, ExampleRobot::new());
#[macro_export]
macro_rules! sync_robot {
    ($rbt:ty) => {
        $crate::__gen_sync_exports!($rbt);

        #[no_mangle]
        extern "C" fn initialize() {
            ::pros::task::__init_entrypoint();
            unsafe {
                ROBOT = Some(Default::default());
            }
        }
    };
    ($rbt:ty, $init:expr) => {
        $crate::__gen_sync_exports!($rbt);

        #[no_mangle]
        extern "C" fn initialize() {
            ::pros::task::__init_entrypoint();
            unsafe {
                ROBOT = Some($init);
            }
        }
    };
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    let current_task = task::current();

    let task_name = current_task.name().unwrap_or_else(|_| "<unknown>".into());

    // task 'User Initialization (PROS)' panicked at src/lib.rs:22:1:
    // panic message here
    println!("task '{task_name}' {info}");

    unsafe {
        #[cfg(target_arch = "wasm32")]
        wasm_env::sim_log_backtrace();
        pros_sys::exit(1);
    }
}

/// Commonly used features of pros-rs.
/// This module is meant to be glob imported.
pub mod prelude {
    // Import Box from alloc so that it can be used in async_trait!
    pub use alloc::boxed::Box;

    pub use crate::{
        async_robot,
        async_runtime::*,
        dbg,
        devices::{
            adi::{
                analog::{AdiAnalogIn, AdiAnalogOut},
                digital::{AdiDigitalIn, AdiDigitalOut},
                encoder::AdiEncoder,
                gyro::AdiGyro,
                motor::AdiMotor,
                potentiometer::{AdiPotentiometer, AdiPotentiometerType},
                ultrasonic::AdiUltrasonic,
                AdiDevice, AdiPort,
            },
            peripherals::{DynamicPeripherals, Peripherals},
            position::Position,
            smart::{
                distance::DistanceSensor,
                gps::GpsSensor,
                imu::InertialSensor,
                link::{Link, RxLink, TxLink},
                motor::{BrakeMode, Gearset, Motor},
                optical::OpticalSensor,
                rotation::RotationSensor,
                vision::VisionSensor,
                SmartDevice, SmartPort,
            },
        },
        eprint, eprintln,
        error::PortError,
        io::{BufRead, Read, Seek, Write},
        lcd::{buttons::Button, llemu_print, llemu_println, LcdError},
        os_task_local,
        pid::*,
        print, println, sync_robot,
        task::{delay, sleep, spawn},
        AsyncRobot, SyncRobot,
    };
}
