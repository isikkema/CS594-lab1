use glam::{Mat4, Vec3};

pub struct Camera {
    position: Vec3,
    target: Vec3,
    up: Vec3,
}

impl Camera {
    pub fn new(position: Vec3, target: Vec3, up: Vec3) -> Self {
        Camera {
            position,
            target,
            up,
        }
    }

    pub fn compute_view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }
}
