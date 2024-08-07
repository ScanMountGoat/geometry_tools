use std::ops::Mul;
use thiserror::Error;

use glam::{Vec2, Vec3A, Vec4};

use crate::vectors::orthonormalize;

/// The value returned when any component of the calculated tangent would be `NaN` or infinite.
pub const DEFAULT_TANGENT: Vec3A = Vec3A::X;

/// The value returned when any component of the calculated bitangent would be `NaN` or infinite.
pub const DEFAULT_BITANGENT: Vec3A = Vec3A::Y;

/// Errors that can occur while calculating tangents or bitangents.
#[derive(Error, Debug)]
pub enum TangentBitangentError {
    #[error(
        "The list sizes do not match. Positions: {}, Normals: {}, uvs: {}.",
        position_count,
        normal_count,
        uv_count
    )]
    AttributeCountMismatch {
        position_count: usize,
        normal_count: usize,
        uv_count: usize,
    },
    #[error(
        "A vertex index count of {} is not supported. Expected {} to be divisible by 3.",
        index_count,
        index_count
    )]
    InvalidIndexCont { index_count: usize },
}

// TODO: Rewrite these functions to update existing array to better support ffi.

/// Calculates smooth per-vertex tangents and bitangents by averaging over the vertices in each face.
/// `indices` is assumed to contain triangle indices for `positions`, so `indices.len()` should be a multiple of 3.
/// If either of `positions` or `indices` is empty, the result is empty.
/// # Examples
/**
```rust
use geometry_tools::vectors::calculate_tangents_bitangents;
use glam::Vec3A;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let positions = vec![glam::Vec3A::ZERO; 3];
# let normals = vec![glam::Vec3A::ZERO; 3];
# let uvs = vec![glam::Vec2::ZERO; 3];
# let indices = vec![0, 1, 2];
// Some applications require multiplying the resulting bitangents by -1.0.
let (tangents, bitangents) = calculate_tangents_bitangents(&positions, &normals, &uvs, &indices)?;
# Ok(())
# }
```
 */
pub fn calculate_tangents_bitangents<P, N, I>(
    positions: &[P],
    normals: &[N],
    uvs: &[Vec2],
    indices: &[I],
) -> Result<(Vec<Vec3A>, Vec<Vec3A>), TangentBitangentError>
where
    P: Into<Vec3A> + Copy,
    N: Into<Vec3A> + Copy,
    I: TryInto<usize> + Copy,
    <I as TryInto<usize>>::Error: std::fmt::Debug,
{
    // TODO: This can be generic over the face count?
    if indices.len() % 3 != 0 {
        return Err(TangentBitangentError::InvalidIndexCont {
            index_count: indices.len(),
        });
    }

    if !(positions.len() == normals.len() && normals.len() == uvs.len()) {
        return Err(TangentBitangentError::AttributeCountMismatch {
            position_count: positions.len(),
            normal_count: normals.len(),
            uv_count: uvs.len(),
        });
    }

    let mut tangents = vec![Vec3A::ZERO; positions.len()];
    let mut bitangents = vec![Vec3A::ZERO; positions.len()];

    // Calculate the vectors.
    for face in indices.chunks(3) {
        if let [v0, v1, v2] = face {
            let v0 = (*v0).try_into().unwrap();
            let v1 = (*v1).try_into().unwrap();
            let v2 = (*v2).try_into().unwrap();
            let (tangent, bitangent) = calculate_tangent_bitangent(
                &positions[v0].into(),
                &positions[v1].into(),
                &positions[v2].into(),
                &uvs[v0],
                &uvs[v1],
                &uvs[v2],
            );

            tangents[v0] += tangent;
            tangents[v1] += tangent;
            tangents[v2] += tangent;

            bitangents[v0] += bitangent;
            bitangents[v1] += bitangent;
            bitangents[v2] += bitangent;
        }
    }

    // Even if the vectors are not zero, they may still sum to zero.
    for tangent in tangents.iter_mut() {
        if tangent.length_squared() == 0.0 {
            *tangent = DEFAULT_TANGENT;
        }

        *tangent = tangent.normalize_or_zero();
    }

    for bitangent in bitangents.iter_mut() {
        if bitangent.length_squared() == 0.0 {
            *bitangent = DEFAULT_BITANGENT;
        }
    }

    for (bitangent, normal) in bitangents.iter_mut().zip(normals.iter()) {
        // Account for mirrored normal maps.
        // The default bitangent may be parallel to the normal vector.
        let normal = (*normal).into();
        if bitangent.cross(normal).length_squared() != 0.0 {
            *bitangent = orthonormalize(bitangent, &normal);
        }

        *bitangent = bitangent.normalize_or_zero();
    }

    Ok((tangents, bitangents))
}

/// Calculates smooth per-vertex tangents by averaging over the vertices in each face.
/// The 4th component contains the tangent sign and can be used to calculate the bitangent vectors.
/// This step will normally be done by shader code for the GPU.
///
/// `indices` is assumed to contain triangle indices for `positions`, so `indices.len()` should be a multiple of 3.
/// If either of `positions` or `indices` is empty, the result is empty.
/// # Examples
/**
```rust
use geometry_tools::vectors::calculate_tangents;
use glam::Vec3A;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let positions = vec![glam::Vec3A::ZERO; 3];
# let normals = vec![glam::Vec3A::ZERO; 3];
# let uvs = vec![glam::Vec2::ZERO; 3];
# let indices = vec![0, 1, 2];
let tangents = calculate_tangents(&positions, &normals, &uvs, &indices)?;

// This step is often done by shader code for the GPU.
// Some applications require multiplying the resulting bitangents by -1.0.
let bitangents: Vec<Vec3A> = tangents
    .iter()
    .zip(normals.iter())
    .map(|(t, n)| Vec3A::from_vec4(*t).cross(*n) * t.w)
    .collect();
# Ok(())
# }
```
 */
pub fn calculate_tangents<P, N, I>(
    positions: &[P],
    normals: &[N],
    uvs: &[Vec2],
    indices: &[I],
) -> Result<Vec<Vec4>, TangentBitangentError>
where
    P: Into<Vec3A> + Copy,
    N: Into<Vec3A> + Copy,
    I: TryInto<usize> + Copy,
    <I as TryInto<usize>>::Error: std::fmt::Debug,
{
    let (tangents, bitangents) = calculate_tangents_bitangents(positions, normals, uvs, indices)?;

    // Compute the w component for each tangent.
    // TODO: Compute this without computing and immediately discarding bitangent vectors?
    let tangents_with_w = tangents
        .iter()
        .zip(bitangents.iter())
        .zip(normals.iter())
        .map(|((t, b), n)| {
            let w = calculate_tangent_w(*t, *b, (*n).into());
            Vec4::new(t.x, t.y, t.z, w)
        })
        .collect();
    Ok(tangents_with_w)
}

/// Calculates the tangent sign of 1.0 or -1.0, which is often stored in the W component for a 4 component tangent vector.
/// The tangent sign is used to flip the generated bitangent to account for mirrored (overlapping) texture coordinates.
/// Depending on the conventions of the game or application, it may be necessary to multiply the returned value by -1.0.
/// # Examples
/**
```rust
use geometry_tools::vectors::calculate_tangent_w;

# let tangent = glam::Vec3A::ZERO;
# let bitangent = glam::Vec3A::ZERO;
# let normal = glam::Vec3A::ZERO;
let tangent_w = calculate_tangent_w(tangent, bitangent, normal);

// The bitangent can be generated from the tangent and normal vector.
// This step is often done by shader code for the GPU.
let bitangent = normal.cross(tangent) * tangent_w;
```
*/
#[inline]
pub fn calculate_tangent_w(tangent: Vec3A, bitangent: Vec3A, normal: Vec3A) -> f32 {
    // 0.0 should stil return 1.0 to avoid generating black bitangents.
    if tangent.cross(bitangent).dot(normal) >= 0.0 {
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
            calculate_tangents_bitangents(&positions, &normals, &uvs, &[0u16, 1u16, 2u16]).unwrap();

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
        for bitangent in bitangents {
            assert_relative_eq!(1.0, bitangent.x, epsilon = EPSILON);
            assert_relative_eq!(0.0, bitangent.y, epsilon = EPSILON);
            assert_relative_eq!(0.0, bitangent.z, epsilon = EPSILON);
        }
    }

    #[test]
    fn triangle_list_single_triangle_with_w() {
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

        let tangents = calculate_tangents(&positions, &normals, &uvs, &[0u16, 1u16, 2u16]).unwrap();
        let bitangents: Vec<Vec3A> = tangents
            .iter()
            .zip(normals.iter())
            .map(|(t, n)| Vec3A::from_vec4(*t).cross(*n) * t.w)
            .collect();

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
        )
        .unwrap();

        assert_eq!(24, tangents.len());
        assert_eq!(24, bitangents.len());

        for (tangent, bitangent) in tangents.iter().zip(bitangents) {
            assert_relative_eq!(1.0, tangent.length(), epsilon = EPSILON);
            assert_relative_eq!(1.0, bitangent.length(), epsilon = EPSILON);
            assert!(is_good_tangent_bitangent(tangent, &bitangent));
        }
    }

    #[test]
    fn triangle_list_not_enough_indices() {
        let positions = vec![Vec3A::ZERO; 5];
        let normals = vec![Vec3A::ZERO; 5];
        let uvs = vec![Vec2::ZERO; 5];
        let indices = vec![0, 1, 2, 3, 4];

        match calculate_tangents_bitangents(&positions, &normals, &uvs, &indices) {
            Err(TangentBitangentError::InvalidIndexCont { index_count }) => {
                assert_eq!(5, index_count)
            }
            _ => panic!("Unexpected variant"),
        };
    }

    #[test]
    fn triangle_list_no_vertices() {
        let (tangents, bitangents) =
            calculate_tangents_bitangents::<Vec3A, Vec3A, u32>(&[], &[], &[], &[]).unwrap();

        assert!(tangents.is_empty());
        assert!(bitangents.is_empty());
    }

    #[test]
    #[should_panic]
    fn triangle_list_incorrect_normals_count() {
        match calculate_tangents_bitangents::<Vec3A, _, u32>(&[], &[Vec3A::ZERO], &[], &[]) {
            Err(TangentBitangentError::AttributeCountMismatch {
                position_count,
                normal_count,
                uv_count,
            }) => {
                assert_eq!(1, position_count);
                assert_eq!(0, normal_count);
                assert_eq!(0, uv_count);
            }
            _ => panic!("Unexpected variant"),
        };
    }

    #[test]
    fn triangle_list_incorrect_uvs_count() {
        match calculate_tangents_bitangents::<Vec3A, _, u32>(
            &[],
            &[Vec3A::ZERO],
            &[Vec2::ZERO],
            &[],
        ) {
            Err(TangentBitangentError::AttributeCountMismatch {
                position_count,
                normal_count,
                uv_count,
            }) => {
                assert_eq!(0, position_count);
                assert_eq!(1, normal_count);
                assert_eq!(1, uv_count);
            }
            _ => panic!("Unexpected variant"),
        };
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
        let w = calculate_tangent_w(tangent, bitangent, normal);
        assert_eq!(-1.0, w);
    }

    #[test]
    fn tangent_w_should_not_flip() {
        // cross(tangent, bitangent) is in the same direction as the normal.
        // This occurs on the side without mirrored UVs.
        let tangent = Vec3A::new(1.0, 0.0, 0.0);
        let bitangent = Vec3A::new(0.0, 1.0, 0.0);
        let normal = Vec3A::new(0.0, 0.0, 1.0);
        let w = calculate_tangent_w(tangent, bitangent, normal);
        assert_eq!(1.0, w);
    }

    #[test]
    fn tangent_w_should_not_be_zero() {
        // cross(tangent, bitangent) is orthogonal to the normal.
        let tangent = Vec3A::new(1.0, 0.0, 0.0);
        let bitangent = Vec3A::new(0.0, 1.0, 0.0);
        let normal = Vec3A::new(1.0, 0.0, 0.0);
        let w = calculate_tangent_w(tangent, bitangent, normal);
        assert_eq!(1.0, w);
    }
}
