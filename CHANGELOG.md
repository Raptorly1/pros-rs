# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!--
Before releasing:

- change versions in Cargo.toml
- change Unreleased to the version number
- create new Unreleased section
- update links at the end of the document
- add "New Contributors" section if there were any first-time contributors

### New Contributors

- @new-contributor made their first contribution in #11!
-->

## [Unreleased]

### Added

### Fixed

### Changed

### Removed

## [0.7.0]

### Added

- `SmartPort` struct for device access. (#34)
- `SmartDevice` trait for common functionality across smart port devices. (#34)
- Methods to get a device's port number as well as determine if the device is plugged in or not. (#34)
- Added various missing derives for hardware-related data structures. (#34)
- `CompetitionSystem` and `CompetitionMode` structs for better retrieving information about the robot's competition state. (#38)
- `competition::system` method for retrieving the type of competition control the robot is connected to. (#38)
- New `From` implementation to convert `Quaternion` and `Euler` to their pros-sys equivalents. (#45)
- `pros::io` module for I/O related operations. (#30)
- Various types from the `no_std_io` have are re-exported from this module to provide missing functionality from `std`. (#30)
- Macros for printing to stdout (`println`, `print`, `eprintln`, etc...) (#30)
- All ADI device bindings (#55)
- `LocalKey` now has `Cell`/`RefCell`-specific methods for setting and taking values. (#42)
- `Peripherals` and `DynamicPeripherals` structs to ensure that you have only registered one device on a given smart or ADI port. (#53)

### Fixed

- Fixed competition state-related getters in the `pros::competition` module. (#38)
- Fixed error handling in IMU sensor bindings. (#37)
- Fixed errors in doctests and examples throughout the crate. (#37)
- Fixed Missing ERRNO and ADI config variants in pros-sys (#55)
- Fixed incorrect error handling with `InertialSensor::status`. (#65)
- `Controller::status` now handles errors by returning `Result<ControllerStatus, ControllerError>`. (**Breaking Change**) (#74)

### Changed

- Overhauled the `competition` module with more straightforward getters for competition state. (#38) (**Breaking Change**)
- LLEMU-related macros have been prefixed with `llemu_` (e.g. `llemu_println`). (**Breaking Change**) (#30)
- Added `Debug`, `Copy`, and `Clone` derives for common structs (#37)
- Renamed `InertialSensor::is_calibrating` to `InertialSensor::calibrating`. (**Breaking Change**) (#65)
- Battery API functions now return `Result<_, BatteryError>`. (**Breaking Change**) (#62)
- Renamed `battery::get_capacity` to `battery::capacity`, `battery::get_current` -> `battery::current`, `battery::get_temperature` -> `battery::temperature`, `battery::get_voltage` -> `battery::voltage`. (**Breaking Change**) (#62)

### Removed

- Removed several broken bindings in `pros_sys` relating to competition state. (#38) (**Breaking Change**)
- `LocalKey` no longer implements `set` for non-`Cell`/`RefCell` stored values. (**Breaking change**) (#42)
- Removed the now-redundant `InertialStatus::error` function. (**Breaking Change**) (#65)

## [0.6.0] - 2024-01-14

### Added

### Fixed

- GPS sensor `set_offset` function now returns a result. The relevant PROS C bindings have been fixed as well. (**Breaking change**)
- FreeRTOS task creation now does not garble data that the provided closure captured.
- Grammar in the feature request template has been fixed.
- Wasm build flags have been updated and fixed.

### Changed

- Panicking behavior has been improved so that spawned tasks will not panic the entire program.
- Panic messages are now improved and printed over the serial connection.
- `AsyncRobot` should now be implemented using the newly stabilized async trait syntax instead of the old `async_trait` attribute macro. (**Breaking change**)
- Add contributing information, pull request templates, and changelog. (#34)
- `AdiPort` is now structured with ADI expander modules in mind. (**Breaking change**) (#34)
- Reorganized ADI, Smart Port, and builtin devices into a common `devices` module. (**Breaking change**) (#34)
- Devices now take `SmartPort`/`AdiPort` rather than a raw port index. (**Breaking change**) (#34)
- All devices now take `&mut self` for methods modifying hardware state. (**Breaking change**) (#34)

### Removed

- `Copy`/`Clone` derives for some existing device types. (**Breaking change**) (#34)
- A nonexistent runner for armv7a-vexos-eabi target has been removed from the cargo config.

## [0.5.0] - 2024-01-08

### Added

- Standard library like `Instant`s
- Optical sensor bindings.
- IMU sensor bindings.

### Fixed

- The async executor now does not starve the OS of cycles when unnecessary.

### Changed

- Updated readme with fixed grammar.

### Removed

## [0.4.0] - 2024-01-02

### Added

- Add methods to controller for checking individual buttons and axes.

### Fixed

### Changed

- Write doc comments for previously undocumented modules and functions.

### Removed

[unreleased]: https://github.com/pros-rs/pros-rs/compare/v0.7.0...HEAD
[0.4.0]: https://github.com/pros-rs/pros-rs/releases/tag/v0.4.0
[0.5.0]: https://github.com/pros-rs/pros-rs/compare/v0.4.0...v0.5.0
[0.6.0]: https://github.com/pros-rs/pros-rs/compare/v0.5.0...v0.6.0
[0.7.0]: https://github.com/pros-rs/pros-rs/compare/v0.6.0...v0.7.0
