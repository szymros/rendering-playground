use std::collections::HashSet;


pub type Key = winit::keyboard::KeyCode;
pub type Button = winit::event::MouseButton;

pub struct InputState {
    pub held: HashSet<Key>,
    pub pressed: HashSet<Key>,
    pub released: HashSet<Key>,
    pub mouse_delta: (f64, f64),
    pub mouse_buttons: HashSet<Button>,
}

impl InputState {
    pub fn new() -> InputState {
        return InputState {
            held: HashSet::new(),
            pressed: HashSet::new(),
            released: HashSet::new(),
            mouse_delta: (0.0, 0.0),
            mouse_buttons: HashSet::new(),
        };
    }

    pub fn clear(&mut self) {
        self.pressed.clear();
        self.released.clear();
        self.mouse_buttons.clear();
        self.mouse_delta = (0.0, 0.0);
    }

    pub fn key_pressed(&mut self, key: Key) {
        self.held.insert(key.clone());
        self.pressed.insert(key);
    }

    pub fn key_released(&mut self, key: Key) {
        self.held.remove(&key);
        self.released.insert(key);
    }

    pub fn mouse_moved(&mut self, delta: (f64, f64)) {
        self.mouse_delta.0 += delta.0;
        self.mouse_delta.1 += delta.1;
    }

    pub fn mouse_pressed(&mut self, button: Button) {
        self.mouse_buttons.insert(button);
    }

    pub fn mouse_released(&mut self, button: Button) {
        self.mouse_buttons.remove(&button);
    }
}


