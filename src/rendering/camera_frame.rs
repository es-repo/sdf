use crate::geometry::{Vec2, Vec3};

pub struct CameraFrame {
    /// World-space camera position
    pub position: Vec3<f32>,

    forward: Vec3<f32>,
    right: Vec3<f32>,
    up: Vec3<f32>,
    half_width: f32,
    half_height: f32,
}

impl CameraFrame {
    pub(super) fn new(
        position: Vec3<f32>,
        forward: Vec3<f32>,
        right: Vec3<f32>,
        up: Vec3<f32>,
        half_width: f32,
        half_height: f32,
    ) -> Self {
        Self {
            position,
            forward,
            right,
            up,
            half_width,
            half_height,
        }
    }

    /// Returns a normalized world-space ray direction for a normalized screen coordinate.
    /// The coordinate is expected before aspect correction, with `(0, 0)` at the screen center.
    /// The virtual projection plane is one unit in front of the camera.
    pub fn ray_direction(&self, coord: Vec2<f32>) -> Vec3<f32> {
        let direction = self.forward + self.right * coord.x * self.half_width + self.up * coord.y * self.half_height;
        direction / direction.len()
    }
}
