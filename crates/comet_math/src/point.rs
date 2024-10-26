use crate::vector::{Vec2, Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2 {
	x: f32,
	y: f32
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3 {
	x: f32,
	y: f32,
	z: f32
}

impl Point2 {
	pub fn new(x: f32, y: f32) -> Self {
		Point2 { x, y }
	}

	pub fn from_vec(v: Vec2) -> Self {
		Self { x: v.x(), y: v.y() }
	}

	pub fn to_vec(&self) -> Vec2 {
		Vec2::new(self.x, self.y)
	}

	pub fn x(&self) -> f32 {
		self.x
	}

	pub fn y(&self) -> f32 {
		self.y
	}
}

impl Point3 {
	pub fn new(x: f32, y: f32, z: f32) -> Self {
		Point3 { x, y, z }
	}

	pub fn from_vec(v: Vec3) -> Self {
		Self { x: v.x(), y: v.y(), z: v.z() }
	}

	pub fn to_vec(&self) -> Vec3 {
		Vec3::new(self.x, self.y, self.z)
	}

	pub fn x(&self) -> f32 {
		self.x
	}

	pub fn y(&self) -> f32 {
		self.y
	}

	pub fn z(&self) -> f32 {
		self.z
	}
}
