use super::Camera;
use super::CameraBasis;
use crate::geometry::{Quat, Vec3};
use crate::input::InputState;
use std::f32::consts::FRAC_PI_2;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

const MAX_PITCH: f32 = FRAC_PI_2 - 0.001;

pub struct CameraControls {
    pub forward: KeyCode,
    pub backward: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub look_button: Option<MouseButton>,
}

impl Default for CameraControls {
    fn default() -> Self {
        Self {
            forward: KeyCode::KeyW,
            backward: KeyCode::KeyS,
            left: KeyCode::KeyA,
            right: KeyCode::KeyD,
            look_button: Some(MouseButton::Left),
        }
    }
}

pub struct CameraController {
    pub controls: CameraControls,
    pub speed: f32,
    pub look_sensitivity: f32,
    pub scroll_move_sensitivity: f32,
    yaw: f32,
    pitch: f32,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            controls: CameraControls::default(),
            speed,
            look_sensitivity: 0.003,
            scroll_move_sensitivity: 0.005,
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, delta_time: f32, input: &InputState) {
        self.update_camera_rotation(camera, input);

        let camera_frame = camera.prepare_frame();
        let mut direction = Vec3::zero();

        if input.is_key_pressed(self.controls.forward) {
            direction = direction + camera_frame.basis.forward;
        }

        if input.is_key_pressed(self.controls.backward) {
            direction = direction - camera_frame.basis.forward;
        }

        if input.is_key_pressed(self.controls.left) {
            direction = direction - camera_frame.basis.right;
        }

        if input.is_key_pressed(self.controls.right) {
            direction = direction + camera_frame.basis.right;
        }

        if direction.len_squared() > 0.0 {
            camera.position = camera.position + direction.normalize() * self.speed * delta_time;
        }

        let scroll_delta = input.scroll_delta();
        let scroll_direction = camera_frame.basis.right * -scroll_delta.x + camera_frame.basis.forward * scroll_delta.y;
        if scroll_direction.len_squared() > 0.0 {
            camera.position = camera.position + scroll_direction * self.scroll_move_sensitivity * self.speed;
        }
    }

    fn update_camera_rotation(&mut self, camera: &mut Camera, input: &InputState) {
        if !self.is_look_active(input) {
            return;
        }

        let mouse_delta = input.mouse_delta();

        if mouse_delta.len_squared() == 0.0 {
            return;
        }

        self.yaw += mouse_delta.x * self.look_sensitivity;
        self.pitch = (self.pitch + mouse_delta.y * self.look_sensitivity).clamp(-MAX_PITCH, MAX_PITCH);

        let yaw = Quat::from_axis_angle(CameraBasis::DEFAULT.up, self.yaw);
        let pitch = Quat::from_axis_angle(CameraBasis::DEFAULT.right, self.pitch);

        camera.set_rotation(yaw * pitch);
    }

    fn is_look_active(&self, input: &InputState) -> bool {
        match self.controls.look_button {
            Some(button) => input.is_mouse_button_pressed(button),
            None => true,
        }
    }
}
