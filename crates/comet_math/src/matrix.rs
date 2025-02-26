use std::ops::{Add, Sub, Mul, Div};
use crate::{cross, dot, Point3};
use crate::vector::{Vec2, Vec3, Vec4};

trait LinearTransformation {
	fn det(&self) -> f32;
}

// ##################################################
// #                   MATRIX 2D                    #
// ##################################################


/// Representation of a 2x2 Matrix
#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
pub struct Mat2 {
	x00: f32,
	x01: f32,
	x10: f32,
	x11: f32,
}

impl Mat2 {
	pub const ZERO: Mat2 = Mat2 {
		x00: 0.0, x01: 0.0,
		x10: 0.0, x11: 0.0
	};

	pub const IDENTITY: Mat2 = Mat2 {
		x00: 1.0, x01: 0.0,
		x10: 0.0, x11: 1.0
	};
	pub const fn new(x00: f32, x01: f32, x10: f32, x11: f32) -> Self {
		Self {
			x00, x01,
			x10, x11
		}
	}

	pub fn from_rows(row1: Vec2, row2: Vec2) -> Self {
		Self {
			x00: row1.x(), x01: row1.y(),
			x10: row2.x(), x11: row2.y()
		}
	}

	pub fn from_cols(col1: Vec2, col2: Vec2) -> Self {
		Self {
			x00: col1.x(), x01: col2.x(),
			x10: col1.y(), x11: col2.y()
		}
	}

	pub fn get(&self, row: usize, col: usize) -> Option<f32> {
		assert!(row <= 1, "This row ({}) is out of bounds! Bounds: 0..1", row);
		assert!(col <= 1, "This row ({}) is out of bounds! Bounds: 0..1", col);

		match (row, col) {
			(0, 0) => Some(self.x00),
			(0, 1) => Some(self.x01),
			(1, 0) => Some(self.x10),
			(1, 1) => Some(self.x11),
			_ => None,
		}
	}

	pub fn get_row(&self, row: usize) -> Option<Vec2> {
		assert!(row <= 1, "This row ({}) is out of bounds! Bounds: 0..1", row);
		match row {
			0 => Some(Vec2::new(self.x00, self.x01)),
			1 => Some(Vec2::new(self.x10, self.x11)),
			_ => None
		}
	}

	pub fn get_col(&self, col: usize) -> Option<Vec2> {
		assert!(col <= 1, "This row ({}) is out of bounds! Bounds: 0..1", col);
		match col {
			0 => Some(Vec2::new(self.x00, self.x10)),
			1 => Some(Vec2::new(self.x01, self.x11)),
			_ => None
		}
	}

	pub fn set(&mut self, row: usize, col: usize, element: f32) {
		assert!(row <= 1, "This row ({}) is out of bounds! Bounds: 0..1", row);
		assert!(col <= 1, "This row ({}) is out of bounds! Bounds: 0..1", col);

		match (row, col) {
			(0,0) => self.x00 = element,
			(0,1) => self.x01 = element,
			(1,0) => self.x10 = element,
			(1,1) => self.x11 = element,
			_ => {}
		}
	}

	pub fn set_row(&mut self, row: usize, row_content: Vec2) {
		assert!(row <= 1, "This row ({}) is out of bounds! Bounds: 0..1", row);

		match row {
			0 => { self.x00 = row_content.x(); self.x01 = row_content.y(); },
			1 => { self.x10 = row_content.x(); self.x11 = row_content.y(); },
			_ => {}
		}
	}

	pub fn set_col(&mut self, col: usize, col_content: Vec2) {
		assert!(col <= 1, "This row ({}) is out of bounds! Bounds: 0..1", col);

		match col {
			0 => { self.x00 = col_content.x(); self.x10 = col_content.y(); },
			1 => { self.x01 = col_content.x(); self.x11 = col_content.y(); },
			_ => {}
		}
	}

	pub fn det(&self) -> f32 {
		self.x00 * self.x11
			- self.x01 * self.x10
	}

	pub fn transpose(&self) -> Self {
		Self {
			x00: self.x00, x01: self.x10,
			x10: self.x01, x11: self.x11
		}
	}

	pub fn swap_rows(&mut self, row1: usize, row2: usize) {
		let tmp = self.get_row(row1).expect(format!("This row ({}) is out of bounds! Bounds: 0..1", row1).as_str());
		self.set_row(row1, self.get_row(row2).expect(format!("This row ({}) is out of bounds! Bounds: 0..1", row2).as_str()));
		self.set_row(row2, tmp);
	}

	pub fn swap_cols(&mut self, col1: usize, col2: usize) {
		let tmp = self.get_col(col1).expect(format!("This row ({}) is out of bounds! Bounds: 0..1", col1).as_str());
		self.set_col(col1, self.get_col(col2).expect(format!("This row ({}) is out of bounds! Bounds: 0..1", col2).as_str()));
		self.set_col(col2, tmp);
	}
}

impl Add<Mat2> for Mat2 {
	type Output = Self;

	fn add(self, other: Mat2) -> Self {
		Self {
			x00: self.x00 + other.x00, x01: self.x01 + other.x01,
			x10: self.x10 + other.x10, x11: self.x11 + other.x11
		}
	}
}

impl Sub<Mat2> for Mat2 {
	type Output = Self;

	fn sub(self, other: Mat2) -> Self {
		Self {
			x00: self.x00 - other.x00, x01: self.x01 - other.x01,
			x10: self.x10 - other.x10, x11: self.x11 - other.x11
		}
	}
}

impl Mul<f32> for Mat2 {
	type Output = Self;

	fn mul(self, other: f32) -> Self {
		Self {
			x00: self.x00 * other, x01: self.x01 * other,
			x10: self.x10 * other, x11: self.x11 * other
		}
	}
}

impl Mul<Mat2> for f32 {
	type Output = Mat2;

	fn mul(self, other: Mat2) -> Mat2 {
		Mat2 {
			x00: self * other.x00, x01: self * other.x01,
			x10: self * other.x10, x11: self * other.x11
		}
	}
}

impl Mul<Mat2> for Mat2 {
	type Output = Self;

	fn mul(self, other: Mat2) -> Self {
		Self {
			x00: self.x00 * other.x00 + self.x01 * other.x10,
			x01: self.x00 * other.x01 + self.x01 * other.x11,
			x10: self.x10 * other.x00 + self.x11 * other.x10,
			x11: self.x10 * other.x01 + self.x11 * other.x11
		}
	}
}

impl Mul<Vec2> for Mat2 {
	type Output = Vec2;

	fn mul(self, other: Vec2) -> Vec2 {
		Vec2::new(
			self.x00 * other.x() + self.x01 * other.y(),
			self.x10 * other.x() + self.x11 * other.y()
		)
	}
}

impl Div<f32> for Mat2 {
	type Output = Self;

	fn div(self, other: f32) -> Self {
		let inv = 1.0 / other;
		inv * self
	}
}

/// [WARN]: This will return a column-major array for wgpu use!
impl Into<[[f32; 2]; 2]> for Mat2 {
	fn into(self) -> [[f32; 2]; 2] {
		[
			[self.x00, self.x10],
			[self.x01, self.x11],
		]
	}
}

// ##################################################
// #                   MATRIX 3D                    #
// ##################################################

/// Representation of a 3x3 Matrix
#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
pub struct Mat3 {
	x00: f32,
	x01: f32,
	x02: f32,
	x10: f32,
	x11: f32,
	x12: f32,
	x20: f32,
	x21: f32,
	x22: f32
}

impl Mat3 {
	pub const ZERO: Mat3 = Mat3 {
		x00: 0.0, x01: 0.0, x02: 0.0,
		x10: 0.0, x11: 0.0, x12: 0.0,
		x20: 0.0, x21: 0.0, x22: 0.0
	};
	pub const IDENTITY: Mat3 = Mat3 {
		x00: 1.0, x01: 0.0, x02: 0.0,
		x10: 0.0, x11: 1.0, x12: 0.0,
		x20: 0.0, x21: 0.0, x22: 1.0
	};
	pub const fn new(x00: f32, x01: f32, x02: f32, x10: f32, x11: f32, x12: f32, x20: f32, x21: f32, x22: f32) -> Self {
		Self {
			x00, x01, x02,
			x10, x11, x12,
			x20, x21, x22
		}
	}

	pub fn from_rows(row1: Vec3, row2: Vec3, row3: Vec3) -> Self {
		Self {
			x00: row1.x(), x01: row1.y(), x02: row1.z(),
			x10: row2.x(), x11: row2.y(), x12: row2.z(),
			x20: row3.x(), x21: row3.y(), x22: row3.z()
		}
	}

	pub fn from_cols(col1: Vec3, col2: Vec3, col3: Vec3) -> Self {
		Self {
			x00: col1.x(), x01: col2.x(), x02: col3.x(),
			x10: col1.y(), x11: col2.y(), x12: col3.y(),
			x20: col1.z(), x21: col2.z(), x22: col3.z()
		}
	}

	pub fn get(&self, row: usize, col: usize) -> Option<f32> {
		assert!(row <= 2, "This row ({}) is out of bounds! Bounds: 0..2", row);
		assert!(col <= 2, "This row ({}) is out of bounds! Bounds: 0..2", col);
		match (row, col) {
			(0, 0) => Some(self.x00),
			(0, 1) => Some(self.x01),
			(0, 2) => Some(self.x02),
			(1, 0) => Some(self.x10),
			(1, 1) => Some(self.x11),
			(1, 2) => Some(self.x12),
			(2, 0) => Some(self.x20),
			(2, 1) => Some(self.x21),
			(2, 2) => Some(self.x22),
			_ => None,
		}
	}

	pub fn get_row(&self, row: usize) -> Option<Vec3> {
		assert!(row <= 2, "This row ({}) is out of bounds! Bounds: 0..2", row);
		match row {
			0 => Some(Vec3::new(self.x00, self.x01, self.x02)),
			1 => Some(Vec3::new(self.x10, self.x11, self.x12)),
			2 => Some(Vec3::new(self.x20, self.x21, self.x22)),
			_ => None
		}
	}

	pub fn get_col(&self, col: usize) -> Option<Vec3> {
		assert!(col <= 2, "This row ({}) is out of bounds! Bounds: 0..2", col);
		match col {
			0 => Some(Vec3::new(self.x00, self.x10, self.x20)),
			1 => Some(Vec3::new(self.x01, self.x11, self.x21)),
			2 => Some(Vec3::new(self.x02, self.x12, self.x22)),
			_ => None
		}
	}

	pub fn set(&mut self, row: usize, col: usize, element: f32) {
		assert!(row <= 2, "This row ({}) is out of bounds! Bounds: 0..2", row);
		assert!(col <= 2, "This row ({}) is out of bounds! Bounds: 0..2", col);

		match (row, col) {
			(0,0) => self.x00 = element,
			(0,1) => self.x01 = element,
			(0,2) => self.x02 = element,
			(1,0) => self.x10 = element,
			(1,1) => self.x11 = element,
			(1,2) => self.x12 = element,
			(2,0) => self.x20 = element,
			(2,1) => self.x21 = element,
			(2,2) => self.x22 = element,
			_ => {}
		}
	}

	pub fn set_row(&mut self, row: usize, row_content: Vec3) {
		assert!(row <= 2, "This row ({}) is out of bounds! Bounds: 0..2", row);
		match row {
			0 => { self.x00 = row_content.x; self.x01 = row_content.y; self.x02 = row_content.z; },
			1 => { self.x10 = row_content.x; self.x11 = row_content.y; self.x12 = row_content.z; },
			2 => { self.x20 = row_content.x; self.x21 = row_content.y; self.x22 = row_content.z; }
			_ => {}
		}
	}

	pub fn set_col(&mut self, col: usize, col_content: Vec3) {
		assert!(col <= 2, "This row ({}) is out of bounds! Bounds: 0..2", col);
		match col {
			0 => { self.x00 = col_content.x; self.x10 = col_content.y; self.x20 = col_content.z; },
			1 => { self.x01 = col_content.x; self.x11 = col_content.y; self.x21 = col_content.z; },
			2 => { self.x02 = col_content.x; self.x12 = col_content.y; self.x22 = col_content.z; }
			_ => {}
		}
	}

	pub fn det(&self) -> f32 {
		self.x00 * self.x11 * self.x22
			+ self.x01 * self.x12 * self.x20
			+ self.x02 * self.x10 * self.x21
			- self.x02 * self.x11 * self.x20
			- self.x01 * self.x10 * self.x22
			- self.x00 * self.x12 * self.x21
	}

	pub fn transpose(&self) -> Self {
		Self {
			x00: self.x00, x01: self.x10, x02: self.x20,
			x10: self.x01, x11: self.x11, x12: self.x21,
			x20: self.x02, x21: self.x12, x22: self.x22
		}
	}

	pub fn swap_rows(&mut self, row1: usize, row2: usize) {
		let tmp = self.get_row(row1).expect(format!("This row ({}) is out of bounds! Bounds: 0..2", row1).as_str());
		self.set_row(row1, self.get_row(row2).expect(format!("This row ({}) is out of bounds! Bounds: 0..2", row2).as_str()));
		self.set_row(row2, tmp);
	}

	pub fn swap_cols(&mut self, col1: usize, col2: usize) {
		let tmp = self.get_col(col1).expect(format!("This row ({}) is out of bounds! Bounds: 0..2", col1).as_str());
		self.set_col(col1, self.get_col(col2).expect(format!("This row ({}) is out of bounds! Bounds: 0..2", col2).as_str()));
		self.set_col(col2, tmp);
	}
}

impl Add<Mat3> for Mat3 {
	type Output = Self;

	fn add(self, other: Mat3) -> Self {
		Self {
			x00: self.x00 + other.x00, x01: self.x01 + other.x01, x02: self.x02 + other.x02,
			x10: self.x10 + other.x10, x11: self.x11 + other.x11, x12: self.x12 + other.x12,
			x20: self.x20 + other.x20, x21: self.x21 + other.x21, x22: self.x22 + other.x22
		}
	}
}

impl Sub<Mat3> for Mat3 {
	type Output = Self;

	fn sub(self, other: Mat3) -> Self {
		Self {
			x00: self.x00 - other.x00, x01: self.x01 - other.x01, x02: self.x02 - other.x02,
			x10: self.x10 - other.x10, x11: self.x11 - other.x11, x12: self.x12 - other.x12,
			x20: self.x20 - other.x20, x21: self.x21 - other.x21, x22: self.x22 - other.x22
		}
	}
}

impl Mul<f32> for Mat3 {
	type Output = Self;

	fn mul(self, other: f32) -> Self {
		Self {
			x00: self.x00 * other, x01: self.x01 * other, x02: self.x02 * other,
			x10: self.x10 * other, x11: self.x11 * other, x12: self.x12 * other,
			x20: self.x20 * other, x21: self.x21 * other, x22: self.x22 * other
		}
	}
}

impl Mul<Mat3> for f32 {
	type Output = Mat3;

	fn mul(self, other: Mat3) -> Mat3 {
		Mat3 {
			x00: self * other.x00, x01: self * other.x01, x02: self * other.x02,
			x10: self * other.x10, x11: self * other.x11, x12: self * other.x12,
			x20: self * other.x20, x21: self * other.x21, x22: self * other.x22
		}
	}
}

impl Mul<Mat3> for Mat3 {
	type Output = Self;

	fn mul(self, other: Mat3) -> Self {
		Self {
			x00: self.x00 * other.x00 + self.x01 * other.x10 + self.x02 * other.x20,
			x01: self.x00 * other.x01 + self.x01 * other.x11 + self.x02 * other.x21,
			x02: self.x00 * other.x02 + self.x01 * other.x12 + self.x02 * other.x22,
			x10: self.x10 * other.x00 + self.x11 * other.x10 + self.x12 * other.x20,
			x11: self.x10 * other.x01 + self.x11 * other.x11 + self.x12 * other.x21,
			x12: self.x10 * other.x02 + self.x11 * other.x12 + self.x12 * other.x22,
			x20: self.x20 * other.x00 + self.x21 * other.x10 + self.x22 * other.x20,
			x21: self.x20 * other.x01 + self.x21 * other.x11 + self.x22 * other.x21,
			x22: self.x20 * other.x02 + self.x21 * other.x12 + self.x22 * other.x22
		}
	}
}

impl Mul<Vec3> for Mat3 {
	type Output = Vec3;

	fn mul(self, other: Vec3) -> Vec3 {
		Vec3::new(
			self.x00 * other.x() + self.x01 * other.y() + self.x02 * other.z(),
			self.x10 * other.x() + self.x11 * other.y() + self.x12 * other.z(),
			self.x20 * other.x() + self.x21 * other.y() + self.x22 * other.z()
		)
	}
}

impl Div<f32> for Mat3 {
	type Output = Self;

	fn div(self, other: f32) -> Self {
		let inv = 1.0 / other;
		inv * self
	}
}

impl Into<[[f32; 3]; 3]> for Mat3 {
	fn into(self) -> [[f32; 3]; 3] {
		[
			[self.x00, self.x10, self.x20],
			[self.x01, self.x11, self.x21],
			[self.x02, self.x12, self.x22],
		]
	}
}

// ##################################################
// #                   MATRIX 4D                    #
// ##################################################

/// Representation of a 4x4 Matrix
#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Mat4 {
	x00: f32,
	x01: f32,
	x02: f32,
	x03: f32,
	x10: f32,
	x11: f32,
	x12: f32,
	x13: f32,
	x20: f32,
	x21: f32,
	x22: f32,
	x23: f32,
	x30: f32,
	x31: f32,
	x32: f32,
	x33: f32
}

impl Mat4 {
	pub const ZERO: Mat4 = Mat4 {
		x00: 0.0, x01: 0.0, x02: 0.0, x03: 0.0,
		x10: 0.0, x11: 0.0, x12: 0.0, x13: 0.0,
		x20: 0.0, x21: 0.0, x22: 0.0, x23: 0.0,
		x30: 0.0, x31: 0.0, x32: 0.0, x33: 0.0
	};

	pub const IDENTITY: Mat4 = Mat4 {
		x00: 1.0, x01: 0.0, x02: 0.0, x03: 0.0,
		x10: 0.0, x11: 1.0, x12: 0.0, x13: 0.0,
		x20: 0.0, x21: 0.0, x22: 1.0, x23: 0.0,
		x30: 0.0, x31: 0.0, x32: 0.0, x33: 1.0
	};

	pub const OPENGL: Mat4 = Mat4 {
		x00: 1.0, x01: 0.0, x02: 0.0, x03: 0.0,
		x10: 0.0, x11: 1.0, x12: 0.0, x13: 0.0,
		x20: 0.0, x21: 0.0, x22: 0.5, x23: 0.0,
		x30: 0.0, x31: 0.0, x32: 0.5, x33: 1.0
	};

	pub const fn new(x00: f32, x01: f32,x02: f32,x03: f32,x10: f32,x11: f32,x12: f32,x13: f32,x20: f32,x21: f32,x22: f32,x23: f32,x30: f32, x31: f32, x32: f32,x33: f32) -> Self {
		Self {
			x00, x01, x02, x03,
			x10, x11, x12, x13,
			x20, x21, x22, x23,
			x30, x31, x32, x33
		}
	}

	pub fn from_rows(row1: Vec4, row2: Vec4, row3: Vec4, row4: Vec4) -> Self {
		Self {
			x00: row1.x(), x01: row1.y(), x02: row1.z(), x03: row1.w(),
			x10: row2.x(), x11: row2.y(), x12: row2.z(), x13: row2.w(),
			x20: row3.x(), x21: row3.y(), x22: row3.z(), x23: row3.w(),
			x30: row4.x(), x31: row4.y(), x32: row4.z(), x33: row4.w()
		}
	}

	pub fn from_cols(col1: Vec4, col2: Vec4, col3: Vec4, col4: Vec4) -> Self {
		Self {
			x00: col1.x(), x01: col2.x(), x02: col3.x(), x03: col4.x(),
			x10: col1.y(), x11: col2.y(), x12: col3.y(), x13: col4.y(),
			x20: col1.z(), x21: col2.z(), x22: col3.z(), x23: col4.z(),
			x30: col1.w(), x31: col2.w(), x32: col3.w(), x33: col4.w()
		}
	}

	pub fn rh_look_to(camera: Point3, dir: Vec3, up: Vec3) -> Self {
		let f = dir.normalize();
		let s = cross(f, up).normalize();
		let u = cross(s,f);
		let cam = camera.to_vec();


		Mat4::new(
			s.x().clone(), u.x().clone(), -f.x().clone(), 0.0,
			s.y().clone(), u.y().clone(), -f.y().clone(), 0.0,
			s.z().clone(), u.z().clone(), -f.z().clone(), 0.0,
			-dot(&cam, &s), -dot(&cam, &u), dot(&cam, &f),1.0
		)

		/*Mat4::new(
			s.x().clone(), s.y().clone(), s.z().clone(), 0.0,
			u.x().clone(), u.y().clone(), u.z().clone(), 0.0,
			-f.x().clone(), -f.y().clone(), -f.z().clone(), 0.0,
			-dot(&cam, &s), -dot(&cam, &u), dot(&cam, &f), 1.0
		)*/

	}

	pub fn lh_look_to(camera: Point3, dir: Vec3, up: Vec3) -> Self {
		Self::lh_look_to(camera, dir * -1.0, up)
	}

	pub fn look_at_rh(camera: Point3, center: Point3, up: Vec3) -> Mat4 {
		Self::rh_look_to(camera, (center.to_vec() - camera.to_vec()), up)
	}

	pub fn look_at_lh(camera: Point3, center: Point3, up: Vec3) -> Self {
		Self::lh_look_to(camera, (center.to_vec() - camera.to_vec()), up)
	}

	pub fn perspective_matrix(fovy: f32, aspect: f32, near: f32, far: f32) -> Self {
		let angle = fovy * 0.5;
		let ymax = near * angle.tan();
		let xmax = ymax * aspect;

		let left = -xmax;
		let right = xmax;
		let bottom = -ymax;
		let top = ymax;

		Mat4::new(
			(2.0 * near) / (right - left), 0.0, (right + left) / (right - left), 0.0,
			0.0, (2.0 * near) / (top - bottom), (top + bottom) / (top - bottom), 0.0,
			0.0, 0.0, -(far + near) / (far - near), -(2.0 * far * near) / (far - near),
			0.0, 0.0, -1.0, 0.0
		)

		/*Mat4::new(
			(2.0 * near) / (right - left), 0.0, 0.0, 0.0,
			0.0, (2.0 * near) / (top - bottom), 0.0, 0.0,
			(right + left) / (right - left), (top + bottom) / (top - bottom), -(far + near) / (far - near), -1.0,
			0.0, 0.0, -(2.0 * far * near) / (far - near), 0.0
		)*/

	}

	pub fn orthographic_matrix(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
		Mat4::new(
			2.0 / (right - left), 0.0, 0.0, 0.0,
			0.0, 2.0 / (top - bottom), 0.0, 0.0,
			0.0, 0.0, -2.0 / (far - near), 0.0,
			-(right + left) / (right - left), -(top + bottom) / (top - bottom), -(far + near) / (far - near), 1.0
		)
	}

	pub fn get(&self, row: usize, col: usize) -> Option<f32> {
		assert!(row <= 3, "This row ({}) is out of bounds! Bounds: 0..3", row);
		assert!(col <= 3, "This row ({}) is out of bounds! Bounds: 0..3", col);
		match (row, col) {
			(0, 0) => Some(self.x00),
			(0, 1) => Some(self.x01),
			(0, 2) => Some(self.x02),
			(0, 3) => Some(self.x03),
			(1, 0) => Some(self.x10),
			(1, 1) => Some(self.x11),
			(1, 2) => Some(self.x12),
			(1, 3) => Some(self.x13),
			(2, 0) => Some(self.x20),
			(2, 1) => Some(self.x21),
			(2, 2) => Some(self.x22),
			(2, 3) => Some(self.x23),
			(3, 0) => Some(self.x30),
			(3, 1) => Some(self.x31),
			(3, 2) => Some(self.x32),
			(3, 3) => Some(self.x33),
			_ => None,
		}
	}

	pub fn get_row(&self, row: usize) -> Option<Vec4> {
		assert!(row <= 3, "This row ({}) is out of bounds! Bounds: 0..3", row);
		match row {
			0 => Some(Vec4::new(self.x00, self.x01, self.x02, self.x03)),
			1 => Some(Vec4::new(self.x10, self.x11, self.x12, self.x13)),
			2 => Some(Vec4::new(self.x20, self.x21, self.x22, self.x23)),
			3 => Some(Vec4::new(self.x30, self.x31, self.x32, self.x33)),
			_ => None
		}
	}

	pub fn get_col(&self, col: usize) -> Option<Vec4> {
		assert!(col <= 3, "This row ({}) is out of bounds! Bounds: 0..3", col);
		match col {
			0 => Some(Vec4::new(self.x00, self.x10, self.x20, self.x30)),
			1 => Some(Vec4::new(self.x01, self.x11, self.x21, self.x31)),
			2 => Some(Vec4::new(self.x02, self.x12, self.x22, self.x32)),
			3 => Some(Vec4::new(self.x03, self.x13, self.x23, self.x33)),
			_ => None
		}
	}

	pub fn set(&mut self, row: usize, col: usize, element: f32) {
		assert!(row <= 3, "The given row ({}) is out of bounds! Bounds: 0..3", row);
		assert!(col <= 3, "The given column ({}) is out of bounds! Bounds: 0..3", col);
		match (row, col) {
			(0,0) => self.x00 = element,
			(0,1) => self.x01 = element,
			(0,2) => self.x02 = element,
			(0,3) => self.x03 = element,
			(1,0) => self.x10 = element,
			(1,1) => self.x11 = element,
			(1,2) => self.x12 = element,
			(1,3) => self.x13 = element,
			(2,0) => self.x20 = element,
			(2,1) => self.x21 = element,
			(2,2) => self.x22 = element,
			(2,3) => self.x23 = element,
			(3,0) => self.x30 = element,
			(3,1) => self.x31 = element,
			(3,2) => self.x32 = element,
			(3,3) => self.x33 = element,
			_ => {}
		}
	}

	pub fn set_row(&mut self, row: usize, row_content: Vec4) {
		assert!(row <= 3, "This row ({}) is out of bounds: Bounds: 0..3", row);
		match row {
			0 => { self.x00 = row_content.x(); self.x01 = row_content.y(); self.x02 = row_content.z(); self.x03 = row_content.w(); },
			1 => { self.x10 = row_content.x(); self.x11 = row_content.y(); self.x12 = row_content.z(); self.x13 = row_content.w(); },
			2 => { self.x20 = row_content.x(); self.x21 = row_content.y(); self.x22 = row_content.z(); self.x23 = row_content.w(); },
			3 => { self.x30 = row_content.x(); self.x31 = row_content.y(); self.x32 = row_content.z(); self.x33 = row_content.w(); }
			_ => {}
		}
	}

	pub fn set_col(&mut self, col: usize, col_content: Vec4) {
		assert!(col <= 3, "This column ({}) is out of bounds! Bounds: 0..3", col);
		match col {
			0 => { self.x00 = col_content.x(); self.x10 = col_content.y(); self.x20 = col_content.z(); self.x30 = col_content.w(); },
			1 => { self.x01 = col_content.x(); self.x11 = col_content.y(); self.x21 = col_content.z(); self.x31 = col_content.w(); },
			2 => { self.x02 = col_content.x(); self.x12 = col_content.y(); self.x22 = col_content.z(); self.x32 = col_content.w(); },
			3 => { self.x03 = col_content.x(); self.x13 = col_content.y(); self.x23 = col_content.z(); self.x33 = col_content.w(); }
			_ => {}
		}
	}

	pub fn det(&self) -> f32 {
		self.x00 * (self.x11 * (self.x22* self.x33 - self.x23 * self.x32)
			- self.x21 * (self.x12 * self.x33 - self.x13 * self.x32)
			+ self.x31 * (self.x12 * self.x23 - self.x13 * self.x22))
			- self.x10 * (self.x01 * (self.x22* self.x33 - self.x23 * self.x32)
			- self.x21 * (self.x02 * self.x33 - self.x32 * self.x03)
			+ self.x31 * (self.x02 * self.x23 - self.x22 * self.x03))
			+ self.x20 * (self.x01 * (self.x12 * self.x33 - self.x13 * self.x32)
			- self.x11 * (self.x02 * self.x33 - self.x03 * self.x32)
			+ self.x31 * (self.x02 * self.x13 - self.x03 * self.x12))
			- self.x30 * (self.x01 * (self.x12 * self.x23 - self.x22 * self.x13)
			- self.x11 * (self.x02 * self.x23 - self.x22 * self.x03)
			+ self.x21 * (self.x02 * self.x13 - self.x03 * self.x12))
	}

	pub fn transpose(&self) -> Self {
		Self {
			x00: self.x00, x01: self.x10, x02: self.x20, x03: self.x30,
			x10: self.x01, x11: self.x11, x12: self.x21, x13: self.x31,
			x20: self.x02, x21: self.x12, x22: self.x22, x23: self.x32,
			x30: self.x03, x31: self.x13, x32: self.x23, x33: self.x33
		}
	}

	pub fn swap_rows(&mut self, row1: usize, row2: usize) {
		let tmp = self.get_row(row1).expect(format!("This row ({}) is out of bounds! Bounds: 0..2", row1).as_str());
		self.set_row(row1, self.get_row(row2).expect(format!("This row ({}) is out of bounds! Bounds: 0..2", row2).as_str()));
		self.set_row(row2, tmp);
	}

	pub fn swap_cols(&mut self, col1: usize, col2: usize) {
		let tmp = self.get_col(col1).expect(format!("This row ({}) is out of bounds! Bounds: 0..2", col1).as_str());
		self.set_col(col1, self.get_col(col2).expect(format!("This row ({}) is out of bounds! Bounds: 0..2", col2).as_str()));
		self.set_col(col2, tmp);
	}
}

impl Add<Mat4> for Mat4 {
	type Output = Self;

	fn add(self, other: Mat4) -> Self {
		Self {
			x00: self.x00 + other.x00, x01: self.x01 + other.x01, x02: self.x02 + other.x02, x03: self.x03 + other.x03,
			x10: self.x10 + other.x10, x11: self.x11 + other.x11, x12: self.x12 + other.x12, x13: self.x13 + other.x13,
			x20: self.x20 + other.x20, x21: self.x21 + other.x21, x22: self.x22 + other.x22, x23: self.x23 + other.x23,
			x30: self.x30 + other.x30, x31: self.x31 + other.x31, x32: self.x32 + other.x32, x33: self.x33 + other.x33
		}
	}
}

impl Sub<Mat4> for Mat4 {
	type Output = Self;

	fn sub(self, other: Mat4) -> Self {
		Self {
			x00: self.x00 - other.x00, x01: self.x01 - other.x01, x02: self.x02 - other.x02, x03: self.x03 - other.x03,
			x10: self.x10 - other.x10, x11: self.x11 - other.x11, x12: self.x12 - other.x12, x13: self.x13 - other.x13,
			x20: self.x20 - other.x20, x21: self.x21 - other.x21, x22: self.x22 - other.x22, x23: self.x23 - other.x23,
			x30: self.x30 - other.x30, x31: self.x31 - other.x31, x32: self.x32 - other.x32, x33: self.x33 - other.x33
		}
	}
}

impl Mul<f32> for Mat4 {
	type Output = Self;

	fn mul(self, other: f32) -> Self {
		Self {
			x00: self.x00 * other, x01: self.x01 * other, x02: self.x02 * other, x03: self.x03 * other,
			x10: self.x10 * other, x11: self.x11 * other, x12: self.x12 * other, x13: self.x13 * other,
			x20: self.x20 * other, x21: self.x21 * other, x22: self.x22 * other, x23: self.x23 * other,
			x30: self.x30 * other, x31: self.x31 * other, x32: self.x32 * other, x33: self.x33 * other
		}
	}
}

impl Div<f32> for Mat4 {
	type Output = Self;

	fn div(self, other: f32) -> Self {
		Self {
			x00: self.x00 / other, x01: self.x01 / other, x02: self.x02 / other, x03: self.x03 / other,
			x10: self.x10 / other, x11: self.x11 / other, x12: self.x12 / other, x13: self.x13 / other,
			x20: self.x20 / other, x21: self.x21 / other, x22: self.x22 / other, x23: self.x23 / other,
			x30: self.x30 / other, x31: self.x31 / other, x32: self.x32 / other, x33: self.x33 / other
		}
	}
}

impl Mul<Mat4> for Mat4 {
	type Output = Self;

	fn mul(self, other: Mat4) -> Self {
		Self {
			x00: self.x00 * other.x00 + self.x01 * other.x10 + self.x02 * other.x20 + self.x03 * other.x30,
			x01: self.x00 * other.x01 + self.x01 * other.x11 + self.x02 * other.x21 + self.x03 * other.x31,
			x02: self.x00 * other.x02 + self.x01 * other.x12 + self.x02 * other.x22 + self.x03 * other.x32,
			x03: self.x00 * other.x03 + self.x01 * other.x13 + self.x02 * other.x23 + self.x03 * other.x33,
			x10: self.x10 * other.x00 + self.x11 * other.x10 + self.x12 * other.x20 + self.x13 * other.x30,
			x11: self.x10 * other.x01 + self.x11 * other.x11 + self.x12 * other.x21 + self.x13 * other.x31,
			x12: self.x10 * other.x02 + self.x11 * other.x12 + self.x12 * other.x22 + self.x13 * other.x32,
			x13: self.x10 * other.x03 + self.x11 * other.x13 + self.x12 * other.x23 + self.x13 * other.x33,
			x20: self.x20 * other.x00 + self.x21 * other.x10 + self.x22 * other.x20 + self.x23 * other.x30,
			x21: self.x20 * other.x01 + self.x21 * other.x11 + self.x22 * other.x21 + self.x23 * other.x31,
			x22: self.x20 * other.x02 + self.x21 * other.x12 + self.x22 * other.x22 + self.x23 * other.x32,
			x23: self.x20 * other.x03 + self.x21 * other.x13 + self.x22 * other.x23 + self.x23 * other.x33,
			x30: self.x30 * other.x00 + self.x31 * other.x10 + self.x32 * other.x20 + self.x33 * other.x30,
			x31: self.x30 * other.x01 + self.x31 * other.x11 + self.x32 * other.x21 + self.x33 * other.x31,
			x32: self.x30 * other.x02 + self.x31 * other.x12 + self.x32 * other.x22 + self.x33 * other.x32,
			x33: self.x30 * other.x03 + self.x31 * other.x13 + self.x32 * other.x23 + self.x33 * other.x33
		}
	}
}

impl Into<[[f32; 4]; 4]> for Mat4 {
	fn into(self) -> [[f32; 4]; 4] {
		[
			[self.x00, self.x10, self.x20, self.x30],
			[self.x01, self.x11, self.x21, self.x31],
			[self.x02, self.x12, self.x22, self.x32],
			[self.x03, self.x13, self.x23, self.x33],
		]
	}
}

// ##################################################
// #              MATRIX FUNCTIONS                  #
// ##################################################

pub fn det<T: LinearTransformation>(mat: &T) -> f32 {
	mat.det()
}
