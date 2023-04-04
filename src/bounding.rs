//! Functions for calculating bounding spheres and axis-aligned bounding boxes.

use glam::Vec3A;

// TODO: Use a special struct for the result?
// ex: BoundingSphere::from_points

/// Calculates a bounding sphere of the form `(center, radius)` that contains all the specified points.
/// The returned result may be larger than the optimal solution.
/// # Examples
/**
```rust
use geometry_tools::bounding::calculate_bounding_sphere_from_points;
use glam::Vec3A;

let points = vec![
    Vec3A::new(0f32, -1f32, 0f32),
    Vec3A::new(0f32,  0f32, 0f32),
    Vec3A::new(0f32,  1f32, 0f32),
];

let (center, radius) = calculate_bounding_sphere_from_points(&points);
assert_eq!(Vec3A::ZERO, center);
assert_eq!(1f32, radius);
```
 */
/// If `points` is empty, the center and radius will both be zero.
/**
```rust
# use geometry_tools::bounding::calculate_bounding_sphere_from_points;
# use glam::Vec3A;
let bounding_sphere = calculate_bounding_sphere_from_points(&[]);
assert_eq!((Vec3A::ZERO, 0f32), bounding_sphere);
```
 */
pub fn calculate_bounding_sphere_from_points(points: &[Vec3A]) -> (Vec3A, f32) {
    if points.is_empty() {
        return (Vec3A::ZERO, 0f32);
    }

    // It's possible to optimize the center iteratively at the cost of performance.
    // Use the simple approach of averaging the points as the center.
    let center: Vec3A = points.iter().sum::<Vec3A>() / points.len() as f32;

    // Find the smallest radius that contains all points given a center.
    let mut radius_squared = 0f32;
    for length_squared in points.iter().map(|p| p.distance_squared(center)) {
        if length_squared > radius_squared {
            radius_squared = length_squared;
        }
    }

    (center, radius_squared.sqrt())
}

/// Calculates a bounding sphere of the form `(center, radius)` that contains all the specified bounding spheres.
/// The returned result may be larger than the optimal solution.
///
/// # Examples
/**
```rust
use geometry_tools::bounding::calculate_bounding_sphere_from_spheres;
use glam::Vec3A;

let spheres = vec![
    (Vec3A::new(0f32, -1f32, 0f32), 1.0),
    (Vec3A::new(0f32,  0f32, 0f32), 1.0),
    (Vec3A::new(0f32,  1f32, 0f32), 1.0),
];

let (center, radius) = calculate_bounding_sphere_from_spheres(&spheres);
assert_eq!(Vec3A::ZERO, center);
assert_eq!(2f32, radius);
```
 */
/// If `spheres` is empty, the center and radius will both be zero.
/**
```rust
# use geometry_tools::bounding::calculate_bounding_sphere_from_spheres;
# use glam::Vec3A;
let bounding_sphere = calculate_bounding_sphere_from_spheres(&[]);
assert_eq!((Vec3A::ZERO, 0f32), bounding_sphere);
```
 */
pub fn calculate_bounding_sphere_from_spheres(spheres: &[(Vec3A, f32)]) -> (Vec3A, f32) {
    if spheres.is_empty() {
        return (Vec3A::ZERO, 0f32);
    }

    // Use the simple approach of averaging the points as the center.
    let center: Vec3A = spheres.iter().map(|s| &s.0).sum::<Vec3A>() / spheres.len() as f32;

    // Find the smallest radius that contains all spheres given a center.
    // This is a simple extension of testing for sphere-sphere intersection.
    // We want the distance between centers to not exceed the sum of the radii.
    // We iteratively increase the radius for each sphere.
    // TODO: Share code with above?
    let mut radius = 0f32;
    for length in spheres
        .iter()
        .map(|(center2, radius2)| center2.distance(center) + radius2)
    {
        if length > radius {
            radius = length;
        }
    }

    (center, radius)
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
let aabb = calculate_aabb_from_points(&[]);
assert_eq!((Vec3A::ZERO, Vec3A::ZERO), aabb);
```
 */
pub fn calculate_aabb_from_points(points: &[Vec3A]) -> (Vec3A, Vec3A) {
    match points.first() {
        Some(p) => {
            let mut min_xyz = *p;
            let mut max_xyz = *p;

            for point in points {
                min_xyz = min_xyz.min(*point);
                max_xyz = max_xyz.max(*point);
            }

            (min_xyz, max_xyz)
        }
        None => (Vec3A::ZERO, Vec3A::ZERO),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sphere_contains_points(points: &[Vec3A], sphere: &(Vec3A, f32)) -> bool {
        let (center, radius) = sphere;
        for point in points {
            if point.distance(*center) > *radius {
                return false;
            }
        }

        true
    }

    fn sphere_contains_spheres(spheres: &[(Vec3A, f32)], sphere: &(Vec3A, f32)) -> bool {
        // Two spheres intersect if the distance between their centers
        // is less than the sum of their radii.
        let (center, radius) = sphere;
        for (center2, radius2) in spheres {
            if center.distance(*center2) > radius + radius2 {
                return false;
            }
        }

        true
    }

    #[test]
    fn aabb_no_points() {
        let aabb = calculate_aabb_from_points(&[]);
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
        let bounding_sphere = calculate_bounding_sphere_from_points(&[]);
        assert_eq!((Vec3A::ZERO, 0f32), bounding_sphere);
    }

    #[test]
    fn sphere_single_point() {
        let points = vec![Vec3A::new(0.5f32, -0.5f32, -0.5f32)];

        let bounding_sphere = calculate_bounding_sphere_from_points(&points);
        assert!(sphere_contains_points(&points, &bounding_sphere));
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
        assert!(sphere_contains_points(&points, &bounding_sphere));
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
        assert!(sphere_contains_points(&points, &bounding_sphere));
    }

    #[test]
    fn sphere_no_spheres() {
        let bounding_sphere = calculate_bounding_sphere_from_spheres(&[]);
        assert_eq!((Vec3A::ZERO, 0f32), bounding_sphere);
    }

    #[test]
    fn sphere_single_sphere() {
        let spheres = vec![(Vec3A::new(0.1, 0.2, 0.3), 1.5)];

        let bounding_sphere = calculate_bounding_sphere_from_spheres(&spheres);
        assert!(sphere_contains_spheres(&spheres, &bounding_sphere));
    }

    #[test]
    fn sphere_multiple_spheres() {
        let spheres = vec![
            (Vec3A::new(0.1, 0.2, 0.3), 1.5),
            (Vec3A::new(-1.0, 5.0, 2.5), 3.2),
            (Vec3A::new(4.0, 5.0, 6.0), 10.5),
        ];

        let bounding_sphere = calculate_bounding_sphere_from_spheres(&spheres);
        assert!(sphere_contains_spheres(&spheres, &bounding_sphere));
    }
}
