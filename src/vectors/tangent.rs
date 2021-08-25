use std::ops::Mul;

use glam::{Vec2, Vec3A};

use crate::vectors::orthonormalize;

/// The tangent value returned when any component is `NaN` or infinite.
pub const DEFAULT_TANGENT: Vec3A = Vec3A::X;

/// The bitangent value returned when any component is `NaN` or infinite.
pub const DEFAULT_BITANGENT: Vec3A = Vec3A::Y;

pub fn calculate_tangents_bitangents(
    positions: &[Vec3A],
    normals: &[Vec3A],
    uvs: &[Vec2],
    indices: &[u32],
) -> (Vec<Vec3A>, Vec<Vec3A>) {
    // TODO: Return an error type.
    // if (normals.Count != positions.Count)
    //     throw new System.ArgumentOutOfRangeException(nameof(normals), "Vector source lengths do not match.");

    // if (uvs.Count != positions.Count)
    //     throw new System.ArgumentOutOfRangeException(nameof(uvs), "Vector source lengths do not match.");

    let mut tangents = vec![Vec3A::ZERO; positions.len()];
    let mut bitangents = vec![Vec3A::ZERO; positions.len()];

    // TODO: Rewrite this to be more idiomatic.
    // TODO: There's a nicer way to find each chunk of three indices.
    // Calculate the vectors.
    for i in (0..indices.len()).step_by(3) {
        let (tangent, bitangent) = calculate_tangent_bitangent(
            &positions[indices[i] as usize],
            &positions[indices[i + 1] as usize],
            &positions[indices[i + 2] as usize],
            &uvs[indices[i] as usize],
            &uvs[indices[i + 1] as usize],
            &uvs[indices[i + 2] as usize],
        );

        tangents[indices[i] as usize] += tangent;
        tangents[indices[i + 1] as usize] += tangent;
        tangents[indices[i + 2] as usize] += tangent;

        bitangents[indices[i] as usize] += bitangent;
        bitangents[indices[i + 1] as usize] += bitangent;
        bitangents[indices[i + 2] as usize] += bitangent;
    }

    // Even if the vectors are not zero, they may still sum to zero.
    for i in 0..tangents.len() {
        if tangents[i].length_squared() == 0.0 {
            tangents[i] = DEFAULT_TANGENT;
        }

        if bitangents[i].length_squared() == 0.0 {
            bitangents[i] = DEFAULT_BITANGENT;
        }
    }

    // Account for mirrored normal maps.
    for i in 0..bitangents.len() {
        // TODO: Implement Gram–Schmidt orthogonalization.
        // The default bitangent may be parallel to the normal vector.
        if bitangents[i].cross(normals[i]).length_squared() != 0.0 {
            bitangents[i] = orthonormalize(&bitangents[i], &normals[i]);
        }
        // TODO: Document why this flip is necessary.
        bitangents[i] *= -1.0;
    }

    for i in 0..tangents.len() {
        tangents[i] = tangents[i].normalize();
        bitangents[i] = bitangents[i].normalize();
    }

    (tangents, bitangents)
}

/// Calculates the tangent sign, which is often stored in the W component for a 4 component tangent vector.
/// The bitangent can be generated from the tangent and normal vector. This step will normally be done by shader code for the GPU.
/**
```rust
# let tangent = glam::Vec3A::ZERO;
# let bitangent = glam::Vec3A::ZERO;
# let normal = glam::Vec3A::ZERO;
let tangent_w = geometry_tools::vectors::calculate_tangent_w(&normal, &tangent, &bitangent);
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

fn calculate_tangent_bitangent(
    v0: &Vec3A,
    v1: &Vec3A,
    v2: &Vec3A,
    uv0: &Vec2,
    uv1: &Vec2,
    uv2: &Vec2,
) -> (Vec3A, Vec3A) {
    let pos_a = *v1 - *v0;
    let pos_b = *v2 - *v0;

    let uv_a = *uv1 - *uv0;
    let uv_b = *uv2 - *uv0;

    let div = uv_a.x * uv_b.y - uv_b.x * uv_a.y;

    // Fix +/- infinity from division by zero.
    // TODO: Make this check less strict?
    let r = if div != 0.0 { 1.0 / div } else { 1.0 };

    let tangent = calculate_tangent(&pos_a, &pos_b, &uv_a, &uv_b, r);
    let bitangent = calculate_bitangent(&pos_a, &pos_b, &uv_a, &uv_b, r);

    // Set zero vectors to arbitrarily chosen orthogonal vectors.
    // This prevents unwanted black faces when rendering tangent space normal maps.
    let tangent = if tangent.length_squared() == 0.0 {
        DEFAULT_TANGENT
    } else {
        tangent
    };

    let bitangent = if bitangent.length_squared() == 0.0 {
        DEFAULT_BITANGENT
    } else {
        bitangent
    };

    (tangent, bitangent)
}

fn calculate_tangent(pos_a: &Vec3A, pos_b: &Vec3A, uv_a: &Vec2, uv_b: &Vec2, r: f32) -> Vec3A {
    (pos_a.mul(uv_b.y) - pos_b.mul(uv_a.y)) * r
}

fn calculate_bitangent(pos_a: &Vec3A, pos_b: &Vec3A, uv_a: &Vec2, uv_b: &Vec2, r: f32) -> Vec3A {
    (pos_b.mul(uv_a.x) - pos_a.mul(uv_b.x)) * r
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::{assert_relative_eq, relative_eq};
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
    fn different_uvs_different_positions() {
        let v1 = Vec3A::new(1.0, 0.0, 0.0);
        let v2 = Vec3A::new(0.0, 1.0, 0.0);
        let v3 = Vec3A::new(0.0, 0.0, 1.0);
        let uv1 = Vec2::new(1.0, 0.0);
        let uv2 = Vec2::new(0.0, 1.0);
        let uv3 = Vec2::new(1.0, 1.0);

        let (tangent, bitangent) = calculate_tangent_bitangent(&v1, &v2, &v3, &uv1, &uv2, &uv3);

        assert_eq!(Vec3A::new(0.0, -1.0, 1.0), tangent);
        assert_eq!(Vec3A::new(-1.0, 0.0, 1.0), bitangent);
    }

    #[test]
    fn different_uvs_same_positions() {
        let v1 = Vec3A::new(1.0, 0.0, 0.0);
        let v2 = Vec3A::new(1.0, 0.0, 0.0);
        let v3 = Vec3A::new(1.0, 0.0, 0.0);
        let uv1 = Vec2::new(1.0, 0.0);
        let uv2 = Vec2::new(0.0, 1.0);
        let uv3 = Vec2::new(1.0, 1.0);
        let (tangent, bitangent) = calculate_tangent_bitangent(&v1, &v2, &v3, &uv1, &uv2, &uv3);

        // Make sure tangents and bitangents aren't all zero.
        assert_eq!(DEFAULT_TANGENT, tangent);
        assert_eq!(DEFAULT_BITANGENT, bitangent);
    }

    #[test]
    fn same_uvs_different_positions() {
        let v1 = Vec3A::new(1.0, 0.0, 0.0);
        let v2 = Vec3A::new(0.0, 1.0, 0.0);
        let v3 = Vec3A::new(0.0, 0.0, 1.0);
        let uv1 = Vec2::new(1.0, 1.0);
        let uv2 = Vec2::new(1.0, 1.0);
        let uv3 = Vec2::new(1.0, 1.0);
        let (tangent, bitangent) = calculate_tangent_bitangent(&v1, &v2, &v3, &uv1, &uv2, &uv3);

        // Make sure tangents and bitangents aren't all zero.
        assert_eq!(DEFAULT_TANGENT, tangent);
        assert_eq!(DEFAULT_BITANGENT, bitangent);
    }

    #[test]
    fn same_uvs_same_positions() {
        let v1 = Vec3A::new(1.0, 0.0, 0.0);
        let v2 = Vec3A::new(1.0, 0.0, 0.0);
        let v3 = Vec3A::new(1.0, 0.0, 0.0);
        let uv1 = Vec2::new(1.0, 1.0);
        let uv2 = Vec2::new(1.0, 1.0);
        let uv3 = Vec2::new(1.0, 1.0);
        let (tangent, bitangent) = calculate_tangent_bitangent(&v1, &v2, &v3, &uv1, &uv2, &uv3);

        // Make sure tangents and bitangents aren't all zero.
        assert_eq!(DEFAULT_TANGENT, tangent);
        assert_eq!(DEFAULT_BITANGENT, bitangent);
    }

    #[test]
    fn uvs_would_cause_divide_by_zero() {
        let v1 = Vec3A::new(1.0, 0.0, 0.0);
        let v2 = Vec3A::new(0.0, 1.0, 0.0);
        let v3 = Vec3A::new(0.0, 0.0, 1.0);

        // Force the divisor to be 0.
        let uv1 = Vec2::new(0.5, 0.0);
        let uv2 = Vec2::new(0.5, 0.0);
        let uv3 = Vec2::new(1.0, 1.0);

        let (tangent, bitangent) = calculate_tangent_bitangent(&v1, &v2, &v3, &uv1, &uv2, &uv3);

        // Check for division by 0.
        assert!(tangent.is_finite());
        assert!(bitangent.is_finite());
    }

    #[test]
    fn triangle_list_single_triangle() {
        let positions = vec![
            Vec3A::new(0.0, 0.0, 0.0),
            Vec3A::new(0.0, 1.0, 0.0),
            Vec3A::new(1.0, 1.0, 0.0),
        ];
        let normals = vec![
            Vec3A::new(0.0, 0.0, 1.0),
            Vec3A::new(0.0, 0.0, 1.0),
            Vec3A::new(0.0, 0.0, 1.0),
        ];
        let uvs = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
        ];

        let (tangents, bitangents) =
            calculate_tangents_bitangents(&positions, &normals, &uvs, &[0, 1, 2]);

        assert_eq!(3, tangents.len());
        assert_eq!(3, bitangents.len());

        // The tangent should point in the direct of the U coordinate.
        for tangent in tangents {
            assert_relative_eq!(0.0, tangent.x, epsilon = EPSILON);
            assert_relative_eq!(1.0, tangent.y, epsilon = EPSILON);
            assert_relative_eq!(0.0, tangent.z, epsilon = EPSILON);
        }

        // The bitangent should be orthogonal to the tangent and normal.
        // The only option in this case is to use the x-axis.
        // TODO: Why is the sign flip necessary?
        for bitangent in bitangents {
            assert_relative_eq!(-1.0, bitangent.x, epsilon = EPSILON);
            assert_relative_eq!(0.0, bitangent.y, epsilon = EPSILON);
            assert_relative_eq!(0.0, bitangent.z, epsilon = EPSILON);
        }
    }

    #[test]
    fn triangle_list_basic_cube_normalized_no_weird_floats() {
        let (tangents, bitangents) = calculate_tangents_bitangents(
            &cube_positions(),
            &cube_normals(),
            &cube_uvs(),
            &cube_indices(),
        );

        assert_eq!(24, tangents.len());
        assert_eq!(24, bitangents.len());

        for (tangent, bitangent) in tangents.iter().zip(bitangents) {
            assert_relative_eq!(1.0, tangent.length(), epsilon = EPSILON);
            assert_relative_eq!(1.0, bitangent.length(), epsilon = EPSILON);
            assert!(is_good_tangent_bitangent(tangent, &bitangent));
        }
    }

    #[test]
    fn triangle_list_no_vertices() {
        let (tangents, bitangents) = calculate_tangents_bitangents(&[], &[], &[], &[]);

        assert!(tangents.is_empty());
        assert!(bitangents.is_empty());
    }

    // TODO: Test the actual values produced for a small set of test points?

    // TODO: Enable these tests once the return type is fixed.
    #[test]
    #[ignore]
    #[should_panic(expected = "Vector source lengths do not match.")]
    fn triangle_list_incorrect_normals_count() {
        calculate_tangents_bitangents(&[], &[Vec3A::ZERO], &[], &[]);
    }

    #[test]
    #[ignore]
    #[should_panic(expected = "Vector source lengths do not match.")]
    fn triangle_list_incorrect_uvs_count() {
        calculate_tangents_bitangents(&[], &[], &[Vec2::ZERO], &[]);
    }

    fn is_good_tangent_bitangent(tangent: &Vec3A, bitangent: &Vec3A) -> bool {
        // Check that the values are finite and very close to being orthogonal.
        tangent.is_finite()
            && bitangent.is_finite()
            && relative_eq!(0.0, tangent.dot(*bitangent), epsilon = EPSILON)
    }

    #[test]
    fn tangent_w_should_flip() {
        // cross(tangent,bitangent) is in the opposite direction of the normal.
        // This occurs on the side with mirrored UVs.
        let tangent = Vec3A::new(0.0, 1.0, 0.0);
        let bitangent = Vec3A::new(1.0, 0.0, 0.0);
        let normal = Vec3A::new(0.0, 0.0, 1.0);
        let w = calculate_tangent_w(&normal, &tangent, &bitangent);
        assert_eq!(-1.0, w);
    }

    #[test]
    fn tangent_w_should_not_flip() {
        // cross(tangent, bitangent) is in the same direction as the normal.
        // This occurs on the side without mirrored UVs.
        let tangent = Vec3A::new(1.0, 0.0, 0.0);
        let bitangent = Vec3A::new(0.0, 1.0, 0.0);
        let normal = Vec3A::new(0.0, 0.0, 1.0);
        let w = calculate_tangent_w(&normal, &tangent, &bitangent);
        assert_eq!(1.0, w);
    }

    #[test]
    fn tangent_w_should_not_be_zero() {
        // cross(tangent, bitangent) is orthogonal to the normal.
        let tangent = Vec3A::new(1.0, 0.0, 0.0);
        let bitangent = Vec3A::new(0.0, 1.0, 0.0);
        let normal = Vec3A::new(1.0, 0.0, 0.0);
        let w = calculate_tangent_w(&normal, &tangent, &bitangent);
        assert_eq!(1.0, w);
    }
}
