use crate::InnerSpace;
use crate::vector::{v2, v3};

pub trait Point {
	fn lerp(&self, other: &Self, t: f32) -> Self;
	fn to_vec(&self) -> impl InnerSpace;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct p2 {
	x: f32,
	y: f32
}

impl p2 {
	pub fn new(x: f32, y: f32) -> Self {
		p2 { x, y }
	}

	pub fn from_vec(v: v2) -> Self {
		Self { x: v.x(), y: v.y() }
	}

	pub fn x(&self) -> f32 {
		self.x
	}

	pub fn y(&self) -> f32 {
		self.y
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct p3 {
	x: f32,
	y: f32,
	z: f32
}

impl p3 {
	pub fn new(x: f32, y: f32, z: f32) -> Self {
		p3 { x, y, z }
	}

	pub fn from_vec(v: v3) -> Self {
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

impl Point for p2 {
	fn lerp(&self, other: &Self, t: f32) -> Self {
		let x = self.x + (other.x - self.x) * t;
		let y = self.y + (other.y - self.y) * t;
		Self { x, y }
	}

	fn to_vec(&self) -> v2 {
		v2::new(self.x, self.y)
	}
}
impl Point for p3 {
	fn lerp(&self, other: &Self, t: f32) -> Self {
		let x = self.x + (other.x - self.x) * t;
		let y = self.y + (other.y - self.y) * t;
		let z = self.z + (other.z - self.z) * t;
		Self { x, y, z }
	}

	fn to_vec(&self) -> v3 {
		v3::new(self.x, self.y, self.z)
	}
}

impl Into<v2> for p2 {
	fn into(self) -> v2 {
		self.to_vec()
	}
}

impl Into<v3> for p3 {
	fn into(self) -> v3 {
		self.to_vec()
	}
}

impl From<v2> for p2 {
	fn from(v: v2) -> Self {
		Self::from_vec(v)
	}
}

impl From<v3> for p3 {
	fn from(v: v3) -> Self {
		Self::from_vec(v)
	}
}