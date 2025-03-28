use crate::point::{Point2, Point3};
use crate::quaternion::Quat;
use crate::utilities::acos;
use std::ops::*;

pub trait InnerSpace {
	fn dot(&self, other: &Self) -> f32;
	fn dist(&self, other: &Self) -> f32;
	fn v_angle(&self, other: &Self) -> f32;
}

// ##################################################
// #                   VECTOR 2D                    #
// ##################################################

/// Representation of a 2D Vector
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Vec2 {
	x: f32,
	y: f32,
}

impl Vec2 {
	pub const X: Vec2 = Vec2 { x: 1.0, y: 0.0 };
	pub const Y: Vec2 = Vec2 { x: 0.0, y: 1.0 };
	pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

	pub const fn new(x: f32, y: f32) -> Self {
		Vec2 { x, y }
	}

	pub fn from_point(p: Point2) -> Self {
		Self { x: p.x(), y: p.y() }
	}

	pub fn x(&self) -> f32 {
		self.x
	}

	pub fn y(&self) -> f32 {
		self.y
	}

	pub fn set_x(&mut self, new_x: f32) {
		self.x = new_x;
	}

	pub fn set_y(&mut self, new_y: f32) {
		self.y = new_y;
	}

	pub fn length(&self) -> f32 {
		(self.x * self.x + self.y * self.y).sqrt()
	}

	pub fn normalize(&self) -> Self {
		let factor = 1.0 / self.length();
		Vec2 {
			x: factor * self.x,
			y: factor * self.y,
		}
	}

	pub fn xx(&self) -> Vec2 {
		Vec2 {
			x: self.x,
			y: self.x,
		}
	}

	pub fn xy(&self) -> Vec2 {
		Vec2 {
			x: self.x,
			y: self.y,
		}
	}

	pub fn yx(&self) -> Vec2 {
		Vec2 {
			x: self.y,
			y: self.x,
		}
	}

	pub fn yy(&self) -> Vec2 {
		Vec2 {
			x: self.y,
			y: self.y,
		}
	}
}

impl Add<Vec2> for Vec2 {
	type Output = Vec2;

	fn add(self, other: Vec2) -> Vec2 {
		Vec2 {
			x: self.x + other.x,
			y: self.y + other.y,
		}
	}
}

impl AddAssign for Vec2 {
	fn add_assign(&mut self, other: Vec2) {
		self.x += other.x;
		self.y += other.y;
	}
}

impl Sub<Vec2> for Vec2 {
	type Output = Vec2;

	fn sub(self, other: Vec2) -> Vec2 {
		Vec2 {
			x: self.x - other.x,
			y: self.y - other.y,
		}
	}
}

impl SubAssign for Vec2 {
	fn sub_assign(&mut self, other: Vec2) {
		self.x -= other.x;
		self.y -= other.y;
	}
}

impl Mul<f32> for Vec2 {
	type Output = Vec2;

	fn mul(self, other: f32) -> Vec2 {
		Vec2 {
			x: self.x * other,
			y: self.y * other,
		}
	}
}

impl Mul<Vec2> for f32 {
	type Output = Vec2;

	fn mul(self, other: Vec2) -> Vec2 {
		Vec2 {
			x: self * other.x,
			y: self * other.y,
		}
	}
}

impl Div<f32> for Vec2 {
	type Output = Vec2;

	fn div(self, other: f32) -> Vec2 {
		Vec2 {
			x: self.x / other,
			y: self.y / other,
		}
	}
}

impl Into<[f32;2]> for Vec2 {
	fn into(self) -> [f32;2] {
		[self.x, self.y]
	}
}

impl Into<Vec2> for [f32;2] {
	fn into(self) -> Vec2 {
		Vec2 {
			x: self[0],
			y: self[1],
		}
	}
}

/// Representation of a 2D integer Vector
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IVec2 {
	x: i64,
	y: i64,
}

impl IVec2 {
	pub const X: IVec2 = IVec2 { x: 1, y: 0 };
	pub const Y: IVec2 = IVec2 { x: 0, y: 1 };
	pub const ZERO: IVec2 = IVec2 { x: 0, y: 0 };

	pub const fn new(x: i64, y: i64) -> Self {
		IVec2 { x, y }
	}

	pub fn from_point(p: Point2) -> Self {
		Self { x: p.x() as i64, y: p.y() as i64 }
	}

	pub fn as_vec2(&self) -> Vec2 {
		Vec2 {
			x: self.x as f32,
			y: self.y as f32,
		}
	}

	pub fn x(&self) -> i64 {
		self.x
	}

	pub fn y(&self) -> i64 {
		self.y
	}

	pub fn set_x(&mut self, new_x: i64) {
		self.x = new_x;
	}

	pub fn set_y(&mut self, new_y: i64) {
		self.y = new_y;
	}

	pub fn length(&self) -> i64 {
		((self.x * self.x + self.y * self.y) as f32).sqrt() as i64
	}

	pub fn normalize(&self) -> Self {
		let factor = 1.0 / self.length() as f32;
		IVec2 {
			x: (factor * self.x as f32) as i64,
			y: (factor * self.y as f32) as i64,
		}
	}

	pub fn xx(&self) -> Self {
		Self {
			x: self.x,
			y: self.x,
		}
	}

	pub fn xy(&self) -> Self {
		Self {
			x: self.x,
			y: self.y,
		}
	}

	pub fn yx(&self) -> Self {
		Self {
			x: self.y,
			y: self.x,
		}
	}

	pub fn yy(&self) -> Self {
		Self {
			x: self.y,
			y: self.y,
		}
	}
}

impl Add<IVec2> for IVec2 {
	type Output = IVec2;

	fn add(self, other: IVec2) -> IVec2 {
		IVec2 {
			x: self.x + other.x,
			y: self.y + other.y,
		}
	}
}

impl Add<IVec2> for Vec2 {
	type Output = Vec2;

	fn add(self, other: IVec2) -> Vec2 {
		Vec2 {
			x: self.x + other.x as f32,
			y: self.y + other.y as f32,
		}
	}
}

impl Add<Vec2> for IVec2 {
	type Output = Vec2;

	fn add(self, other: Vec2) -> Vec2 {
		Vec2 {
			x: self.x as f32 + other.x,
			y: self.y as f32 + other.y,
		}
	}
}

impl AddAssign for IVec2 {
	fn add_assign(&mut self, other: IVec2) {
		self.x += other.x;
		self.y += other.y;
	}
}

impl Sub<IVec2> for IVec2 {
	type Output = IVec2;

	fn sub(self, other: IVec2) -> IVec2 {
		IVec2 {
			x: self.x - other.x,
			y: self.y - other.y,
		}
	}
}

impl Sub<IVec2> for Vec2 {
	type Output = Vec2;

	fn sub(self, other: IVec2) -> Vec2 {
		Vec2 {
			x: self.x - other.x as f32,
			y: self.y - other.y as f32,
		}
	}
}

impl Sub<Vec2> for IVec2 {
	type Output = Vec2;

	fn sub(self, other: Vec2) -> Vec2 {
		Vec2 {
			x: self.x as f32 - other.x,
			y: self.y as f32 - other.y,
		}
	}
}

impl SubAssign for IVec2 {
	fn sub_assign(&mut self, other: IVec2) {
		self.x -= other.x;
		self.y -= other.y;
	}
}

impl Mul<f32> for IVec2 {
	type Output = IVec2;

	fn mul(self, other: f32) -> IVec2 {
		IVec2 {
			x: self.x * other as i64,
			y: self.y * other as i64,
		}
	}
}

impl From<IVec2> for Vec2 {
	fn from(v: IVec2) -> Vec2 {
		Vec2 {
			x: v.x as f32,
			y: v.y as f32,
		}
	}
}

impl From<Vec2> for IVec2 {
	fn from(v: Vec2) -> IVec2 {
		IVec2 {
			x: v.x as i64,
			y: v.y as i64,
		}
	}
}

impl Into<[i64;2]> for IVec2 {
	fn into(self) -> [i64;2] {
		[self.x, self.y]
	}
}

impl Into<[f32;2]> for IVec2 {
	fn into(self) -> [f32;2] {
		[self.x as f32, self.y as f32]
	}
}

// ##################################################
// #                   VECTOR 3D                    #
// ##################################################

/// Representation of a 3D Vector
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Vec3 {
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

impl Vec3 {
	pub const X: Vec3 = Vec3 { x: 1.0, y: 0.0, z: 0.0 };
	pub const Y: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
	pub const Z: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 1.0 };
	pub const ZERO: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

	pub const fn new(x: f32, y: f32, z: f32) -> Self {
		Vec3 { x, y, z }
	}

	pub fn from_point(p: Point3) -> Self {
		Self {
			x: p.x(),
			y: p.y(),
			z: p.z(),
		}
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

	pub fn set_x(&mut self, new_x: f32) {
		self.x = new_x;
	}

	pub fn set_y(&mut self, new_y: f32) {
		self.y = new_y;
	}

	pub fn set_z(&mut self, new_z: f32) {
		self.z = new_z;
	}

	pub fn length(&self) -> f32 {
		(self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
	}

	pub fn normalize(&self) -> Self {
		let factor = 1.0 / self.length();
		Vec3 {
			x: factor * self.x,
			y: factor * self.y,
			z: factor * self.z,
		}
	}

	pub fn xxx(&self) -> Vec3 {
		Vec3 {
			x: self.x,
			y: self.x,
			z: self.x,
		}
	}
	pub fn xxy(&self) -> Vec3 {
		Vec3 {
			x: self.x,
			y: self.x,
			z: self.y,
		}
	}
	pub fn xxz(&self) -> Vec3 {
		Vec3 {
			x: self.x,
			y: self.x,
			z: self.z,
		}
	}
	pub fn xyx(&self) -> Vec3 {
		Vec3 {
			x: self.x,
			y: self.y,
			z: self.x,
		}
	}
	pub fn xyy(&self) -> Vec3 {
		Vec3 {
			x: self.x,
			y: self.y,
			z: self.y,
		}
	}
	pub fn xyz(&self) -> Vec3 {
		Vec3 {
			x: self.x,
			y: self.y,
			z: self.z,
		}
	}
	pub fn xzx(&self) -> Vec3 {
		Vec3 {
			x: self.x,
			y: self.z,
			z: self.x,
		}
	}
	pub fn xzy(&self) -> Vec3 {
		Vec3 {
			x: self.x,
			y: self.z,
			z: self.y,
		}
	}
	pub fn xzz(&self) -> Vec3 {
		Vec3 {
			x: self.x,
			y: self.z,
			z: self.z,
		}
	}
	pub fn yxx(&self) -> Vec3 {
		Vec3 {
			x: self.y,
			y: self.x,
			z: self.x,
		}
	}
	pub fn yxy(&self) -> Vec3 {
		Vec3 {
			x: self.y,
			y: self.x,
			z: self.y,
		}
	}
	pub fn yxz(&self) -> Vec3 {
		Vec3 {
			x: self.y,
			y: self.x,
			z: self.z,
		}
	}
	pub fn yyx(&self) -> Vec3 {
		Vec3 {
			x: self.y,
			y: self.y,
			z: self.x,
		}
	}
	pub fn yyy(&self) -> Vec3 {
		Vec3 {
			x: self.y,
			y: self.y,
			z: self.y,
		}
	}
	pub fn yyz(&self) -> Vec3 {
		Vec3 {
			x: self.y,
			y: self.y,
			z: self.z,
		}
	}
	pub fn yzx(&self) -> Vec3 {
		Vec3 {
			x: self.y,
			y: self.z,
			z: self.x,
		}
	}
	pub fn yzy(&self) -> Vec3 {
		Vec3 {
			x: self.y,
			y: self.z,
			z: self.y,
		}
	}
	pub fn yzz(&self) -> Vec3 {
		Vec3 {
			x: self.y,
			y: self.z,
			z: self.z,
		}
	}
	pub fn zxx(&self) -> Vec3 {
		Vec3 {
			x: self.z,
			y: self.x,
			z: self.x,
		}
	}
	pub fn zxy(&self) -> Vec3 {
		Vec3 {
			x: self.z,
			y: self.x,
			z: self.y,
		}
	}
	pub fn zxz(&self) -> Vec3 {
		Vec3 {
			x: self.z,
			y: self.x,
			z: self.z,
		}
	}
	pub fn zyx(&self) -> Vec3 {
		Vec3 {
			x: self.z,
			y: self.y,
			z: self.x,
		}
	}
	pub fn zyy(&self) -> Vec3 {
		Vec3 {
			x: self.z,
			y: self.y,
			z: self.y,
		}
	}
	pub fn zyz(&self) -> Vec3 {
		Vec3 {
			x: self.z,
			y: self.y,
			z: self.z,
		}
	}
	pub fn zzx(&self) -> Vec3 {
		Vec3 {
			x: self.z,
			y: self.z,
			z: self.x,
		}
	}
	pub fn zzy(&self) -> Vec3 {
		Vec3 {
			x: self.z,
			y: self.z,
			z: self.y,
		}
	}
	pub fn zzz(&self) -> Vec3 {
		Vec3 {
			x: self.z,
			y: self.z,
			z: self.z,
		}
	}
}

impl Add<Vec3> for Vec3 {
	type Output = Vec3;

	fn add(self, other: Vec3) -> Vec3 {
		Vec3 {
			x: self.x + other.x,
			y: self.y + other.y,
			z: self.z + other.z,
		}
	}
}

impl AddAssign for Vec3 {
	fn add_assign(&mut self, other: Vec3) {
		self.x += other.x;
		self.y += other.y;
		self.z += other.z;
	}
}

impl Sub<Vec3> for Vec3 {
	type Output = Vec3;

	fn sub(self, other: Vec3) -> Vec3 {
		Vec3 {
			x: self.x - other.x,
			y: self.y - other.y,
			z: self.z - other.z,
		}
	}
}

impl SubAssign for Vec3 {
	fn sub_assign(&mut self, other: Vec3) {
		self.x -= other.x;
		self.y -= other.y;
		self.z -= other.z;
	}
}

impl Mul<f32> for Vec3 {
	type Output = Vec3;

	fn mul(self, other: f32) -> Vec3 {
		Vec3 {
			x: self.x * other,
			y: self.y * other,
			z: self.z * other,
		}
	}
}

impl Mul<Vec3> for f32 {
	type Output = Vec3;

	fn mul(self, other: Vec3) -> Vec3 {
		Vec3 {
			x: self * other.x,
			y: self * other.y,
			z: self * other.z,
		}
	}
}

impl Div<f32> for Vec3 {
	type Output = Vec3;

	fn div(self, other: f32) -> Vec3 {
		Vec3 {
			x: self.x / other,
			y: self.y / other,
			z: self.z / other,
		}
	}
}

impl Into<Quat> for Vec3 {
	fn into(self) -> Quat {
		Quat::new(0.0, self)
	}
}

impl Into<[f32;3]> for Vec3 {
	fn into(self) -> [f32;3] {
		[self.x, self.y, self.z]
	}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IVec3 {
	pub x: i64,
	pub y: i64,
	pub z: i64,
}

impl IVec3 {
	pub const X: IVec3 = IVec3 { x: 1, y: 0, z: 0 };
	pub const Y: IVec3 = IVec3 { x: 0, y: 1, z: 0 };
	pub const Z: IVec3 = IVec3 { x: 0, y: 0, z: 1 };
	pub const ZERO: IVec3 = IVec3 { x: 0, y: 0, z: 0 };

	pub const fn new(x: i64, y: i64, z: i64) -> Self {
		IVec3 { x, y, z }
	}

	pub fn from_point(p: Point3) -> Self {
		Self {
			x: p.x() as i64,
			y: p.y() as i64,
			z: p.z() as i64,
		}
	}

	pub fn x(&self) -> i64 {
		self.x
	}

	pub fn y(&self) -> i64 {
		self.y
	}

	pub fn z(&self) -> i64 {
		self.z
	}

	pub fn set_x(&mut self, new_x: i64) {
		self.x = new_x;
	}

	pub fn set_y(&mut self, new_y: i64) {
		self.y = new_y;
	}

	pub fn set_z(&mut self, new_z: i64) {
		self.z = new_z;
	}

	pub fn length(&self) -> i64 {
		((self.x * self.x + self.y * self.y + self.z * self.z) as f32).sqrt() as i64
	}

	pub fn normalize(&self) -> Self {
		let factor = 1 / self.length();
		IVec3 {
			x: factor * self.x,
			y: factor * self.y,
			z: factor * self.z,
		}
	}
}

impl Add<IVec3> for IVec3 {
	type Output = IVec3;

	fn add(self, other: IVec3) -> IVec3 {
		IVec3 {
			x: self.x + other.x,
			y: self.y + other.y,
			z: self.z + other.z,
		}
	}
}

impl Add<IVec3> for Vec3 {
	type Output = Vec3;

	fn add(self, other: IVec3) -> Vec3 {
		Vec3 {
			x: self.x + other.x as f32,
			y: self.y + other.y as f32,
			z: self.z + other.z as f32,
		}
	}
}

impl Add<Vec3> for IVec3 {
	type Output = Vec3;

	fn add(self, other: Vec3) -> Vec3 {
		Vec3 {
			x: self.x as f32 + other.x,
			y: self.y as f32 + other.y,
			z: self.z as f32 + other.z,
		}
	}
}

impl AddAssign for IVec3 {
	fn add_assign(&mut self, other: IVec3) {
		self.x += other.x;
		self.y += other.y;
		self.z += other.z;
	}
}

impl Sub<IVec3> for IVec3 {
	type Output = IVec3;

	fn sub(self, other: IVec3) -> IVec3 {
		IVec3 {
			x: self.x - other.x,
			y: self.y - other.y,
			z: self.z - other.z,
		}
	}
}

impl Sub<IVec3> for Vec3 {
	type Output = Vec3;

	fn sub(self, other: IVec3) -> Vec3 {
		Vec3 {
			x: self.x - other.x as f32,
			y: self.y - other.y as f32,
			z: self.z - other.z as f32,
		}
	}
}

impl Sub<Vec3> for IVec3 {
	type Output = Vec3;

	fn sub(self, other: Vec3) -> Vec3 {
		Vec3 {
			x: self.x as f32 - other.x,
			y: self.y as f32 - other.y,
			z: self.z as f32 - other.z,
		}
	}
}

impl SubAssign for IVec3 {
	fn sub_assign(&mut self, other: IVec3) {
		self.x -= other.x;
		self.y -= other.y;
	}
}

impl Mul<f32> for IVec3 {
	type Output = IVec3;

	fn mul(self, other: f32) -> IVec3 {
		IVec3 {
			x: self.x * other as i64,
			y: self.y * other as i64,
			z: self.z * other as i64,
		}
	}
}

impl From<IVec3> for Vec3 {
	fn from(v: IVec3) -> Vec3 {
		Vec3 {
			x: v.x as f32,
			y: v.y as f32,
			z: v.z as f32,
		}
	}
}

impl From<Vec3> for IVec3 {
	fn from(v: Vec3) -> IVec3 {
		IVec3 {
			x: v.x as i64,
			y: v.y as i64,
			z: v.z as i64,
		}
	}
}

// ##################################################
// #                   VECTOR 4D                    #
// ##################################################

/// Representation of a 4D Vector
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Vec4 {
	x: f32,
	y: f32,
	z: f32,
	w: f32,
}

impl Vec4 {
	pub const X: Vec4 = Vec4 { x: 1.0, y: 0.0, z: 0.0, w: 0.0 };
	pub const Y: Vec4 = Vec4 { x: 0.0, y: 1.0, z: 0.0, w: 0.0 };
	pub const Z: Vec4 = Vec4 { x: 0.0, y: 0.0, z: 1.0, w: 0.0 };
	pub const W: Vec4 = Vec4 { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

	pub const ZERO: Vec4 = Vec4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };

	pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
		Vec4 { x, y, z, w }
	}

	pub fn length(&self) -> f32 {
		(self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
	}

	pub fn normalize(&self) -> Self {
		let factor = 1.0 / self.length();
		Vec4 {
			x: factor * self.x,
			y: factor * self.y,
			z: factor * self.z,
			w: factor * self.w,
		}
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

	pub fn w(&self) -> f32 {
		self.w
	}

	pub fn set_x(&mut self, new_x: f32) {
		self.x = new_x;
	}

	pub fn set_y(&mut self, new_y: f32) {
		self.y = new_y;
	}

	pub fn set_z(&mut self, new_z: f32) {
		self.z = new_z;
	}

	pub fn set_w(&mut self, new_w: f32) {
		self.w = new_w;
	}

	pub fn xxxx(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.x,
			z: self.x,
			w: self.x,
		}
	}
	pub fn xxxy(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.x,
			z: self.x,
			w: self.y,
		}
	}
	pub fn xxxz(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.x,
			z: self.x,
			w: self.z,
		}
	}
	pub fn xxxw(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.x,
			z: self.x,
			w: self.w,
		}
	}
	pub fn xxyx(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.x,
			z: self.y,
			w: self.x,
		}
	}
	pub fn xxyy(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.x,
			z: self.y,
			w: self.y,
		}
	}
	pub fn xxyz(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.x,
			z: self.y,
			w: self.z,
		}
	}
	pub fn xxyw(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.x,
			z: self.y,
			w: self.w,
		}
	}
	pub fn xxzx(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.x,
			z: self.z,
			w: self.x,
		}
	}
	pub fn xxzy(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.x,
			z: self.z,
			w: self.y,
		}
	}
	pub fn xxzz(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.x,
			z: self.z,
			w: self.z,
		}
	}
	pub fn xxzw(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.x,
			z: self.z,
			w: self.w,
		}
	}
	pub fn xxwx(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.x,
			z: self.w,
			w: self.x,
		}
	}
	pub fn xxwy(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.x,
			z: self.w,
			w: self.y,
		}
	}
	pub fn xxwz(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.x,
			z: self.w,
			w: self.z,
		}
	}
	pub fn xxww(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.x,
			z: self.w,
			w: self.w,
		}
	}
	pub fn xyxx(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.y,
			z: self.x,
			w: self.x,
		}
	}
	pub fn xyxy(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.y,
			z: self.x,
			w: self.y,
		}
	}
	pub fn xyxz(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.y,
			z: self.x,
			w: self.z,
		}
	}
	pub fn xyxw(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.y,
			z: self.x,
			w: self.w,
		}
	}
	pub fn xyyx(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.y,
			z: self.y,
			w: self.x,
		}
	}
	pub fn xyyy(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.y,
			z: self.y,
			w: self.y,
		}
	}
	pub fn xyyz(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.y,
			z: self.y,
			w: self.z,
		}
	}
	pub fn xyyw(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.y,
			z: self.y,
			w: self.w,
		}
	}
	pub fn xyzx(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.y,
			z: self.z,
			w: self.x,
		}
	}
	pub fn xyzy(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.y,
			z: self.z,
			w: self.y,
		}
	}
	pub fn xyzz(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.y,
			z: self.z,
			w: self.z,
		}
	}
	pub fn xyzw(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.y,
			z: self.z,
			w: self.w,
		}
	}
	pub fn xywx(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.y,
			z: self.w,
			w: self.x,
		}
	}
	pub fn xywy(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.y,
			z: self.w,
			w: self.y,
		}
	}
	pub fn xywz(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.y,
			z: self.w,
			w: self.z,
		}
	}
	pub fn xyww(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.y,
			z: self.w,
			w: self.w,
		}
	}
	pub fn xzxx(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.z,
			z: self.x,
			w: self.x,
		}
	}
	pub fn xzxy(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.z,
			z: self.x,
			w: self.y,
		}
	}
	pub fn xzxz(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.z,
			z: self.x,
			w: self.z,
		}
	}
	pub fn xzxw(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.z,
			z: self.x,
			w: self.w,
		}
	}
	pub fn xzyx(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.z,
			z: self.y,
			w: self.x,
		}
	}
	pub fn xzyy(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.z,
			z: self.y,
			w: self.y,
		}
	}
	pub fn xzyz(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.z,
			z: self.y,
			w: self.z,
		}
	}
	pub fn xzyw(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.z,
			z: self.y,
			w: self.w,
		}
	}
	pub fn xzzx(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.z,
			z: self.z,
			w: self.x,
		}
	}
	pub fn xzzy(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.z,
			z: self.z,
			w: self.y,
		}
	}
	pub fn xzzz(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.z,
			z: self.z,
			w: self.z,
		}
	}
	pub fn xzzw(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.z,
			z: self.z,
			w: self.w,
		}
	}
	pub fn xzwx(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.z,
			z: self.w,
			w: self.x,
		}
	}
	pub fn xzwy(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.z,
			z: self.w,
			w: self.y,
		}
	}
	pub fn xzwz(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.z,
			z: self.w,
			w: self.z,
		}
	}
	pub fn xzww(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.z,
			z: self.w,
			w: self.w,
		}
	}
	pub fn xwxx(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.w,
			z: self.x,
			w: self.x,
		}
	}
	pub fn xwxy(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.w,
			z: self.x,
			w: self.y,
		}
	}
	pub fn xwxz(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.w,
			z: self.x,
			w: self.z,
		}
	}
	pub fn xwxw(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.w,
			z: self.x,
			w: self.w,
		}
	}
	pub fn xwyx(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.w,
			z: self.y,
			w: self.x,
		}
	}
	pub fn xwyy(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.w,
			z: self.y,
			w: self.y,
		}
	}
	pub fn xwyz(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.w,
			z: self.y,
			w: self.z,
		}
	}
	pub fn xwyw(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.w,
			z: self.y,
			w: self.w,
		}
	}
	pub fn xwzx(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.w,
			z: self.z,
			w: self.x,
		}
	}
	pub fn xwzy(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.w,
			z: self.z,
			w: self.y,
		}
	}
	pub fn xwzz(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.w,
			z: self.z,
			w: self.z,
		}
	}
	pub fn xwzw(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.w,
			z: self.z,
			w: self.w,
		}
	}
	pub fn xwwx(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.w,
			z: self.w,
			w: self.x,
		}
	}
	pub fn xwwy(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.w,
			z: self.w,
			w: self.y,
		}
	}
	pub fn xwwz(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.w,
			z: self.w,
			w: self.z,
		}
	}
	pub fn xwww(&self) -> Vec4 {
		Vec4 {
			x: self.x,
			y: self.w,
			z: self.w,
			w: self.w,
		}
	}
	pub fn yxxx(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.x,
			z: self.x,
			w: self.x,
		}
	}
	pub fn yxxy(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.x,
			z: self.x,
			w: self.y,
		}
	}
	pub fn yxxz(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.x,
			z: self.x,
			w: self.z,
		}
	}
	pub fn yxxw(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.x,
			z: self.x,
			w: self.w,
		}
	}
	pub fn yxyx(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.x,
			z: self.y,
			w: self.x,
		}
	}
	pub fn yxyy(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.x,
			z: self.y,
			w: self.y,
		}
	}
	pub fn yxyz(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.x,
			z: self.y,
			w: self.z,
		}
	}
	pub fn yxyw(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.x,
			z: self.y,
			w: self.w,
		}
	}
	pub fn yxzx(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.x,
			z: self.z,
			w: self.x,
		}
	}
	pub fn yxzy(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.x,
			z: self.z,
			w: self.y,
		}
	}
	pub fn yxzz(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.x,
			z: self.z,
			w: self.z,
		}
	}
	pub fn yxzw(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.x,
			z: self.z,
			w: self.w,
		}
	}
	pub fn yxwx(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.x,
			z: self.w,
			w: self.x,
		}
	}
	pub fn yxwy(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.x,
			z: self.w,
			w: self.y,
		}
	}
	pub fn yxwz(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.x,
			z: self.w,
			w: self.z,
		}
	}
	pub fn yxww(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.x,
			z: self.w,
			w: self.w,
		}
	}
	pub fn yyxx(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.y,
			z: self.x,
			w: self.x,
		}
	}
	pub fn yyxy(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.y,
			z: self.x,
			w: self.y,
		}
	}
	pub fn yyxz(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.y,
			z: self.x,
			w: self.z,
		}
	}
	pub fn yyxw(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.y,
			z: self.x,
			w: self.w,
		}
	}
	pub fn yyyx(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.y,
			z: self.y,
			w: self.x,
		}
	}
	pub fn yyyy(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.y,
			z: self.y,
			w: self.y,
		}
	}
	pub fn yyyz(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.y,
			z: self.y,
			w: self.z,
		}
	}
	pub fn yyyw(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.y,
			z: self.y,
			w: self.w,
		}
	}
	pub fn yyzx(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.y,
			z: self.z,
			w: self.x,
		}
	}
	pub fn yyzy(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.y,
			z: self.z,
			w: self.y,
		}
	}
	pub fn yyzz(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.y,
			z: self.z,
			w: self.z,
		}
	}
	pub fn yyzw(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.y,
			z: self.z,
			w: self.w,
		}
	}
	pub fn yywx(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.y,
			z: self.w,
			w: self.x,
		}
	}
	pub fn yywy(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.y,
			z: self.w,
			w: self.y,
		}
	}
	pub fn yywz(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.y,
			z: self.w,
			w: self.z,
		}
	}
	pub fn yyww(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.y,
			z: self.w,
			w: self.w,
		}
	}
	pub fn yzxx(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.z,
			z: self.x,
			w: self.x,
		}
	}
	pub fn yzxy(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.z,
			z: self.x,
			w: self.y,
		}
	}
	pub fn yzxz(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.z,
			z: self.x,
			w: self.z,
		}
	}
	pub fn yzxw(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.z,
			z: self.x,
			w: self.w,
		}
	}
	pub fn yzyx(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.z,
			z: self.y,
			w: self.x,
		}
	}
	pub fn yzyy(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.z,
			z: self.y,
			w: self.y,
		}
	}
	pub fn yzyz(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.z,
			z: self.y,
			w: self.z,
		}
	}
	pub fn yzyw(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.z,
			z: self.y,
			w: self.w,
		}
	}
	pub fn yzzx(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.z,
			z: self.z,
			w: self.x,
		}
	}
	pub fn yzzy(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.z,
			z: self.z,
			w: self.y,
		}
	}
	pub fn yzzz(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.z,
			z: self.z,
			w: self.z,
		}
	}
	pub fn yzzw(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.z,
			z: self.z,
			w: self.w,
		}
	}
	pub fn yzwx(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.z,
			z: self.w,
			w: self.x,
		}
	}
	pub fn yzwy(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.z,
			z: self.w,
			w: self.y,
		}
	}
	pub fn yzwz(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.z,
			z: self.w,
			w: self.z,
		}
	}
	pub fn yzww(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.z,
			z: self.w,
			w: self.w,
		}
	}
	pub fn ywxx(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.w,
			z: self.x,
			w: self.x,
		}
	}
	pub fn ywxy(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.w,
			z: self.x,
			w: self.y,
		}
	}
	pub fn ywxz(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.w,
			z: self.x,
			w: self.z,
		}
	}
	pub fn ywxw(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.w,
			z: self.x,
			w: self.w,
		}
	}
	pub fn ywyx(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.w,
			z: self.y,
			w: self.x,
		}
	}
	pub fn ywyy(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.w,
			z: self.y,
			w: self.y,
		}
	}
	pub fn ywyz(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.w,
			z: self.y,
			w: self.z,
		}
	}
	pub fn ywyw(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.w,
			z: self.y,
			w: self.w,
		}
	}
	pub fn ywzx(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.w,
			z: self.z,
			w: self.x,
		}
	}
	pub fn ywzy(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.w,
			z: self.z,
			w: self.y,
		}
	}
	pub fn ywzz(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.w,
			z: self.z,
			w: self.z,
		}
	}
	pub fn ywzw(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.w,
			z: self.z,
			w: self.w,
		}
	}
	pub fn ywwx(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.w,
			z: self.w,
			w: self.x,
		}
	}
	pub fn ywwy(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.w,
			z: self.w,
			w: self.y,
		}
	}
	pub fn ywwz(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.w,
			z: self.w,
			w: self.z,
		}
	}
	pub fn ywww(&self) -> Vec4 {
		Vec4 {
			x: self.y,
			y: self.w,
			z: self.w,
			w: self.w,
		}
	}
	pub fn zxxx(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.x,
			z: self.x,
			w: self.x,
		}
	}
	pub fn zxxy(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.x,
			z: self.x,
			w: self.y,
		}
	}
	pub fn zxxz(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.x,
			z: self.x,
			w: self.z,
		}
	}
	pub fn zxxw(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.x,
			z: self.x,
			w: self.w,
		}
	}
	pub fn zxyx(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.x,
			z: self.y,
			w: self.x,
		}
	}
	pub fn zxyy(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.x,
			z: self.y,
			w: self.y,
		}
	}
	pub fn zxyz(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.x,
			z: self.y,
			w: self.z,
		}
	}
	pub fn zxyw(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.x,
			z: self.y,
			w: self.w,
		}
	}
	pub fn zxzx(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.x,
			z: self.z,
			w: self.x,
		}
	}
	pub fn zxzy(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.x,
			z: self.z,
			w: self.y,
		}
	}
	pub fn zxzz(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.x,
			z: self.z,
			w: self.z,
		}
	}
	pub fn zxzw(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.x,
			z: self.z,
			w: self.w,
		}
	}
	pub fn zxwx(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.x,
			z: self.w,
			w: self.x,
		}
	}
	pub fn zxwy(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.x,
			z: self.w,
			w: self.y,
		}
	}
	pub fn zxwz(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.x,
			z: self.w,
			w: self.z,
		}
	}
	pub fn zxww(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.x,
			z: self.w,
			w: self.w,
		}
	}
	pub fn zyxx(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.y,
			z: self.x,
			w: self.x,
		}
	}
	pub fn zyxy(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.y,
			z: self.x,
			w: self.y,
		}
	}
	pub fn zyxz(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.y,
			z: self.x,
			w: self.z,
		}
	}
	pub fn zyxw(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.y,
			z: self.x,
			w: self.w,
		}
	}
	pub fn zyyx(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.y,
			z: self.y,
			w: self.x,
		}
	}
	pub fn zyyy(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.y,
			z: self.y,
			w: self.y,
		}
	}
	pub fn zyyz(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.y,
			z: self.y,
			w: self.z,
		}
	}
	pub fn zyyw(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.y,
			z: self.y,
			w: self.w,
		}
	}
	pub fn zyzx(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.y,
			z: self.z,
			w: self.x,
		}
	}
	pub fn zyzy(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.y,
			z: self.z,
			w: self.y,
		}
	}
	pub fn zyzz(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.y,
			z: self.z,
			w: self.z,
		}
	}
	pub fn zyzw(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.y,
			z: self.z,
			w: self.w,
		}
	}
	pub fn zywx(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.y,
			z: self.w,
			w: self.x,
		}
	}
	pub fn zywy(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.y,
			z: self.w,
			w: self.y,
		}
	}
	pub fn zywz(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.y,
			z: self.w,
			w: self.z,
		}
	}
	pub fn zyww(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.y,
			z: self.w,
			w: self.w,
		}
	}
	pub fn zzxx(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.z,
			z: self.x,
			w: self.x,
		}
	}
	pub fn zzxy(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.z,
			z: self.x,
			w: self.y,
		}
	}
	pub fn zzxz(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.z,
			z: self.x,
			w: self.z,
		}
	}
	pub fn zzxw(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.z,
			z: self.x,
			w: self.w,
		}
	}
	pub fn zzyx(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.z,
			z: self.y,
			w: self.x,
		}
	}
	pub fn zzyy(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.z,
			z: self.y,
			w: self.y,
		}
	}
	pub fn zzyz(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.z,
			z: self.y,
			w: self.z,
		}
	}
	pub fn zzyw(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.z,
			z: self.y,
			w: self.w,
		}
	}
	pub fn zzzx(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.z,
			z: self.z,
			w: self.x,
		}
	}
	pub fn zzzy(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.z,
			z: self.z,
			w: self.y,
		}
	}
	pub fn zzzz(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.z,
			z: self.z,
			w: self.z,
		}
	}
	pub fn zzzw(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.z,
			z: self.z,
			w: self.w,
		}
	}
	pub fn zzwx(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.z,
			z: self.w,
			w: self.x,
		}
	}
	pub fn zzwy(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.z,
			z: self.w,
			w: self.y,
		}
	}
	pub fn zzwz(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.z,
			z: self.w,
			w: self.z,
		}
	}
	pub fn zzww(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.z,
			z: self.w,
			w: self.w,
		}
	}
	pub fn zwxx(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.w,
			z: self.x,
			w: self.x,
		}
	}
	pub fn zwxy(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.w,
			z: self.x,
			w: self.y,
		}
	}
	pub fn zwxz(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.w,
			z: self.x,
			w: self.z,
		}
	}
	pub fn zwxw(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.w,
			z: self.x,
			w: self.w,
		}
	}
	pub fn zwyx(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.w,
			z: self.y,
			w: self.x,
		}
	}
	pub fn zwyy(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.w,
			z: self.y,
			w: self.y,
		}
	}
	pub fn zwyz(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.w,
			z: self.y,
			w: self.z,
		}
	}
	pub fn zwyw(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.w,
			z: self.y,
			w: self.w,
		}
	}
	pub fn zwzx(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.w,
			z: self.z,
			w: self.x,
		}
	}
	pub fn zwzy(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.w,
			z: self.z,
			w: self.y,
		}
	}
	pub fn zwzz(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.w,
			z: self.z,
			w: self.z,
		}
	}
	pub fn zwzw(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.w,
			z: self.z,
			w: self.w,
		}
	}
	pub fn zwwx(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.w,
			z: self.w,
			w: self.x,
		}
	}
	pub fn zwwy(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.w,
			z: self.w,
			w: self.y,
		}
	}
	pub fn zwwz(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.w,
			z: self.w,
			w: self.z,
		}
	}
	pub fn zwww(&self) -> Vec4 {
		Vec4 {
			x: self.z,
			y: self.w,
			z: self.w,
			w: self.w,
		}
	}
	pub fn wxxx(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.x,
			z: self.x,
			w: self.x,
		}
	}
	pub fn wxxy(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.x,
			z: self.x,
			w: self.y,
		}
	}
	pub fn wxxz(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.x,
			z: self.x,
			w: self.z,
		}
	}
	pub fn wxxw(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.x,
			z: self.x,
			w: self.w,
		}
	}
	pub fn wxyx(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.x,
			z: self.y,
			w: self.x,
		}
	}
	pub fn wxyy(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.x,
			z: self.y,
			w: self.y,
		}
	}
	pub fn wxyz(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.x,
			z: self.y,
			w: self.z,
		}
	}
	pub fn wxyw(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.x,
			z: self.y,
			w: self.w,
		}
	}
	pub fn wxzx(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.x,
			z: self.z,
			w: self.x,
		}
	}
	pub fn wxzy(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.x,
			z: self.z,
			w: self.y,
		}
	}
	pub fn wxzz(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.x,
			z: self.z,
			w: self.z,
		}
	}
	pub fn wxzw(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.x,
			z: self.z,
			w: self.w,
		}
	}
	pub fn wxwx(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.x,
			z: self.w,
			w: self.x,
		}
	}
	pub fn wxwy(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.x,
			z: self.w,
			w: self.y,
		}
	}
	pub fn wxwz(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.x,
			z: self.w,
			w: self.z,
		}
	}
	pub fn wxww(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.x,
			z: self.w,
			w: self.w,
		}
	}
	pub fn wyxx(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.y,
			z: self.x,
			w: self.x,
		}
	}
	pub fn wyxy(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.y,
			z: self.x,
			w: self.y,
		}
	}
	pub fn wyxz(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.y,
			z: self.x,
			w: self.z,
		}
	}
	pub fn wyxw(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.y,
			z: self.x,
			w: self.w,
		}
	}
	pub fn wyyx(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.y,
			z: self.y,
			w: self.x,
		}
	}
	pub fn wyyy(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.y,
			z: self.y,
			w: self.y,
		}
	}
	pub fn wyyz(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.y,
			z: self.y,
			w: self.z,
		}
	}
	pub fn wyyw(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.y,
			z: self.y,
			w: self.w,
		}
	}
	pub fn wyzx(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.y,
			z: self.z,
			w: self.x,
		}
	}
	pub fn wyzy(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.y,
			z: self.z,
			w: self.y,
		}
	}
	pub fn wyzz(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.y,
			z: self.z,
			w: self.z,
		}
	}
	pub fn wyzw(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.y,
			z: self.z,
			w: self.w,
		}
	}
	pub fn wywx(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.y,
			z: self.w,
			w: self.x,
		}
	}
	pub fn wywy(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.y,
			z: self.w,
			w: self.y,
		}
	}
	pub fn wywz(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.y,
			z: self.w,
			w: self.z,
		}
	}
	pub fn wyww(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.y,
			z: self.w,
			w: self.w,
		}
	}
	pub fn wzxx(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.z,
			z: self.x,
			w: self.x,
		}
	}
	pub fn wzxy(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.z,
			z: self.x,
			w: self.y,
		}
	}
	pub fn wzxz(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.z,
			z: self.x,
			w: self.z,
		}
	}
	pub fn wzxw(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.z,
			z: self.x,
			w: self.w,
		}
	}
	pub fn wzyx(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.z,
			z: self.y,
			w: self.x,
		}
	}
	pub fn wzyy(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.z,
			z: self.y,
			w: self.y,
		}
	}
	pub fn wzyz(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.z,
			z: self.y,
			w: self.z,
		}
	}
	pub fn wzyw(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.z,
			z: self.y,
			w: self.w,
		}
	}
	pub fn wzzx(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.z,
			z: self.z,
			w: self.x,
		}
	}
	pub fn wzzy(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.z,
			z: self.z,
			w: self.y,
		}
	}
	pub fn wzzz(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.z,
			z: self.z,
			w: self.z,
		}
	}
	pub fn wzzw(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.z,
			z: self.z,
			w: self.w,
		}
	}
	pub fn wzwx(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.z,
			z: self.w,
			w: self.x,
		}
	}
	pub fn wzwy(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.z,
			z: self.w,
			w: self.y,
		}
	}
	pub fn wzwz(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.z,
			z: self.w,
			w: self.z,
		}
	}
	pub fn wzww(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.z,
			z: self.w,
			w: self.w,
		}
	}
	pub fn wwxx(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.w,
			z: self.x,
			w: self.x,
		}
	}
	pub fn wwxy(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.w,
			z: self.x,
			w: self.y,
		}
	}
	pub fn wwxz(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.w,
			z: self.x,
			w: self.z,
		}
	}
	pub fn wwxw(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.w,
			z: self.x,
			w: self.w,
		}
	}
	pub fn wwyx(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.w,
			z: self.y,
			w: self.x,
		}
	}
	pub fn wwyy(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.w,
			z: self.y,
			w: self.y,
		}
	}
	pub fn wwyz(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.w,
			z: self.y,
			w: self.z,
		}
	}
	pub fn wwyw(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.w,
			z: self.y,
			w: self.w,
		}
	}
	pub fn wwzx(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.w,
			z: self.z,
			w: self.x,
		}
	}
	pub fn wwzy(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.w,
			z: self.z,
			w: self.y,
		}
	}
	pub fn wwzz(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.w,
			z: self.z,
			w: self.z,
		}
	}
	pub fn wwzw(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.w,
			z: self.z,
			w: self.w,
		}
	}
	pub fn wwwx(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.w,
			z: self.w,
			w: self.x,
		}
	}
	pub fn wwwy(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.w,
			z: self.w,
			w: self.y,
		}
	}
	pub fn wwwz(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.w,
			z: self.w,
			w: self.z,
		}
	}
	pub fn wwww(&self) -> Vec4 {
		Vec4 {
			x: self.w,
			y: self.w,
			z: self.w,
			w: self.w,
		}
	}
}

impl Add<Vec4> for Vec4 {
	type Output = Vec4;

	fn add(self, other: Vec4) -> Vec4 {
		Vec4 {
			x: self.x + other.x,
			y: self.y + other.y,
			z: self.z + other.z,
			w: self.w + other.w,
		}
	}
}

impl AddAssign for Vec4 {
	fn add_assign(&mut self, other: Vec4) {
		self.x += other.x;
		self.y += other.y;
		self.z += other.z;
		self.w += other.w;
	}
}

impl Sub<Vec4> for Vec4 {
	type Output = Vec4;

	fn sub(self, other: Vec4) -> Vec4 {
		Vec4 {
			x: self.x - other.x,
			y: self.y - other.y,
			z: self.z - other.z,
			w: self.w - other.w,
		}
	}
}

impl SubAssign for Vec4 {
	fn sub_assign(&mut self, other: Vec4) {
		self.x -= other.x;
		self.y -= other.y;
		self.z -= other.z;
		self.w -= other.w;
	}
}

impl Mul<f32> for Vec4 {
	type Output = Vec4;

	fn mul(self, other: f32) -> Vec4 {
		Vec4 {
			x: self.x * other,
			y: self.y * other,
			z: self.z * other,
			w: self.w * other,
		}
	}
}

impl Mul<Vec4> for f32 {
	type Output = Vec4;

	fn mul(self, other: Vec4) -> Vec4 {
		Vec4 {
			x: self * other.x,
			y: self * other.y,
			z: self * other.z,
			w: self * other.w,
		}
	}
}

impl MulAssign<f32> for Vec4 {
	fn mul_assign(&mut self, other: f32) {
		self.x *= other;
		self.y *= other;
		self.z *= other;
		self.w *= other;
	}
}

impl Into<[f32;4]> for Vec4 {
	fn into(self) -> [f32;4] {
		[self.x, self.y, self.z, self.w]
	}
}

impl InnerSpace for Vec2 {
	fn dot(&self, other: &Self) -> f32 {
		self.x * other.x + self.y * other.y
	}

	fn dist(&self, other: &Self) -> f32 {
		Vec2 {
			x: other.x - self.x,
			y: other.y - self.y,
		}
			.length()
	}

	fn v_angle(&self, other: &Self) -> f32 {
		//debug!("{:?}", dot(self,other)/(self.length()*other.length()));
		acos(dot(self, other) / (self.length() * other.length()))
	}
}

impl InnerSpace for Vec3 {
	fn dot(&self, other: &Self) -> f32 {
		self.x * other.x + self.y * other.y + self.z * other.z
	}

	fn dist(&self, other: &Self) -> f32 {
		Vec3 {
			x: other.x - self.x,
			y: other.y - self.y,
			z: other.z - self.z,
		}
			.length()
	}

	fn v_angle(&self, other: &Self) -> f32 {
		acos(dot(self, other) / (self.length() * other.length()))
	}
}

impl InnerSpace for Vec4 {
	fn dot(&self, other: &Self) -> f32 {
		self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
	}

	fn dist(&self, other: &Self) -> f32 {
		Vec4 {
			x: other.x - self.x,
			y: other.y - self.y,
			z: other.z - self.z,
			w: other.w - self.w,
		}
			.length()
	}

	fn v_angle(&self, other: &Self) -> f32 {
		acos(dot(self, other) / (self.length() * other.length()))
	}
}

// ##################################################
// #              VECTOR FUNCTIONS                  #
// ##################################################

pub fn dot<T: InnerSpace>(v1: &T, v2: &T) -> f32 {
	v1.dot(v2)
}

pub fn cross(v1: Vec3, v2: Vec3) -> Vec3 {
	Vec3 {
		x: v1.y * v2.z - v1.z * v2.y,
		y: v1.z * v2.x - v1.x * v2.z,
		z: v1.x * v2.y - v2.y * v1.x,
	}
}

pub fn v_dist<T: InnerSpace>(v1: &T, v2: &T) -> f32 {
	v1.dist(v2)
}

pub fn v_angle<T: InnerSpace>(v1: &T, v2: &T) -> f32 {
	v1.v_angle(v2)
}
