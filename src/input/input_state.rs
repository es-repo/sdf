use crate::geometry::Vec2;
use std::collections::HashSet;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

pub struct InputState {
    pressed_keys: HashSet<KeyCode>,
    pressed_mouse_buttons: HashSet<MouseButton>,
    mouse_position: Option<Vec2>,
    mouse_delta: Vec2,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            pressed_mouse_buttons: HashSet::new(),
            mouse_position: None,
            mouse_delta: Vec2::zero(),
        }
    }
}

impl InputState {
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn set_key_pressed(&mut self, key: KeyCode, pressed: bool) {
        if pressed {
            self.pressed_keys.insert(key);
        } else {
            self.pressed_keys.remove(&key);
        }
    }

    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_mouse_buttons.contains(&button)
    }

    pub fn set_mouse_button_pressed(&mut self, button: MouseButton, pressed: bool) {
        if pressed {
            self.pressed_mouse_buttons.insert(button);
        } else {
            self.pressed_mouse_buttons.remove(&button);
        }
    }

    pub fn mouse_position(&self) -> Option<Vec2> {
        self.mouse_position
    }

    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }

    pub fn set_mouse_position(&mut self, position: Vec2) {
        if let Some(previous_position) = self.mouse_position {
            self.mouse_delta = self.mouse_delta + position - previous_position;
        }

        self.mouse_position = Some(position);
    }

    pub fn clear_mouse_position(&mut self) {
        self.mouse_position = None;
    }

    pub fn clear(&mut self) {
        self.pressed_keys.clear();
        self.pressed_mouse_buttons.clear();
        self.mouse_position = None;
        self.reset_frame_delta();
    }

    pub fn reset_frame_delta(&mut self) {
        self.mouse_delta = Vec2::zero();
    }
}
