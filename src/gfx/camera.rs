use glam::{Mat4, Vec3, Vec2};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Camera {
    pub view_proj : [[f32; 4]; 4],
    pub position : [f32; 3],
    pub _padding : f32,
}

impl Camera {
    pub fn build_camera_matrix (
        eye : Vec3,
        target : Vec3,
        aspect : f32 ) -> Self
    {
        let view = Mat4::look_at_rh(eye, target, Vec3::Y);

        let proj = Mat4::perspective_rh(70.0_f32.to_radians(), aspect, 0.1, 100.0);

        Camera{
            view_proj : (proj * view).to_cols_array_2d(),
            position : eye.to_array(),
            _padding : 0.0,
        }
    }
}

#[derive(Default)]
pub struct OrbitCamera {
    // Mouse control
    sensitivity : f32,
    last_pos : Vec2,
    pub pressed : bool,
    pub just_pressed : bool,
    pub middle_pressed : bool,
    pub middle_just_pressed : bool,

    // Camera state
    pub zoom_factor : f32,
    target : Vec3,
    offset : Vec3,
}

const WORLD_UP : Vec3 = Vec3::Y;

// TODO: Pan control, change origin, ortho projection
impl OrbitCamera {
    pub fn new(start_x : f32, start_y : f32) -> Self {
        Self {
            sensitivity : 1.0,
            last_pos : Vec2{x : start_x, y : start_y},
            pressed : false,
            just_pressed : false,
            middle_pressed : false,
            middle_just_pressed : false,
            zoom_factor : 10.0,
            target : Vec3{x : 0.0, y : 0.0, z : 0.0},
            offset : Vec3{x : 1.0, y : 1.0, z : 1.0},
        }
    }

    pub fn update_mouse_pos(&mut self, x : f32, y : f32) {
        if self.pressed {
            let dx : f32 = (self.last_pos.x - x) * self.sensitivity;
            let dy : f32 = (self.last_pos.y - y) * self.sensitivity;
            self.last_pos = Vec2 { x, y };
            self.rotate_camera(dx, dy);
        } else if self.middle_pressed {
            let dx : f32 = (self.last_pos.x - x) * self.sensitivity;
            let dy : f32 = (self.last_pos.y - y) * self.sensitivity;
            self.last_pos = Vec2 { x, y };
            self.pan_camera(dx, dy);           
        }
    }

    fn rotate_camera(&mut self, dx : f32, dy : f32) {
        self.offset = self.offset.rotate_axis(WORLD_UP, dx.to_radians());
        let forward = (-self.offset).normalize();
        let mut right = forward.cross(WORLD_UP).normalize();
//        if right.length() < 0.00001 {
//            right = Vec3{x : 1.0, y : 0.0, z : 0.0};
//        }
       self.offset = self.offset.rotate_axis(right, dy.to_radians());
    }

    fn pan_camera(&mut self, dx : f32, dy : f32) {
        let forward : Vec3 = (-self.offset).normalize();
        let right : Vec3 = forward.cross(WORLD_UP).normalize();
        let up = right.cross(forward);

        let scale : f32 = (self.offset * self.zoom_factor).length() * 0.005;
        let pan : Vec3 = (-dx * right + dy * up) * scale;
        self.target += pan;
    }

    pub fn construct_camera(&self, aspect : f32) -> Camera {
        let camera_position = self.target + self.offset * self.zoom_factor;
        Camera::build_camera_matrix(camera_position, self.target, aspect)
    }

    pub fn reset_mouse_pos(&mut self, x : f32, y : f32) {
        self.last_pos.x = x;
        self.last_pos.y = y;
    }
}
