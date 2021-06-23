use glam::{Vec2, Vec3A};

pub fn calculate_tangents_bitangents(
    positions: &[Vec3A],
    normals: &[Vec3A],
    uvs: &[Vec2],
    indices: &[u32],
) -> (Vec<Vec3A>, Vec<Vec3A>) {
    (positions.into(), positions.into())
}

/// Calculates the tangent sign, which is often stored in the W component for a 4 component tangent vector.
/// The bitangent can be generated from the tangent and normal vector. This step will normally be done by shader code for the GPU.
/**
```rust
# let tangent = glam::Vec3A::ZERO;
# let bitangent = glam::Vec3A::ZERO;
# let normal = glam::Vec3A::ZERO;=
let tangent_w = calculate_tangent_w(&normal, &tangent, &bitangent);
let generated_bitangent = normal.cross(tangent) * tangent_w;
```
*/
pub fn calculate_tangent_w(normal: &Vec3A, tangent: &Vec3A, bitangent: &Vec3A) -> f32 {
    // 0.0 should stil return 1.0 to avoid generating black bitangents.
    if tangent.cross(*bitangent).dot(*normal) >= 0.0 {
        1.0
    } else {
        -1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::relative_eq;
    use glam::Vec2;

    const EPSILON: f32 = 0.0001;

    fn cube_positions() -> Vec<Vec3A> {
        vec![
            Vec3A::new(-0.5, -0.5, 0.5),
            Vec3A::new(0.5, -0.5, 0.5),
            Vec3A::new(-0.5, 0.5, 0.5),
            Vec3A::new(0.5, 0.5, 0.5),
            Vec3A::new(-0.5, 0.5, 0.5),
            Vec3A::new(0.5, 0.5, 0.5),
            Vec3A::new(-0.5, 0.5, -0.5),
            Vec3A::new(0.5, 0.5, -0.5),
            Vec3A::new(-0.5, 0.5, -0.5),
            Vec3A::new(0.5, 0.5, -0.5),
            Vec3A::new(-0.5, -0.5, -0.5),
            Vec3A::new(0.5, -0.5, -0.5),
            Vec3A::new(-0.5, -0.5, -0.5),
            Vec3A::new(0.5, -0.5, -0.5),
            Vec3A::new(-0.5, -0.5, 0.5),
            Vec3A::new(0.5, -0.5, 0.5),
            Vec3A::new(0.5, -0.5, 0.5),
            Vec3A::new(0.5, -0.5, -0.5),
            Vec3A::new(0.5, 0.5, 0.5),
            Vec3A::new(0.5, 0.5, -0.5),
            Vec3A::new(-0.5, -0.5, -0.5),
            Vec3A::new(-0.5, -0.5, 0.5),
            Vec3A::new(-0.5, 0.5, -0.5),
            Vec3A::new(-0.5, 0.5, 0.5),
        ]
    }

    fn cube_normals() -> Vec<Vec3A> {
        vec![
            Vec3A::new(0.0, 0.0, 1.0),
            Vec3A::new(0.0, 0.0, 1.0),
            Vec3A::new(0.0, 0.0, 1.0),
            Vec3A::new(0.0, 0.0, 1.0),
            Vec3A::new(0.0, 1.0, 0.0),
            Vec3A::new(0.0, 1.0, 0.0),
            Vec3A::new(0.0, 1.0, 0.0),
            Vec3A::new(0.0, 1.0, 0.0),
            Vec3A::new(0.0, 0.0, -1.0),
            Vec3A::new(0.0, 0.0, -1.0),
            Vec3A::new(0.0, 0.0, -1.0),
            Vec3A::new(0.0, 0.0, -1.0),
            Vec3A::new(0.0, -1.0, 0.0),
            Vec3A::new(0.0, -1.0, 0.0),
            Vec3A::new(0.0, -1.0, 0.0),
            Vec3A::new(0.0, -1.0, 0.0),
            Vec3A::new(1.0, 0.0, 0.0),
            Vec3A::new(1.0, 0.0, 0.0),
            Vec3A::new(1.0, 0.0, 0.0),
            Vec3A::new(1.0, 0.0, 0.0),
            Vec3A::new(-1.0, 0.0, 0.0),
            Vec3A::new(-1.0, 0.0, 0.0),
            Vec3A::new(-1.0, 0.0, 0.0),
            Vec3A::new(-1.0, 0.0, 0.0),
        ]
    }

    fn cube_uvs() -> Vec<Vec2> {
        vec![
            Vec2::new(0.375, 1.0),
            Vec2::new(0.625, 1.0),
            Vec2::new(0.375, 0.75),
            Vec2::new(0.625, 0.75),
            Vec2::new(0.375, 0.75),
            Vec2::new(0.625, 0.75),
            Vec2::new(0.375, 0.5),
            Vec2::new(0.625, 0.5),
            Vec2::new(0.375, 0.5),
            Vec2::new(0.625, 0.5),
            Vec2::new(0.375, 0.25),
            Vec2::new(0.625, 0.25),
            Vec2::new(0.375, 0.25),
            Vec2::new(0.625, 0.25),
            Vec2::new(0.375, 0.0),
            Vec2::new(0.625, 0.0),
            Vec2::new(0.625, 1.0),
            Vec2::new(0.875, 1.0),
            Vec2::new(0.625, 0.75),
            Vec2::new(0.875, 0.75),
            Vec2::new(0.125, 1.0),
            Vec2::new(0.375, 1.0),
            Vec2::new(0.125, 0.75),
            Vec2::new(0.375, 0.75),
        ]
    }

    fn cube_indices() -> Vec<u32> {
        vec![
            0, 1, 2, 2, 1, 3, 4, 5, 6, 6, 5, 7, 8, 9, 10, 10, 9, 11, 12, 13, 14, 14, 13, 15, 16,
            17, 18, 18, 17, 19, 20, 21, 22, 22, 21, 23,
        ]
    }

    #[test]
    fn three_vertices_normalized() {
        let values3d = vec![
            Vec3A::new(1.0, 0.0, 0.0),
            Vec3A::new(0.0, 1.0, 0.0),
            Vec3A::new(0.0, 0.0, 1.0),
        ];
        let values2d = vec![
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 1.0),
        ];

        let (tangents, bitangents) =
            calculate_tangents_bitangents(&values3d, &values3d, &values2d, &[0, 1, 2]);

        // Ensure vectors are normalized.
        for (tangent, bitangent) in tangents.iter().zip(bitangents) {
            assert!(relative_eq!(1.0, tangent.length(), epsilon = EPSILON));
            assert!(relative_eq!(1.0, bitangent.length(), epsilon = EPSILON));
        }
    }

    #[test]
    fn basic_cube_normalized_no_weird_floats() {
        let (tangents, bitangents) = calculate_tangents_bitangents(
            &cube_positions(),
            &cube_normals(),
            &cube_uvs(),
            &cube_indices(),
        );
        for (tangent, bitangent) in tangents.iter().zip(bitangents) {
            assert!(relative_eq!(1.0, tangent.length(), epsilon = EPSILON));
            assert!(relative_eq!(1.0, bitangent.length(), epsilon = EPSILON));
            assert!(is_good_tangent_bitangent(tangent, &bitangent));
        }
    }

    #[test]
    fn no_vertices() {
        let (tangents, bitangents) = calculate_tangents_bitangents(&[], &[], &[], &[]);

        assert_eq!(0, tangents.len());
        assert_eq!(0, bitangents.len());
    }

    #[test]
    #[should_panic(expected = "Vector source lengths do not match.")]
    fn incorrect_normals_count() {
        let (tangents, bitangents) = calculate_tangents_bitangents(&[], &[Vec3A::ZERO], &[], &[]);
    }

    #[test]
    #[should_panic(expected = "Vector source lengths do not match.")]
    fn incorrect_uvs_count() {
        let (tangents, bitangents) = calculate_tangents_bitangents(&[], &[], &[Vec2::ZERO], &[]);
    }

    fn is_good_tangent_bitangent(tangent: &Vec3A, bitangent: &Vec3A) -> bool {
        tangent.is_finite()
            && bitangent.is_finite()
            && relative_eq!(0.0, tangent.dot(*bitangent), epsilon = EPSILON)
    }

    #[test]
    fn should_flip() {
        // cross(tangent,bitangent) is in the opposite direction of the normal.
        // This occurs on the side with mirrored UVs.
        let tangent = Vec3A::new(0.0, 1.0, 0.0);
        let bitangent = Vec3A::new(1.0, 0.0, 0.0);
        let normal = Vec3A::new(0.0, 0.0, 1.0);
        let w = calculate_tangent_w(&normal, &tangent, &bitangent);
        assert_eq!(-1.0, w);
    }

    #[test]
    fn should_not_flip() {
        // cross(tangent, bitangent) is in the same direction as the normal.
        // This occurs on the side without mirrored UVs.
        let tangent = Vec3A::new(1.0, 0.0, 0.0);
        let bitangent = Vec3A::new(0.0, 1.0, 0.0);
        let normal = Vec3A::new(0.0, 0.0, 1.0);
        let w = calculate_tangent_w(&normal, &tangent, &bitangent);
        assert_eq!(1.0, w);
    }

    #[test]
    fn should_not_be_zero() {
        // cross(tangent, bitangent) is orthogonal to the normal.
        let tangent = Vec3A::new(1.0, 0.0, 0.0);
        let bitangent = Vec3A::new(0.0, 1.0, 0.0);
        let normal = Vec3A::new(1.0, 0.0, 0.0);
        let w = calculate_tangent_w(&normal, &tangent, &bitangent);
        assert_eq!(1.0, w);
    }
}
