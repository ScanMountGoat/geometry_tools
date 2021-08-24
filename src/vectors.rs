//! Functions for computing normal, tangent, and bitangent (binormal) vectors.

pub use tangent::{calculate_tangents_bitangents, calculate_tangent_w};
pub use normal::calculate_smooth_normals;

mod tangent;
mod normal;
// TODO: This is confusing and should probably just be a separate module.
// TODO: Use a single ffi module?
pub use normal::ffi;