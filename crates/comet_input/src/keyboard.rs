use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{ KeyCode, PhysicalKey};

pub type Key = KeyCode;

pub fn key_pressed(event: &WindowEvent, key_code: Key) -> bool {
	match event {
		WindowEvent::KeyboardInput {
			event: KeyEvent {
				state: ElementState::Pressed,
				physical_key: PhysicalKey::Code(code),
				..
			},
			..
		} => *code == key_code,
		_ => false,
	}
}

pub fn key_released(event: &WindowEvent, key_code: Key) -> bool {
	match event {
		WindowEvent::KeyboardInput {
			event: KeyEvent {
				state: ElementState::Released,
				physical_key: PhysicalKey::Code(code),
				..
			},
			..
		} => *code == key_code,
		_ => false,
	}
}