//! Efficient implementations for calculating geometry data for game assets using [glam](https://crates.io/crates/glam).

pub use glam;

// TODO: This module structure is confusing.
pub mod bounding;
pub mod vectors;
pub mod ffi;

// TODO: Are these really necessary?
pub use vectors::calculate_smooth_normals;
pub use bounding::calculate_aabb_from_points;
pub use bounding::calculate_bounding_sphere_from_points;
