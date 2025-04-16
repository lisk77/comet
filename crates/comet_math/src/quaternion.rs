use std::ops::*;
use std::ops::Mul;
use crate::vector::v3;

/// Representation of a quaternion in scalar/vector form
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat {
	pub s: f32,
	pub v: v3,
}

impl Quat {
	pub const fn zero() -> Self {
		Self {
			s: 0.0,
			v: v3 {
				x: 0.0,
				y: 0.0,
				z: 0.0,
			},
		}
	}
	pub const fn new(s: f32, v: v3) -> Self {
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

	pub fn into_vec(&self) -> v3 {
		v3 {
			x: self.v.x,
			y: self.v.y,
			z: self.v.z,
		}
	}
}

impl Add<Quat> for Quat {
	type Output = Quat;

	fn add(self, other: Quat) -> Quat {
		Quat {
			s: self.s + other.s,
			v: self.v + other.v,
		}
	}
}

impl Sub<Quat> for Quat {
	type Output = Quat;

	fn sub(self, other: Quat) -> Quat {
		Quat {
			s: self.s - other.s,
			v: self.v - other.v,
		}
	}
}

impl Neg for Quat {
	type Output = Quat;

	fn neg(self) -> Quat {
		Quat {
			s: -self.s,
			v: -self.v,
		}
	}
}

impl Add<f32> for Quat {
	type Output = Quat;

	fn add(self, scalar: f32) -> Quat {
		Quat {
			s: self.s + scalar,
			v: self.v,
		}
	}
}

impl Sub<f32> for Quat {
	type Output = Quat;

	fn sub(self, scalar: f32) -> Quat {
		Quat {
			s: self.s - scalar,
			v: self.v,
		}
	}
}

impl Mul<Quat> for f32 {
	type Output = Quat;

	fn mul(self, quat: Quat) -> Quat {
		Quat {
			s: self*quat.s,
			v: self*quat.v,
		}
	}
}

impl Mul<Quat> for Quat {
	type Output = Quat;

	fn mul(self, other: Quat) -> Quat {
		Quat {
			s: self.s*other.s - self.v.x*other.v.x - self.v.y*other.v.y - self.v.z*other.v.z,
			v: v3 {
				x: self.s*other.v.x + self.v.x*other.s + self.v.y*other.v.z - self.v.z*other.v.y,
				y: self.s*other.v.y + self.v.y*other.s + self.v.z*other.v.x - self.v.x*other.v.z,
				z: self.s*other.v.z + self.v.z*other.s + self.v.x*other.v.y - self.v.y*other.v.x,
			}
		}
	}
}

impl Mul<f32> for Quat {
	type Output = Quat;

	fn mul(self, scalar: f32) -> Quat {
		Quat {
			s: self.s*scalar,
			v: self.v*scalar,
		}
	}
}

impl Div<f32> for Quat {
	type Output = Quat;

	fn div(self, scalar: f32) -> Quat {
		Quat {
			s: self.s/scalar,
			v: self.v/scalar,
		}
	}
}