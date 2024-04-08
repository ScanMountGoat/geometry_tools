//! Functions for calculating bounding spheres and axis-aligned bounding boxes.

use glam::{Vec3A, Vec4};

/// Calculates a bounding sphere of the form `(center, radius)` that contains all the specified points.
/// The returned result may be larger than the optimal solution.
/// # Examples
/**
```rust
use geometry_tools::bounding::calculate_bounding_sphere_from_points;
use glam::{Vec3, Vec3A, Vec4, Vec4Swizzles};

let points = vec![
    Vec3A::new(0f32, -1f32, 0f32),
    Vec3A::new(0f32,  0f32, 0f32),
    Vec3A::new(0f32,  1f32, 0f32),
];

let center_radius = calculate_bounding_sphere_from_points(&points);
assert_eq!(Vec3::ZERO, center_radius.xyz());
assert_eq!(1f32, center_radius.w);
```
 */
/// If `points` is empty, the center and radius will both be zero.
/**
```rust
# use geometry_tools::bounding::calculate_bounding_sphere_from_points;
# use glam::{Vec3A, Vec4};
let bounding_sphere = calculate_bounding_sphere_from_points::<Vec3A>(&[]);
assert_eq!(Vec4::ZERO, bounding_sphere);
```
 */
pub fn calculate_bounding_sphere_from_points<P>(points: &[P]) -> Vec4
where
    P: Into<Vec3A> + Copy,
{
    if points.is_empty() {
        return Vec4::ZERO;
    }

    // It's possible to optimize the center iteratively at the cost of performance.
    // Use the simple approach of averaging the points as the center.
    let center: Vec3A = points.iter().copied().map(Into::into).sum::<Vec3A>() / points.len() as f32;

    // Find the smallest radius that contains all points given a center.
    let mut radius_squared = 0f32;
    for length_squared in points.iter().map(|p| {
        let p: Vec3A = (*p).into();
        p.distance_squared(center)
    }) {
        if length_squared > radius_squared {
            radius_squared = length_squared;
        }
    }

    center.extend(radius_squared.sqrt())
}

/// Calculates a bounding sphere of the form `(center, radius)` that contains all the specified bounding spheres.
/// The returned result may be larger than the optimal solution.
///
/// # Examples
/**
```rust
use geometry_tools::bounding::calculate_bounding_sphere_from_spheres;
use glam::{Vec3, Vec3A, Vec4, Vec4Swizzles};

let spheres = vec![
    Vec4::new(0f32, -1f32, 0f32, 1.0),
    Vec4::new(0f32,  0f32, 0f32, 1.0),
    Vec4::new(0f32,  1f32, 0f32, 1.0),
];

let center_radius = calculate_bounding_sphere_from_spheres(&spheres);
assert_eq!(Vec3::ZERO, center_radius.xyz());
assert_eq!(2f32, center_radius.w);
```
 */
/// If `spheres` is empty, the center and radius will both be zero.
/**
```rust
# use geometry_tools::bounding::calculate_bounding_sphere_from_spheres;
# use glam::Vec4;
let bounding_sphere = calculate_bounding_sphere_from_spheres(&[]);
assert_eq!(Vec4::ZERO, bounding_sphere);
```
 */
pub fn calculate_bounding_sphere_from_spheres(spheres: &[Vec4]) -> Vec4 {
    if spheres.is_empty() {
        return Vec4::ZERO;
    }

    // Use the simple approach of averaging the points as the center.
    let center: Vec3A =
        spheres.iter().copied().map(Vec3A::from).sum::<Vec3A>() / spheres.len() as f32;

    // Find the smallest radius that contains all spheres given a center.
    // This is a simple extension of testing for sphere-sphere intersection.
    // We want the distance between centers to not exceed the sum of the radii.
    // We iteratively increase the radius for each sphere.
    let radius = spheres
        .iter()
        .map(|sphere2| Vec3A::from(*sphere2).distance(center) + sphere2.w)
        .reduce(f32::max)
        .unwrap_or_default();

    center.extend(radius)
}

/// Calculates an axis-aligned bounding box (abbreviated aabb) of the form `(min_xyz, max_xyz)` containing all the specified points.
/// # Examples
/**
```rust
use geometry_tools::bounding::calculate_aabb_from_points;
use glam::Vec3A;

let (min, max) = calculate_aabb_from_points(&[
    Vec3A::new( 0f32,  2f32,  1f32),
    Vec3A::new(-1f32,  1f32,  2f32),
    Vec3A::new( 2f32, -1f32, -1f32),
]);
assert_eq!(min, Vec3A::new(-1f32, -1f32, -1f32));
assert_eq!(max, Vec3A::new( 2f32,  2f32,  2f32));
```
*/
/// If `points` is empty, both `min_xyz` and `max_xyz` will be zero.
/**
```rust
# use geometry_tools::bounding::calculate_aabb_from_points;
# use glam::Vec3A;
let aabb = calculate_aabb_from_points::<Vec3A>(&[]);
assert_eq!((Vec3A::ZERO, Vec3A::ZERO), aabb);
```
 */
pub fn calculate_aabb_from_points<P>(points: &[P]) -> (Vec3A, Vec3A)
where
    P: Into<Vec3A> + Copy,
{
    match points.first().copied() {
        Some(p) => {
            let mut min_xyz: Vec3A = p.into();
            let mut max_xyz: Vec3A = p.into();

            for point in points {
                min_xyz = min_xyz.min((*point).into());
                max_xyz = max_xyz.max((*point).into());
            }

            (min_xyz, max_xyz)
        }
        None => (Vec3A::ZERO, Vec3A::ZERO),
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec4Swizzles;

    use super::*;

    fn sphere_contains_points(points: &[Vec3A], sphere: Vec4) -> bool {
        let center = sphere.xyz();
        let radius = sphere.w;

        for point in points {
            if point.distance(center.into()) > radius {
                return false;
            }
        }

        true
    }

    fn sphere_contains_spheres(spheres: &[Vec4], sphere: Vec4) -> bool {
        // Two spheres intersect if the distance between their centers
        // is less than the sum of their radii.
        let center = sphere.xyz();
        let radius = sphere.w;
        for sphere2 in spheres {
            let center2 = sphere2.xyz();
            let radius2 = sphere2.w;
            if center.distance(center2) > radius + radius2 {
                return false;
            }
        }

        true
    }

    #[test]
    fn aabb_no_points() {
        let aabb = calculate_aabb_from_points::<Vec3A>(&[]);
        assert_eq!((Vec3A::ZERO, Vec3A::ZERO), aabb);
    }

    #[test]
    fn aabb_single_point() {
        let aabb = calculate_aabb_from_points(&[Vec3A::new(0.5f32, 1.0f32, 2f32)]);
        assert_eq!(
            (
                Vec3A::new(0.5f32, 1.0f32, 2f32),
                Vec3A::new(0.5f32, 1.0f32, 2f32)
            ),
            aabb
        );
    }

    #[test]
    fn aabb_multiple_points() {
        let aabb = calculate_aabb_from_points(&[
            Vec3A::new(-1f32, 1f32, 2f32),
            Vec3A::new(0f32, 2f32, 1f32),
            Vec3A::new(2f32, -1f32, -1f32),
        ]);
        assert_eq!(
            (
                Vec3A::new(-1f32, -1f32, -1f32),
                Vec3A::new(2f32, 2f32, 2f32)
            ),
            aabb
        );
    }

    #[test]
    fn sphere_no_points() {
        let bounding_sphere = calculate_bounding_sphere_from_points::<Vec3A>(&[]);
        assert_eq!(Vec4::ZERO, bounding_sphere);
    }

    #[test]
    fn sphere_single_point() {
        let points = vec![Vec3A::new(0.5f32, -0.5f32, -0.5f32)];

        let bounding_sphere = calculate_bounding_sphere_from_points(&points);
        assert!(sphere_contains_points(&points, bounding_sphere));
    }

    #[test]
    fn sphere_rectangular_prism() {
        let points = vec![
            Vec3A::new(-10f32, -1f32, -1f32),
            Vec3A::new(-10f32, 1f32, -1f32),
            Vec3A::new(-10f32, -1f32, 1f32),
            Vec3A::new(-10f32, 1f32, 1f32),
            Vec3A::new(10f32, -1f32, -1f32),
            Vec3A::new(10f32, 1f32, -1f32),
            Vec3A::new(10f32, -1f32, 1f32),
            Vec3A::new(10f32, 1f32, 1f32),
        ];

        // Test an elongated prism.
        let bounding_sphere = calculate_bounding_sphere_from_points(&points);
        assert!(sphere_contains_points(&points, bounding_sphere));
    }

    #[test]
    fn sphere_unit_cube() {
        let points = vec![
            Vec3A::new(0.5f32, -0.5f32, -0.5f32),
            Vec3A::new(0.5f32, -0.5f32, 0.5f32),
            Vec3A::new(-0.5f32, -0.5f32, 0.5f32),
            Vec3A::new(-0.5f32, -0.5f32, -0.5f32),
            Vec3A::new(0.5f32, 0.5f32, -0.5f32),
            Vec3A::new(0.5f32, 0.5f32, 0.5f32),
            Vec3A::new(-0.5f32, 0.5f32, 0.5f32),
            Vec3A::new(-0.5f32, 0.5f32, -0.5f32),
        ];

        // Check that all the corners are contained in the sphere.
        let bounding_sphere = calculate_bounding_sphere_from_points(&points);
        assert!(sphere_contains_points(&points, bounding_sphere));
    }

    #[test]
    fn sphere_no_spheres() {
        let bounding_sphere = calculate_bounding_sphere_from_spheres(&[]);
        assert_eq!(Vec4::ZERO, bounding_sphere);
    }

    #[test]
    fn sphere_single_sphere() {
        let spheres = vec![Vec4::new(0.1, 0.2, 0.3, 1.5)];

        let bounding_sphere = calculate_bounding_sphere_from_spheres(&spheres);
        assert!(sphere_contains_spheres(&spheres, bounding_sphere));
    }

    #[test]
    fn sphere_multiple_spheres() {
        let spheres = vec![
            Vec4::new(0.1, 0.2, 0.3, 1.5),
            Vec4::new(-1.0, 5.0, 2.5, 3.2),
            Vec4::new(4.0, 5.0, 6.0, 10.5),
        ];

        let bounding_sphere = calculate_bounding_sphere_from_spheres(&spheres);
        assert!(sphere_contains_spheres(&spheres, bounding_sphere));
    }
}
