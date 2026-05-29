use super::{CameraBasis, CameraFrame};
use crate::geometry::{Quat, Vec3};

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
            CameraBasis::DEFAULT.rotated(self.rotation),
            self.half_width,
            self.half_height,
        )
    }

    /// Replaces the camera orientation with the given rotation.
    ///
    /// Use this when the full target orientation is known, for example an FPS
    /// camera built from stored yaw and pitch angles.
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation.normalize();
    }

    /// Applies an additional rotation relative to the current orientation.
    ///
    /// Use this when input produces a rotation delta, for example flight-style
    /// camera controls where each mouse movement rotates from the current pose.
    pub fn rotate_by(&mut self, rotation: Quat) {
        self.set_rotation(rotation * self.rotation);
    }
}
