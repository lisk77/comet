use std::path::Path;
use crate::math::{
	Vec2,
	Vec3
};
use component_derive::Component;

pub trait Component: Send + Sync + PartialEq + Default +  'static {
	fn new() -> Self where Self: Sized;

	fn type_id() -> std::any::TypeId {
		std::any::TypeId::of::<Self>()
	}

	fn type_name() -> String {
		std::any::type_name::<Self>().to_string()
	}
}

// ##################################################
// #                    BASIC                       #
// ##################################################

#[derive(Component)]
pub struct Position2D {
	x: f32,
	y: f32
}

#[derive(Component)]
pub struct Position3D {
	x: f32,
	y: f32,
	z: f32
}

#[derive(Component)]
pub struct Rotation2D {
	theta: f32
}

#[derive(Component)]
pub struct Rotation3D {
	theta_x: f32,
	theta_y: f32,
	theta_z: f32
}

#[derive(Component)]
pub struct Renderer2D {
	is_visible: bool,
	texture: &'static str,
	scale: f32
}

impl Position2D {
	pub fn from_vec(vec: Vec2) -> Self {
		Self {
			x: vec.x(),
			y: vec.y()
		}
	}

	pub fn as_vec(&self) -> Vec2 {
		Vec2::new(
			self.x,
			self.y
		)
	}

	pub fn x(&self) -> &f32 {
		&self.x
	}

	pub fn y(&self) -> &f32 {
		&self.y
	}

	pub fn set_x(&mut self, new_x: f32) {
		self.x = new_x;
	}

	pub fn set_y(&mut self, new_y: f32) {
		self.y = new_y;
	}
}

impl Position3D {
	pub fn from_vec(vec: Vec3) -> Self {
		Self {
			x: vec.x(),
			y: vec.y(),
			z: vec.z()
		}
	}

	pub fn as_vec(&self) -> Vec3 {
		Vec3::new(
			self.x,
			self.y,
			self.z
		)
	}

	pub fn x(&self) -> &f32 {
		&self.x
	}

	pub fn y(&self) -> &f32 {
		&self.y
	}

	pub fn z(&self) -> &f32 {
		&self.z
	}

	pub fn set_x(&mut self, new_x: f32) {
		self.x = new_x;
	}

	pub fn set_y(&mut self, new_y: f32) {
		self.y = new_y;
	}

	pub fn set_z(&mut self, new_z: f32) {
		self.z = new_z
	}
}

// ##################################################
// #                   BUNDLES                      #
// ##################################################

#[derive(Component)]
pub struct Transform2D {
	position: Position2D,
	rotation: Rotation2D
}

#[derive(Component)]
pub struct Transform3D {
	position: Position3D,
	rotation: Rotation3D
}

impl Transform2D {
	pub fn position(&self) -> &Position2D {
		&self.position
	}

	pub fn position_mut(&mut self) -> &mut Position2D {
		&mut self.position
	}

	pub fn rotation(&self) -> &Rotation2D {
		&self.rotation
	}

	pub fn rotation_mut(&mut self) -> &mut Rotation2D {
		&mut self.rotation
	}
}

impl Transform3D {
	pub fn position(&self) -> &Position3D {
		&self.position
	}

	pub fn position_mut(&mut self) -> &mut Position3D {
		&mut self.position
	}

	pub fn rotation(&self) -> &Rotation3D {
		&self.rotation
	}

	pub fn rotation_mut(&mut self) -> &mut Rotation3D {
		&mut self.rotation
	}
}


