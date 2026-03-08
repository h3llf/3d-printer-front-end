use glam::{Mat4, Vec3};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Camera {
    pub view_proj : [[f32; 4]; 4],
}

impl Camera {
    pub fn build_camera_matrix(
        eye : Vec3,
        target : Vec3,
        aspect : f32 ) -> Self
    {
        let view = Mat4::look_at_rh (eye, target, Vec3::Y);

        let proj = Mat4::perspective_rh(90.0_f32.to_radians(), aspect, 0.1, 100.0);

        Camera{view_proj : (proj * view).to_cols_array_2d()}
    }
}
