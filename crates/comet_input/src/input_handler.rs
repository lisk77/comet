use crate::keyboard::Key;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::keyboard::PhysicalKey;

#[derive(Debug)]
pub struct InputHandler {
    keys_pressed: Vec<PhysicalKey>,
    keys_held: Vec<PhysicalKey>,
    keys_released: Vec<PhysicalKey>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            keys_pressed: Vec::new(),
            keys_held: Vec::new(),
            keys_released: Vec::new(),
        }
    }

    pub fn update<T>(&mut self, event: &Event<T>) {
        match event {
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state,
                                physical_key: PhysicalKey::Code(keycode),
                                ..
                            },
                        ..
                    },
                ..
            } => match state {
                ElementState::Pressed => {
                    if self
                        .keys_pressed
                        .contains(&PhysicalKey::Code(keycode.clone()))
                    {
                        self.keys_held.push(PhysicalKey::Code(keycode.clone()));
                    } else {
                        self.keys_pressed.push(PhysicalKey::Code(keycode.clone()));
                    }
                    self.keys_pressed.push(PhysicalKey::Code(keycode.clone()));
                }
                ElementState::Released => {
                    self.keys_released = vec![];
                    if let Some(index) = self
                        .keys_pressed
                        .iter()
                        .position(|&x| x == PhysicalKey::Code(keycode.clone()))
                    {
                        self.keys_pressed.remove(index);
                    }
                    if let Some(index) = self
                        .keys_held
                        .iter()
                        .position(|&x| x == PhysicalKey::Code(keycode.clone()))
                    {
                        self.keys_held.remove(index);
                    }
                    self.keys_released.push(PhysicalKey::Code(keycode.clone()));
                }
            },
            _ => {}
        }
    }

    pub fn key_pressed(&self, key: Key) -> bool {
        self.keys_pressed.contains(&PhysicalKey::Code(key))
    }

    pub fn key_held(&self, key: Key) -> bool {
        self.keys_held.contains(&PhysicalKey::Code(key))
    }

    pub fn key_released(&self, key: Key) -> bool {
        self.keys_released.contains(&PhysicalKey::Code(key))
    }
}
