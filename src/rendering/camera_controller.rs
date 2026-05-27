use super::Camera;
use crate::geometry::Vec3;
use crate::input::InputState;
use winit::keyboard::KeyCode;

pub struct CameraControls {
    pub forward: KeyCode,
    pub backward: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
}

impl Default for CameraControls {
    fn default() -> Self {
        Self {
            forward: KeyCode::KeyW,
            backward: KeyCode::KeyS,
            left: KeyCode::KeyA,
            right: KeyCode::KeyD,
        }
    }
}

pub struct CameraController {
    pub controls: CameraControls,
    pub speed: f32,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            controls: CameraControls::default(),
            speed,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera, delta_time: f32, input: &InputState) {
        let camera_frame = camera.prepare_frame();
        let mut direction = Vec3::zero();

        if input.is_key_pressed(self.controls.forward) {
            direction = direction + camera_frame.forward;
        }

        if input.is_key_pressed(self.controls.backward) {
            direction = direction - camera_frame.forward;
        }

        if input.is_key_pressed(self.controls.left) {
            direction = direction - camera_frame.right;
        }

        if input.is_key_pressed(self.controls.right) {
            direction = direction + camera_frame.right;
        }

        if direction.len_squared() > 0.0 {
            camera.position = camera.position + direction.normalize() * self.speed * delta_time;
        }
    }
}
