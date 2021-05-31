use std::ops::Sub;

use glam::Vec3A;

/// Calculate a bounding sphere that containts all the specified points.
/// The returned result will contain all points but be larger than the optimal solution.
pub fn calculate_bounding_sphere_from_points(points: &[Vec3A]) -> (Vec3A, f32) {
    if points.is_empty() {
        return (Vec3A::ZERO, 0f32);
    }

    // It's possible to optimize the center iteratively at the cost of performance.
    // Use the simple approach of averaging the points as the center.
    let center: Vec3A = points.iter().sum::<Vec3A>() / points.len() as f32;

    // Find the smallest radius that contains all points.
    let mut radius_squared = 0f32;
    for length in points.iter().map(|p| p.sub(center).length_squared()) {
        if length > radius_squared {
            radius_squared = length;
        }
    }

    (center, radius_squared.sqrt())
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
