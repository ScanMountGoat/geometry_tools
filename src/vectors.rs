//! Functions for computing normal, tangent, and bitangent (binormal) vectors.

use glam::Vec3A;
pub use normal::*;
pub use tangent::*;

pub(crate) mod normal;
pub(crate) mod tangent;

// TODO: Is there a way for this to work with vec2 and vec4 as well?
/// Returns a normalized vector based on `target` that is orthogonal to `source` using the Gran-Schmidt process.
fn orthonormalize(target: &Vec3A, source: &Vec3A) -> Vec3A {
    Vec3A::normalize(*target - *source * source.dot(*target))
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    const EPSILON: f32 = 0.0001;

    #[test]
    fn orthogonalize_vector() {
        // Not orthogonal initally.
        let a = Vec3A::new(1.0, 0.5, 0.0);
        let b = Vec3A::new(1.0, 0.0, 0.0);
        assert_ne!(0.0, a.dot(b));

        // a and b should now be orthogonal.
        // dot(a, b) == 0 if a and b are orthogonal.
        let a_ortho_to_b = orthonormalize(&a, &b);
        assert_relative_eq!(0.0, a_ortho_to_b.dot(b), epsilon = EPSILON);
    }

    #[test]
    fn already_orthogonal() {
        // Already orthogonal.
        let a = Vec3A::new(0.0, 1.0, 0.0);
        let b = Vec3A::new(1.0, 0.0, 0.0);
        assert_relative_eq!(0.0, a.dot(b), epsilon = EPSILON);

        // a should remain the same
        let a_ortho_to_b = orthonormalize(&a, &b);
        assert_eq!(a, a_ortho_to_b);
    }
}
