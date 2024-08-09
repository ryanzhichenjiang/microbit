//! microbit contains everything required to get started with the use of Rust
//! to create firmwares for the fabulous [BBC micro:bit](https://microbit.org)
//! microcontroller board.
#![doc(html_root_url = "https://docs.rs/microbit-common/0.15.1")]
#![no_std]
#![deny(missing_docs)]
#![allow(non_camel_case_types)]

pub use nrf52833_hal as hal;

pub use hal::pac;
pub use hal::pac::Peripherals;

pub mod adc;
pub mod board;
pub mod display;
pub mod gpio;

pub use board::Board;
