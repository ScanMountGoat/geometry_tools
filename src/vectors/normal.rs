use glam::Vec3A;

/// Calculates smooth per-vertex normals by by averaging over the vertices in each face.
/// `indices` is assumed to contain triangle indices for `positions`, so `indices.len()` should be a multiple of 3.
/// If either of `positions` or `indices` is empty, the result is empty.
pub fn calculate_smooth_normals<P>(positions: &[P], indices: &[u32]) -> Vec<Vec3A>
where
    P: Into<Vec3A> + Copy,
{
    if positions.is_empty() || indices.is_empty() {
        return Vec::new();
    }

    let mut normals = vec![Vec3A::ZERO; positions.len()];
    update_smooth_normals(positions, &mut normals, indices);
    normals
}

// Use an existing piece of memory for the result to make FFI easier.
// This allows another language such as C# to manage its own memory.
fn update_smooth_normals<P>(positions: &[P], normals: &mut [Vec3A], indices: &[u32])
where
    P: Into<Vec3A> + Copy,
{
    for face in indices.chunks(3) {
        if let [v0, v1, v2] = face {
            let normal = calculate_normal(
                positions[*v0 as usize].into(),
                positions[*v1 as usize].into(),
                positions[*v2 as usize].into(),
            );
            normals[*v0 as usize] += normal;
            normals[*v1 as usize] += normal;
            normals[*v2 as usize] += normal;
        }
    }

    for normal in normals.iter_mut() {
        *normal = normal.normalize_or_zero();
    }
}

#[inline(always)]
fn calculate_normal(v1: Vec3A, v2: Vec3A, v3: Vec3A) -> Vec3A {
    let u = v2 - v1;
    let v = v3 - v1;
    u.cross(v)
}

pub mod ffi {
    use super::*;

    /// A wrapper for [calculate_smooth_normals](crate::vectors::calculate_smooth_normals).
    /// `indices` and `indices_length` define the collection of vertex indices.
    /// The function writes the resulting smooth normals to the first `pos_nrm_length` elements of `normals`.
    ///
    /// # Safety
    ///
    /// `positions` and `normals` must both have length `pos_nrm_length`.
    /// The memory layout of the `positions` and `normals` array should have the xyz values in the first three floats
    /// of each vector of four floats to ensure compatibility with the 16 byte alignment of the [Vec3A] type.
    ///
    /// Example: `x0 y0 z0 _ x1 y1 z1 _ x2 y2 z2 _ ...`
    ///
    /// The fourth value of each vector is included only for alignment purposes and does not affect the computation.
    /// This gives a required size of at least `pos_nrm_length * 16` bytes for both arrays.
    #[no_mangle]
    pub unsafe extern "C" fn calculate_smooth_normals(
        positions: *const glam::Vec3A,
        normals: *mut glam::Vec3A,
        pos_nrm_length: u32,
        indices: *const u32,
        indices_length: u32,
    ) {
        let pos = std::slice::from_raw_parts(positions, pos_nrm_length as usize);
        let nrm = std::slice::from_raw_parts_mut(normals, pos_nrm_length as usize);
        let indices = std::slice::from_raw_parts(indices, indices_length as usize);

        update_smooth_normals(pos, nrm, indices);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    const EPSILON: f32 = 0.0001;

    #[test]
    fn positive_normal() {
        // Vertices facing the camera should be in counter-clockwise order.
        let v1 = Vec3A::new(-5f32, 5f32, 1f32);
        let v2 = Vec3A::new(-5f32, 0f32, 1f32);
        let v3 = Vec3A::new(0f32, 0f32, 1f32);
        let normal = calculate_normal(v1, v2, v3).normalize();

        assert_eq!(0f32, normal.x);
        assert_eq!(0f32, normal.y);
        assert_eq!(1f32, normal.z);
    }

    #[test]
    fn negative_normal() {
        // Vertices facing the camera in clockwise order.
        let v1 = Vec3A::new(-5f32, 5f32, 1f32);
        let v2 = Vec3A::new(-5f32, 0f32, 1f32);
        let v3 = Vec3A::new(0f32, 0f32, 1f32);
        let normal = calculate_normal(v3, v2, v1).normalize();

        assert_eq!(0f32, normal.x);
        assert_eq!(0f32, normal.y);
        assert_eq!(-1f32, normal.z);
    }

    #[test]
    fn smooth_normals_no_points_no_indices() {
        let normals = calculate_smooth_normals::<Vec3A>(&[], &[]);
        assert!(normals.is_empty());
    }

    #[test]
    fn smooth_normals_no_points() {
        let normals = calculate_smooth_normals::<Vec3A>(&[], &[0, 1, 2]);
        assert!(normals.is_empty());
    }

    #[test]
    fn smooth_normals_no_indices() {
        let points = vec![
            Vec3A::new(1f32, 0f32, 0f32),
            Vec3A::new(0f32, 1f32, 0f32),
            Vec3A::new(0f32, 0f32, 1f32),
        ];

        let normals = calculate_smooth_normals::<Vec3A>(&points, &[]);
        assert!(normals.is_empty());
    }

    #[test]
    fn smooth_normals_three_points() {
        let points = vec![
            Vec3A::new(1f32, 0f32, 0f32),
            Vec3A::new(0f32, 1f32, 0f32),
            Vec3A::new(0f32, 0f32, 1f32),
        ];

        let normals = calculate_smooth_normals::<Vec3A>(&points, &[0, 1, 2]);

        // Ensure vectors are normalized.
        assert_relative_eq!(1f32, normals[0].length(), epsilon = EPSILON);
        assert_relative_eq!(1f32, normals[1].length(), epsilon = EPSILON);
        assert_relative_eq!(1f32, normals[2].length(), epsilon = EPSILON);
    }

    #[test]
    fn smooth_normals_zero_normal() {
        let points = vec![Vec3A::X, Vec3A::X, Vec3A::X];

        let normals = calculate_smooth_normals::<Vec3A>(&points, &[0, 1, 2]);

        // Check for divide by 0 when normalizing.
        for normal in normals {
            assert_relative_eq!(0.0, normal.x, epsilon = EPSILON);
            assert_relative_eq!(0.0, normal.y, epsilon = EPSILON);
            assert_relative_eq!(0.0, normal.z, epsilon = EPSILON);
        }
    }

    #[test]
    fn smooth_normals_ffi() {
        let pos = [Vec3A::ONE, Vec3A::ONE];
        let mut nrm = [Vec3A::ONE, Vec3A::ONE];
        let indices = [0, 1, 0, 1, 0, 1, 1, 1, 0];
        unsafe {
            ffi::calculate_smooth_normals(
                pos.as_ptr(),
                nrm.as_mut_ptr(),
                pos.len() as u32,
                indices.as_ptr(),
                indices.len() as u32,
            );
        }
        assert_eq!(nrm[0], Vec3A::ONE.normalize());
        assert_eq!(nrm[1], Vec3A::ONE.normalize());
    }
}
