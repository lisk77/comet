use std::ops::Mul;

use crate::vector::Vec3;

/// Representation of a quaternion in scalar/vector form
pub struct Quat {
	pub s: f32,
	pub v: Vec3,
}

impl Quat {
	pub const fn zero() -> Self {
		Self {
			s: 0.0,
			v: Vec3 {
				x: 0.0,
				y: 0.0,
				z: 0.0,
			},
		}
	}
	pub const fn new(s: f32, v: Vec3) -> Self {
		Self { s, v }
	}

	pub fn conjugate(&self) -> Self {
		Self {
			s: self.s,
			v: self.v * (-1.0),
		}
	}

	pub fn normalize(&self) -> Self {
		let inverse_squareroot = 1.0/(self.s*self.s + self.v.x*self.v.x + self.v.y*self.v.y + self.v.z*self.v.z).sqrt();
		Self::new(self.s*inverse_squareroot, self.v*inverse_squareroot)
	}

	pub fn into_vec(&self) -> Vec3 {
		Vec3 {
			x: self.v.x,
			y: self.v.y,
			z: self.v.z,
		}
	}
}

impl Mul<Quat> for Quat {
	type Output = Quat;

	fn mul(self, other: Quat) -> Quat {
		Quat {
			s: self.s*other.s - self.v.x*other.v.x - self.v.y*other.v.y - self.v.z*other.v.z,
			v: Vec3 {
				x: self.s*other.v.x + self.v.x*other.s + self.v.y*other.v.z - self.v.z*other.v.y,
				y: self.s*other.v.y + self.v.y*other.s + self.v.z*other.v.x - self.v.x*other.v.z,
				z: self.s*other.v.z + self.v.z*other.s + self.v.x*other.v.y - self.v.y*other.v.x,
			}
		}
	}
}
