use glam::Vec3A;

/// Calculates smooth per-vertex normals by calculating normals for each face and averaging over the vertices.
/// `indices` is assumed to contain triangle indices for `positions`, so `indices.len()` should be a multiple of 3.
/// If either of `positions` or `indices` is empty, the result is empty.
pub fn calculate_smooth_normals(positions: &[Vec3A], indices: &[i32]) -> Vec<Vec3A> {
    if positions.is_empty() || indices.is_empty() {
        return Vec::new();
    }

    let mut normals = vec![Vec3A::ZERO; positions.len()];
    update_smooth_normals(positions, &mut normals, indices);
    normals
}

// Use an existing piece of memory for the result to make FFI easier.
// This allows another language such as C# to manage its own memory.
pub(crate) fn update_smooth_normals(positions: &[Vec3A], normals: &mut [Vec3A], indices: &[i32]) {
    for i in (0..indices.len()).step_by(3) {
        let i0 = indices[i + 0] as usize;
        let i1 = indices[i + 1] as usize;
        let i2 = indices[i + 2] as usize;

        let normal = calculate_normal(&positions[i0], &positions[i1], &positions[i2]);
        normals[i0] += normal;
        normals[i1] += normal;
        normals[i2] += normal;
    }

    for i in 0..normals.len() {
        normals[i] = normals[i].normalize();
    }
}

fn calculate_normal(v1: &Vec3A, v2: &Vec3A, v3: &Vec3A) -> Vec3A {
    let u = *v2 - *v1;
    let v = *v3 - *v1;
    u.cross(v)
}

pub mod ffi {
    use super::*;

    #[no_mangle]
    pub extern "C" fn calculate_smooth_normals(
        pos: *const glam::Vec3A,
        nrm: *mut glam::Vec3A,
        vec_len: i32,
        indices: *const i32,
        index_len: i32,
    ) {
        let pos = unsafe { std::slice::from_raw_parts(pos, vec_len as usize) };
        let nrm = unsafe { std::slice::from_raw_parts_mut(nrm, vec_len as usize) };
        let indices = unsafe { std::slice::from_raw_parts(indices, index_len as usize) };

        update_smooth_normals(pos, nrm, indices);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::relative_eq;

    #[test]
    fn positive_normal() {
        // Vertices facing the camera should be in counter-clockwise order.
        let v1 = Vec3A::new(-5f32, 5f32, 1f32);
        let v2 = Vec3A::new(-5f32, 0f32, 1f32);
        let v3 = Vec3A::new(0f32, 0f32, 1f32);
        let normal = calculate_normal(&v1, &v2, &v3).normalize();

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
        let normal = calculate_normal(&v3, &v2, &v1).normalize();

        assert_eq!(0f32, normal.x);
        assert_eq!(0f32, normal.y);
        assert_eq!(-1f32, normal.z);
    }

    #[test]
    fn smooth_normals_no_points_no_indices() {
        let normals = calculate_smooth_normals(&[], &[]);
        assert!(normals.is_empty());
    }

    #[test]
    fn smooth_normals_no_points() {
        let normals = calculate_smooth_normals(&[], &[0, 1, 2]);
        assert!(normals.is_empty());
    }

    #[test]
    fn smooth_normals_no_indices() {
        let points = vec![
            Vec3A::new(1f32, 0f32, 0f32),
            Vec3A::new(0f32, 1f32, 0f32),
            Vec3A::new(0f32, 0f32, 1f32),
        ];

        let normals = calculate_smooth_normals(&points, &[]);
        assert!(normals.is_empty());
    }

    #[test]
    fn smooth_normals_three_points() {
        let points = vec![
            Vec3A::new(1f32, 0f32, 0f32),
            Vec3A::new(0f32, 1f32, 0f32),
            Vec3A::new(0f32, 0f32, 1f32),
        ];

        let normals = calculate_smooth_normals(&points, &[0, 1, 2]);

        // Ensure vectors are normalized.
        let delta = 0.0001f32;
        assert!(relative_eq!(1f32, normals[0].length(), epsilon = delta));
        assert!(relative_eq!(1f32, normals[1].length(), epsilon = delta));
        assert!(relative_eq!(1f32, normals[2].length(), epsilon = delta));
    }

    #[test]
    fn smooth_normals_ffi() {
        let pos = [Vec3A::ONE, Vec3A::ONE];
        let mut nrm = [Vec3A::ONE, Vec3A::ONE];
        let indices = [0, 1, 0, 1, 0, 1, 1, 1, 0];
        ffi::calculate_smooth_normals(
            pos.as_ptr(),
            nrm.as_mut_ptr(),
            pos.len() as i32,
            indices.as_ptr(),
            indices.len() as i32,
        );
        assert_eq!(nrm[0], Vec3A::ONE.normalize());
        assert_eq!(nrm[1], Vec3A::ONE.normalize());
    }
}
