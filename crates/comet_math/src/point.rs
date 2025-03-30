use crate::InnerSpace;
use crate::vector::{Vec2, Vec3};

pub trait Point {
	fn lerp(&self, other: &Self, t: f32) -> Self;
	fn to_vec(&self) -> impl InnerSpace;
}

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

impl Point for Point2 {
	fn lerp(&self, other: &Self, t: f32) -> Self {
		let x = self.x + (other.x - self.x) * t;
		let y = self.y + (other.y - self.y) * t;
		Self { x, y }
	}

	fn to_vec(&self) -> Vec2 {
		Vec2::new(self.x, self.y)
	}
}
impl Point for Point3 {
	fn lerp(&self, other: &Self, t: f32) -> Self {
		let x = self.x + (other.x - self.x) * t;
		let y = self.y + (other.y - self.y) * t;
		let z = self.z + (other.z - self.z) * t;
		Self { x, y, z }
	}

	fn to_vec(&self) -> Vec3 {
		Vec3::new(self.x, self.y, self.z)
	}
}

impl Into<Vec2> for Point2 {
	fn into(self) -> Vec2 {
		self.to_vec()
	}
}

impl Into<Vec3> for Point3 {
	fn into(self) -> Vec3 {
		self.to_vec()
	}
}

impl From<Vec2> for Point2 {
	fn from(v: Vec2) -> Self {
		Self::from_vec(v)
	}
}

impl From<Vec3> for Point3 {
	fn from(v: Vec3) -> Self {
		Self::from_vec(v)
	}
}