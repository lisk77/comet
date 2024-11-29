use std::cell::RefCell;
use std::rc::Rc;
use crate::math::{
	Vec2,
	Vec3
};
use component_derive::Component;

// ##################################################
// #                    BASIC                       #
// ##################################################

#[derive(Component)]
pub struct Position2D {
	position: Vec2
}

#[derive(Component)]
pub struct Position3D {
	position: Vec3
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
pub struct Rectangle2D{
	position: Position2D,
	size: Vec2
}

#[derive(Component)]
pub struct Render2D {
	is_visible: bool,
	texture: &'static str,
	scale: Vec2
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

// ##################################################
// #                    TRAITS                      #
// ##################################################

pub trait Component: Send + Sync + PartialEq + Default +  'static {
	fn new() -> Self where Self: Sized;

	fn type_id() -> std::any::TypeId {
		std::any::TypeId::of::<Self>()
	}

	fn type_name() -> String {
		std::any::type_name::<Self>().to_string()
	}
}

pub trait Collider {
	fn is_colliding(&self, other: &Self) -> bool;
}

pub trait Render {
	fn is_visible(&self) -> bool;
	fn set_visibility(&mut self, is_visible: bool);
	fn get_texture(&self) -> String;
	fn set_texture(&mut self, texture: &'static str);
}

// ##################################################
// #                    IMPLS                       #
// ##################################################

impl Position2D {
	pub fn from_vec(vec: Vec2) -> Self {
		Self {
			position: vec
		}
	}

	pub fn as_vec(&self) -> Vec2 {
		self.position
	}

	pub fn x(&self) -> f32 {
		self.position.x()
	}

	pub fn y(&self) -> f32 {
		self.position.y()
	}

	pub fn set_x(&mut self, new_x: f32) {
		self.position.set_x(new_x);
	}

	pub fn set_y(&mut self, new_y: f32) {
		self.position.set_y(new_y);
	}
}

impl Position3D {
	pub fn from_vec(vec: Vec3) -> Self {
		Self {
			position: vec
		}
	}

	pub fn as_vec(&self) -> Vec3 {
		self.position
	}

	pub fn x(&self) -> f32 {
		self.position.x()
	}

	pub fn y(&self) -> f32 {
		self.position.y()
	}

	pub fn z(&self) -> f32 {
		self.position.z()
	}

	pub fn set_x(&mut self, new_x: f32) {
		self.position.set_x(new_x);
	}

	pub fn set_y(&mut self, new_y: f32) {
		self.position.set_y(new_y);
	}

	pub fn set_z(&mut self, new_z: f32) {
		self.position.set_z(new_z);
	}
}

impl Rectangle2D {
	pub fn new(position: Position2D, size: Vec2) -> Self {
		Self {
			position,
			size
		}
	}

	pub fn position(&self) -> Position2D {
		self.position
	}

	pub fn set_position(&mut self, position: Position2D) {
		self.position = position;
	}

	pub fn size(&self) -> Vec2 {
		self.size
	}
}

impl Collider for Rectangle2D {
	fn is_colliding(&self, other: &Self) -> bool {
		let x1 = self.position().x();
		let y1 = self.position().y();
		let w1 = self.size().x();
		let h1 = self.size().y();

		let x2 = other.position().x();
		let y2 = other.position().y();
		let w2 = other.size().x();
		let h2 = other.size().y();

		x1 < x2 + w2 &&
		x1 + w1 > x2 &&
		y1 < y2 + h2 &&
		y1 + h1 > y2
	}
}

impl Render for Render2D {
	fn is_visible(&self) -> bool {
		self.is_visible
	}

	fn set_visibility(&mut self, is_visible: bool) {
		self.is_visible = is_visible;
	}

	fn get_texture(&self) -> String {
		self.texture.clone().parse().unwrap()
	}

	/// Use the actual file name of the texture instead of the path
	/// e.g. "comet_icon.png" instead of "resources/textures/comet_icon.png"
	/// The resource manager will already look in the resources/textures folder
	fn set_texture(&mut self, texture: &'static str) {
		self.texture = texture;
	}
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

	pub fn translate(&mut self, displacement: Vec2) {
		let x = self.position().x() + displacement.x();
		let y = self.position().y() + displacement.y();
		self.position_mut().set_x(x);
		self.position_mut().set_y(y);
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