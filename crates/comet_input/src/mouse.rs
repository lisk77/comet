use winit::event::{
	ElementState,
	WindowEvent,
	MouseButton,
	MouseScrollDelta
};

pub type Button = MouseButton;

pub fn mouse_pressed(event: &WindowEvent, button: Button) -> bool {
	match event {
		WindowEvent::MouseInput {
			button: button_pressed,
			state: ElementState::Pressed,
			..
		} => *button_pressed == button,
		_ => false,
	}
}

pub fn mouse_released(event: &WindowEvent, button: Button) -> bool {
	match event {
		WindowEvent::MouseInput {
			button: button_released,
			state: ElementState::Released,
			..
		} => *button_released == button,
		_ => false,
	}
}

pub fn mouse_wheel_vertical(event: &WindowEvent) -> f32 {
	match event {
		WindowEvent::MouseWheel {
			delta: MouseScrollDelta::LineDelta(_, y),
			..
		} => *y,
		WindowEvent::MouseWheel {
			delta: MouseScrollDelta::PixelDelta(p),
			..
		} => p.y as f32,
		_ => 0.0,
	}
}

pub fn mouse_wheel_horizontal(event: &WindowEvent) -> f32 {
	match event {
		WindowEvent::MouseWheel {
			delta: MouseScrollDelta::LineDelta(x, _),
			..
		} => *x,
		WindowEvent::MouseWheel {
			delta: MouseScrollDelta::PixelDelta(p),
			..
		} => p.x as f32,
		_ => 0.0,
	}
}

pub fn mouse_moved(event: &WindowEvent) -> (f64, f64) {
	match event {
		WindowEvent::CursorMoved {
			position,
			..
		} => (position.x, position.y),
		_ => (0.0, 0.0),
	}
}

pub fn mouse_entered(event: &WindowEvent) -> bool {
	match event {
		WindowEvent::CursorEntered { .. } => true,
		_ => false,
	}
}

pub fn mouse_exited(event: &WindowEvent) -> bool {
	match event {
		WindowEvent::CursorLeft { .. } => true,
		_ => false,
	}
}

pub fn mouse_dragged(event: &WindowEvent) -> bool {
	match event {
		WindowEvent::CursorMoved { .. } => true,
		_ => false,
	}
}