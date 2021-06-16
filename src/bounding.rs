use glam::Vec3A;

/// Calculates a bounding sphere of the form `(center, radius)` that containts all the specified points.
/// The returned result will contain all points but may be larger than the optimal solution.
/// ```rust
/// use geometry_tools::calculate_bounding_sphere_from_points;
/// use glam::Vec3A;
///
/// let points = vec![
///     Vec3A::new(0f32, -1f32, -0f32),
///     Vec3A::new(0f32,  0f32,  0f32),
///     Vec3A::new(0f32,  1f32,  0f32),
/// ];
///
/// let (center, radius) = calculate_bounding_sphere_from_points(&points);
/// assert_eq!(Vec3A::ZERO, center);
/// assert_eq!(1f32, radius);
/// ```
/// If `points` is empty, the center and radius will both be zero.
/// ```rust
/// # use geometry_tools::calculate_bounding_sphere_from_points;
/// # use glam::Vec3A;
/// let bounding_sphere = calculate_bounding_sphere_from_points(&[]);
/// assert_eq!((Vec3A::ZERO, 0f32), bounding_sphere);
/// ```
pub fn calculate_bounding_sphere_from_points(points: &[Vec3A]) -> (Vec3A, f32) {
    if points.is_empty() {
        return (Vec3A::ZERO, 0f32);
    }

    // It's possible to optimize the center iteratively at the cost of performance.
    // Use the simple approach of averaging the points as the center.
    let center: Vec3A = points.iter().sum::<Vec3A>() / points.len() as f32;

    // Find the smallest radius that contains all points.
    let mut radius_squared = 0f32;
    for length_squared in points.iter().map(|p| p.distance_squared(center)) {
        if length_squared > radius_squared {
            radius_squared = length_squared;
        }
    }

    (center, radius_squared.sqrt())
}

/// Calculates an axis-aligned bounding box (abbreviated aabb) of the form `(min_xyz, max_xyz)` containing all the specified points.
/// ```rust
/// use geometry_tools::calculate_aabb_from_points;
/// use glam::Vec3A;
///
/// let (min, max) = calculate_aabb_from_points(&[
///     Vec3A::new(-1f32,  1f32,  2f32),
///     Vec3A::new( 0f32,  2f32,  1f32),
///     Vec3A::new( 2f32, -1f32, -1f32),
/// ]);
/// assert_eq!(min, Vec3A::new(-1f32, -1f32, -1f32));
/// assert_eq!(max, Vec3A::new( 2f32,  2f32,  2f32));
/// ```
/// If `points` is empty, both `min_xyz` and `max_xyz` will be zero.
/// ```rust
/// # use geometry_tools::calculate_aabb_from_points;
/// # use glam::Vec3A;
/// let aabb = calculate_aabb_from_points(&[]);
/// assert_eq!((Vec3A::ZERO, Vec3A::ZERO), aabb);
/// ```
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
    use std::ops::Sub;

    use super::*;

    fn sphere_contains_points(points: &[Vec3A], sphere: &(Vec3A, f32)) -> bool {
        for point in points {
            if point.sub(sphere.0).length() > sphere.1 {
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
}