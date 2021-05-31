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

    crate::update_smooth_normals(pos, nrm, indices);
}
