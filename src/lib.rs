//! Efficient implementations for calculating geometry data for game assets using [glam](https://crates.io/crates/glam).
//!
//! Most functions support any type that can be converted into [glam::Vec3A].
//! This allows [glam::Vec3A] and [glam::Vec4] to have identical performance.
//! Using [glam::Vec3] will have slightly reduced performance due to conversions to aligned types.

pub use glam;

pub mod bounding;
pub mod ffi;
pub mod vectors;
