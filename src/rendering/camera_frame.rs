use super::{CameraBasis, Ray};
use crate::geometry::{Vec2, Vec3};

pub struct CameraFrame {
    /// World-space camera position
    pub position: Vec3,

    pub basis: CameraBasis,
    half_width: f32,
    half_height: f32,
}

impl CameraFrame {
    pub(super) fn new(position: Vec3, basis: CameraBasis, half_width: f32, half_height: f32) -> Self {
        Self {
            position,
            basis,
            half_width,
            half_height,
        }
    }

    /// Returns a world-space ray for a normalized screen coordinate.
    /// The coordinate is expected before aspect correction, with `(0, 0)` at the screen center.
    /// The virtual projection plane is one unit in front of the camera.
    pub fn ray(&self, coord: Vec2) -> Ray {
        Ray::new(self.position, self.ray_direction(coord))
    }

    /// Returns only the normalized world-space direction for the ray through the coordinate.
    pub fn ray_direction(&self, coord: Vec2) -> Vec3 {
        let direction = self.basis.forward
            + self.basis.right * coord.x * self.half_width
            + self.basis.up * coord.y * self.half_height;
        direction.normalize()
    }
}
