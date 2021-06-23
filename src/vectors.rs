pub use tangent::calculate_tangents_bitangents;
pub use normal::calculate_smooth_normals;

mod tangent;
mod normal;
pub use normal::ffi;