use crate::geometry::Vec2;
use std::collections::{HashMap, HashSet};
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

pub struct InputState {
    pressed_keys: HashSet<KeyCode>,
    pressed_mouse_buttons: HashSet<MouseButton>,
    pointer_position: Option<Vec2>,
    pointer_delta: Vec2,
    scroll_delta: Vec2,
    touch_positions: HashMap<u64, Vec2>,
    touch_look_active: bool,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            pressed_mouse_buttons: HashSet::new(),
            pointer_position: None,
            pointer_delta: Vec2::zero(),
            scroll_delta: Vec2::zero(),
            touch_positions: HashMap::new(),
            touch_look_active: false,
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

    pub fn pointer_delta(&self) -> Vec2 {
        self.pointer_delta
    }

    pub fn scroll_delta(&self) -> Vec2 {
        self.scroll_delta
    }

    pub fn set_pointer_position(&mut self, position: Vec2) {
        if let Some(previous_position) = self.pointer_position {
            self.pointer_delta = self.pointer_delta + position - previous_position;
        }

        self.pointer_position = Some(position);
    }

    pub fn add_scroll_delta(&mut self, delta: Vec2) {
        self.scroll_delta = self.scroll_delta + delta;
    }

    pub fn clear_pointer_position(&mut self) {
        self.pointer_position = None;
    }

    pub fn is_touch_look_active(&self) -> bool {
        self.touch_look_active
    }

    pub fn start_touch(&mut self, id: u64, position: Vec2) {
        self.touch_positions.insert(id, position);
        self.update_touch_gesture();
    }

    pub fn move_touch(&mut self, id: u64, position: Vec2, scale_factor: f32) {
        let Some(previous_position) = self.touch_positions.get_mut(&id) else {
            return;
        };

        let delta = position - *previous_position;
        *previous_position = position;

        match self.touch_positions.len() {
            1 => {
                self.pointer_position = Some(position);
                self.pointer_delta = self.pointer_delta + delta;
            }
            2 => {
                let logical_delta = delta / scale_factor.max(1.0) * -1.0;
                self.scroll_delta = self.scroll_delta + logical_delta * 0.5;
            }
            _ => {}
        }
    }

    pub fn end_touch(&mut self, id: u64) {
        if self.touch_positions.remove(&id).is_some() {
            self.update_touch_gesture();
        }
    }

    fn update_touch_gesture(&mut self) {
        self.touch_look_active = self.touch_positions.len() == 1;
        self.pointer_delta = Vec2::zero();
        self.pointer_position = if self.touch_look_active {
            self.touch_positions.values().next().copied()
        } else {
            None
        };
    }

    pub fn clear(&mut self) {
        self.pressed_keys.clear();
        self.pressed_mouse_buttons.clear();
        self.pointer_position = None;
        self.touch_positions.clear();
        self.touch_look_active = false;
        self.reset_frame_delta();
    }

    pub fn reset_frame_delta(&mut self) {
        self.pointer_delta = Vec2::zero();
        self.scroll_delta = Vec2::zero();
    }
}
