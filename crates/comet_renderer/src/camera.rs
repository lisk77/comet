use comet_math::{Point3, Vec2, Vec3};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - 0.0001;

pub struct Camera {
	zoom: f32,
	dimension: Vec2,
	position: Vec3
}

impl Camera {
	pub fn new(
		zoom: f32,
		dimension: Vec2,
		position: Vec3
	) -> Self {
		Self {
			zoom,
			dimension,
			position
		}
	}

	pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
		// 1.

		let proj = cgmath::ortho(self.position.x() - self.dimension.x() / 2.0, self.position.x() + self.dimension.x() / 2.0, self.position.y() - self.dimension.y() / 2.0, self.position.y() + self.dimension.y() / 2.0, 1.0, 0.0);
		// 3.
		return OPENGL_TO_WGPU_MATRIX * proj;
	}
}


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
	view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
	pub fn new() -> Self {
		use cgmath::SquareMatrix;
		Self {
			view_proj: cgmath::Matrix4::identity().into(),
		}
	}

	pub fn update_view_proj(&mut self, camera: &Camera) {
		self.view_proj = camera.build_view_projection_matrix().into();
	}
}
/*use comet_math::{Mat4, Point3, Vec3};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct Camera {
	eye: Point3,
	target: Point3,
	up: Vec3,
	aspect: f32,
	fovy: f32,
	znear: f32,
	zfar: f32,
}

impl Camera {
	pub fn new(eye: Point3, target: Point3, up: Vec3, aspect: f32, fovy: f32, znear: f32, zfar: f32) -> Self {
		Self {
			eye,
			target,
			up,
			aspect,
			fovy,
			znear,
			zfar,
		}
	}

	pub fn build_view_projection_matrix(&self) -> Mat4 {
		let view = Mat4::look_at_rh(self.eye, self.target, self.up);
		let proj = Mat4::perspective_matrix(self.fovy, self.aspect, self.znear, self.zfar);

		(OPENGL_TO_WGPU_MATRIX * proj * view).transpose()
	}
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
	view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
	pub fn new() -> Self {
		Self {
			view_proj: Mat4::IDENTITY.into(),
		}
	}

	pub fn update_view_proj(&mut self, camera: &Camera) {
		self.view_proj = camera.build_view_projection_matrix().into();
	}
}*/

/*use std::f32::consts::FRAC_PI_2;
use std::time::Duration;
use winit::dpi::PhysicalPosition;
use winit::event::*;
use winit::keyboard::KeyCode;

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct Camera3D {
	pub position: Point3,
	yaw: f32,
	pitch: f32,
}

impl Camera3D {
	pub fn new(
		position: Point3,
		yaw: f32,
		pitch: f32,
	) -> Self {
		Self {
			position: position.into(),
			yaw: yaw.into(),
			pitch: pitch.into(),
		}
	}

	pub fn calc_matrix(&self) -> Mat4 {
		let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
		let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

		Mat4::look_to_rh(
			self.position,
			Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
			Vec3::unit_y(),
		)
	}
}

pub struct Projection {
	aspect: f32,
	fovy: Rad<f32>,
	znear: f32,
	zfar: f32,
}

impl Projection {
	pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
		Self {
			aspect: width as f32 / height as f32,
			fovy: fovy.into(),
			znear,
			zfar,
		}
	}

	pub fn resize(&mut self, width: u32, height: u32) {
		self.aspect = width as f32 / height as f32;
	}

	pub fn calc_matrix(&self) -> Matrix4<f32> {
		// UDPATE
		perspective(self.fovy, self.aspect, self.znear, self.zfar)
	}
}

#[derive(Debug)]
pub struct CameraController {
	amount_left: f32,
	amount_right: f32,
	amount_forward: f32,
	amount_backward: f32,
	amount_up: f32,
	amount_down: f32,
	rotate_horizontal: f32,
	rotate_vertical: f32,
	scroll: f32,
	speed: f32,
	sensitivity: f32,
}

impl CameraController {
	pub fn new(speed: f32, sensitivity: f32) -> Self {
		Self {
			amount_left: 0.0,
			amount_right: 0.0,
			amount_forward: 0.0,
			amount_backward: 0.0,
			amount_up: 0.0,
			amount_down: 0.0,
			rotate_horizontal: 0.0,
			rotate_vertical: 0.0,
			scroll: 0.0,
			speed,
			sensitivity,
		}
	}

	pub fn process_keyboard(&mut self, key: KeyCode, state: ElementState) -> bool {
		let amount = if state == ElementState::Pressed {
			1.0
		} else {
			0.0
		};
		match key {
			KeyCode::KeyW | KeyCode::ArrowUp => {
				self.amount_forward = amount;
				true
			}
			KeyCode::KeyS | KeyCode::ArrowDown => {
				self.amount_backward = amount;
				true
			}
			KeyCode::KeyA | KeyCode::ArrowLeft => {
				self.amount_left = amount;
				true
			}
			KeyCode::KeyD | KeyCode::ArrowRight => {
				self.amount_right = amount;
				true
			}
			KeyCode::Space => {
				self.amount_up = amount;
				true
			}
			KeyCode::ShiftLeft => {
				self.amount_down = amount;
				true
			}
			_ => false,
		}
	}

	pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
		self.rotate_horizontal = mouse_dx as f32;
		self.rotate_vertical = mouse_dy as f32;
	}

	pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
		self.scroll = match delta {
			// I'm assuming a line is about 100 pixels
			MouseScrollDelta::LineDelta(_, scroll) => -scroll * 0.5,
			MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => -*scroll as f32,
		};
	}

	pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
		let dt = dt.as_secs_f32();

		// Move forward/backward and left/right
		let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
		let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
		let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
		camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
		camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

		// Move in/out (aka. "zoom")
		// Note: this isn't an actual zoom. The camera's position
		// changes when zooming. I've added this to make it easier
		// to get closer to an object you want to focus on.
		let (pitch_sin, pitch_cos) = camera.pitch.0.sin_cos();
		let scrollward =
			Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
		camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
		self.scroll = 0.0;

		// Move up/down. Since we don't use roll, we can just
		// modify the y coordinate directly.
		camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

		// Rotate
		camera.yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
		camera.pitch += Rad(-self.rotate_vertical) * self.sensitivity * dt;

		// If process_mouse isn't called every frame, these values
		// will not get set to zero, and the camera will rotate
		// when moving in a non cardinal direction.
		self.rotate_horizontal = 0.0;
		self.rotate_vertical = 0.0;

		// Keep the camera's angle from going too high/low.
		if camera.pitch < -Rad(SAFE_FRAC_PI_2) {
			camera.pitch = -Rad(SAFE_FRAC_PI_2);
		} else if camera.pitch > Rad(SAFE_FRAC_PI_2) {
			camera.pitch = Rad(SAFE_FRAC_PI_2);
		}
	}
}*/