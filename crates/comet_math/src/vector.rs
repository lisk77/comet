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

impl Neg for Vec3 {
	type Output = Vec3;

	fn neg(self) -> Vec3 {
		Vec3 {
			x: -self.x,
			y: -self.y,
			z: -self.z,
		}
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
		acos(self.dot(other) / (self.length() * other.length()))
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
		acos(self.dot(other) / (self.length() * other.length()))
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
		acos(self.dot(other) / (self.length() * other.length()))
	}
}

macro_rules! generate_swizzles2 {
    ($VecType:ident, $($name:ident => ($a:ident, $b:ident)),* $(,)?) => {
        impl $VecType {
            $(
                pub fn $name(&self) -> Self {
                    Self {
                        x: self.$a,
                        y: self.$b,
                    }
                }
            )*
        }
    };
}

macro_rules! generate_swizzles3 {
    ($VecType:ident, $($name:ident => ($a:ident, $b:ident, $c:ident)),* $(,)?) => {
        impl $VecType {
            $(
                pub fn $name(&self) -> Self {
                    Self {
                        x: self.$a,
                        y: self.$b,
						z: self.$c
                    }
                }
            )*
        }
    };
}

macro_rules! generate_swizzles4 {
    ($VecType:ident, $($name:ident => ($a:ident, $b:ident, $c:ident, $d:ident)),* $(,)?) => {
        impl $VecType {
            $(
                pub fn $name(&self) -> Self {
                    Self {
                        x: self.$a,
                        y: self.$b,
						z: self.$c,
						w: self.$d
                    }
                }
            )*
        }
    };
}


generate_swizzles2!(Vec2,
    xx => (x, x), xy => (x, y),
    yx => (y, x), yy => (y, y)
);

generate_swizzles3!(Vec3,
	xxx => (x, x, x), xxy => (x, x, y), xxz => (x, x, z),
	xyx => (x, y, x), xyy => (x, y, y), xyz => (x, y, z),
	xzx => (x, z, x), xzy => (x, z, y), xzz => (x, z, z),
	yxx => (y, x, x), yxy => (y, x, y), yxz => (y, x, z),
	yyx => (y, y, x), yyy => (y, y, y), yyz => (y, y, z),
	yzx => (y, z, x), yzy => (y, z, y), yzz => (y, z, z),
	zxx => (z, x, x), zxy => (z, x, y), zxz => (z, x, z),
	zyx => (z, y, x), zyy => (z, y, y), zyz => (z, y, z),
	zzx => (z, z, x), zzy => (z, z, y), zzz => (z, z, z)
);

generate_swizzles4!(Vec4,
	xxxx => (x, x, x, x), xxxy => (x, x, x, y), xxxz => (x, x, x, z), xxxw => (x, x, x, w),
	xxyx => (x, x, y, x), xxyy => (x, x, y, y), xxyz => (x, x, y, z), xxyw => (x, x, y, w),
	xxzx => (x, x, z, x), xxzy => (x, x, z, y), xxzz => (x, x, z, z), xxzw => (x, x, z, w),
	xxwx => (x, x, w, x), xxwy => (x, x, w, y), xxwz => (x, x, w, z), xxww => (x, x, w, w),
	xyxx => (x, y, x, x), xyxy => (x, y, x, y), xyxz => (x, y, x, z), xyxw => (x, y, x, w),
	xyyx => (x, y, y, x), xyyy => (x, y, y, y), xyyz => (x, y, y, z), xyyw => (x, y, y, w),
	xyzx => (x, y, z, x), xyzy => (x, y, z, y), xyzz => (x, y, z, z), xyzw => (x, y, z, w),
	xywx => (x, y, w, x), xywy => (x, y, w, y), xywz => (x, y, w, z), xyww => (x, y, w, w),
	xzxx => (x, z, x, x), xzxy => (x, z, x, y), xzxz => (x, z, x, z), xzxw => (x, z, x, w),
	xzyx => (x, z, y, x), xzyy => (x, z, y, y), xzyz => (x, z, y, z), xzyw => (x, z, y, w),
	xzzx => (x, z, z, x), xzzy => (x, z, z, y), xzzz => (x, z, z, z), xzzw => (x, z, z, w),
	xzwx => (x, z, w, x), xzwy => (x, z, w, y), xzwz => (x, z, w, z), xzww => (x, z, w, w),
	xwxx => (x, w, x, x), xwxy => (x, w, x, y), xwxz => (x, w, x, z), xwxw => (x, w, x, w),
	xwyx => (x, w, y, x), xwyy => (x, w, y, y), xwyz => (x, w, y, z), xwyw => (x, w, y, w),
	xwzx => (x, w, z, x), xwzy => (x, w, z, y), xwzz => (x, w, z, z), xwzw => (x, w, z, w),
	xwwx => (x, w, w, x), xwwy => (x, w, w, y), xwwz => (x, w, w, z), xwww => (x, w, w, w),
	yxxx => (y, x, x, x), yxxy => (y, x, x, y), yxxz => (y, x, x, z), yxxw => (y, x, x, w),
	yxyx => (y, x, y, x), yxyy => (y, x, y, y), yxyz => (y, x, y, z), yxyw => (y, x, y, w),
	yxzx => (y, x, z, x), yxzy => (y, x, z, y), yxzz => (y, x, z, z), yxzw => (y, x, z, w),
	yxwx => (y, x, w, x), yxwy => (y, x, w, y), yxwz => (y, x, w, z), yxww => (y, x, w, w),
	yyxx => (y, y, x, x), yyxy => (y, y, x, y), yyxz => (y, y, x, z), yyxw => (y, y, x, w),
	yyyx => (y, y, y, x), yyyy => (y, y, y, y), yyyz => (y, y, y, z), yyyw => (y, y, y, w),
	yyzx => (y, y, z, x), yyzy => (y, y, z, y), yyzz => (y, y, z, z), yyzw => (y, y, z, w),
	yywx => (y, y, w, x), yywy => (y, y, w, y), yywz => (y, y, w, z), yyww => (y, y, w, w)
);