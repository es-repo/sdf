use super::CameraFrame;
use crate::geometry::{Quat, Vec3};
const DEFAULT_FORWARD: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 1.0 };
const DEFAULT_RIGHT: Vec3 = Vec3 { x: 1.0, y: 0.0, z: 0.0 };
const DEFAULT_UP: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };

pub struct Camera {
    /// World-space camera position
    pub position: Vec3,

    /// Camera orientation
    rotation: Quat,

    /// Half width of the camera projection plane at unit distance.
    half_width: f32,

    /// Half height of the camera projection plane at unit distance.
    half_height: f32,
}

impl Camera {
    pub fn new(fov_y: f32, aspect: f32) -> Self {
        let half_height = (fov_y * 0.5).tan();
        let half_width = aspect * half_height;

        Self {
            position: Vec3::zero(),
            rotation: Quat::identity(),
            half_width,
            half_height,
        }
    }

    pub fn prepare_frame(&self) -> CameraFrame {
        CameraFrame::new(
            self.position,
            self.forward(),
            self.right(),
            self.up(),
            self.half_width,
            self.half_height,
        )
    }

    fn forward(&self) -> Vec3 {
        self.rotation * DEFAULT_FORWARD
    }

    fn right(&self) -> Vec3 {
        self.rotation * DEFAULT_RIGHT
    }

    fn up(&self) -> Vec3 {
        self.rotation * DEFAULT_UP
    }
}
