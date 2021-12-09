#![doc = include_str!(concat!(env!("OUT_DIR"), "/README-rustdocified.md"))]
#![deny(missing_docs)]
#![forbid(unsafe_code)]

pub use crate::{dual_crc::DualCrc, rolling_dual_crc::RollingDualCrc, zeros::Zeros};

mod dual_crc;
mod rolling_dual_crc;
mod tables;
mod zeros;
