use num_traits::Pow;

use crate::point::{Point2, Point3};
use crate::vector::{Vec2, Vec3};
use crate::utilities::{lerp2, lerp3, fac};

pub trait ParameterCurve2 {
	//fn arcLen(&self) -> f32;
	fn getPoint(&self, t: f32) -> Point2;
}

pub trait ParameterCurve3 {
	//fn arcLen(&self) -> f32;
	fn getPoint(&self, t: f32) -> Point3;
}

/// A general Bézier Curve in 2D
/// WORK IN PROGRESS: DOES NOT WORK (use cBezier2 instead)
#[repr(C)]
pub struct Bezier2 {
	points: Vec<Point2>,
	degree: u8
}

impl Bezier2 {
	pub fn new(points: Vec<Point2>) -> Self {
		let n = points.len() as u8;
		Self { points: points, degree: n }
	}
}


/// A general Bézier Curve in 3D
/// WORK IN PROGRESS: DOES NOT WORK (use cBezier3 instead)
#[repr(C)]
pub struct Bezier3 {
	points: Vec<Point3>,
	degree: u8
}

impl Bezier3 {
	pub fn new(points: Vec<Point3>) -> Self {
		let n = points.len() as u8;
		Self { points: points, degree: n }
	}
}

/// A cubic Bézier Curve in 2D
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct cBezier2 {
	p0: Point2,
	p1: Point2,
	p2: Point2,
	p3: Point2
}

impl cBezier2 {
	pub fn new(p0: Point2, p1: Point2, p2: Point2, p3: Point2) -> Self {
		Self { p0, p1, p2, p3 }
	}
}

impl ParameterCurve2 for cBezier2 {
	fn getPoint(&self, t: f32) -> Point2 {
		let tSquared = t * t;
		let tCubed = tSquared * t;
		let vP0 = Vec2::from_point(self.p0);
		let vP1 = Vec2::from_point(self.p1);
		let vP2 = Vec2::from_point(self.p2);
		let vP3 = Vec2::from_point(self.p3);

		Point2::from_vec(
			vP0 * (-tCubed + 3.0 * tSquared - 3.0 * t + 1.0 ) +
				vP1 * (3.0 * tCubed - 6.0 * tSquared + 3.0 * t ) +
				vP2 * (-3.0 * tCubed + 3.0 * tSquared ) +
				vP3 * tCubed
		)
	}
}

/// A cubic Bézier Curve in 3D
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct cBezier3 {
	p0: Point3,
	p1: Point3,
	p2: Point3,
	p3: Point3
}

impl cBezier3 {
	pub fn new(p0: Point3, p1: Point3, p2: Point3, p3: Point3) -> Self {
		Self { p0, p1, p2, p3 }
	}
}

impl ParameterCurve2 for Bezier2 {
	fn getPoint(&self, t: f32) -> Point2 {
		let n = self.points.len();
		let mut points = self.points.clone();

		for k in 1..n {
			for i in 0..n - k {
				points[i] = Point2::from_vec(lerp2(Vec2::from_point(points[i]), Vec2::from_point(points[i + 1]), t));
			}
		}

		points[0]
	}
}

impl ParameterCurve3 for Bezier3 {
	fn getPoint(&self, t: f32) -> Point3 {
		let n = self.points.len();
		let mut points = self.points.clone();

		for k in 1..n {
			for i in 0..n - k {
				points[i] = Point3::from_vec(lerp3(Vec3::from_point(points[i]), Vec3::from_point(points[i + 1]), t));
			}
		}

		points[0]
	}
}

impl ParameterCurve3 for cBezier3 {
	fn getPoint(&self, t: f32) -> Point3 {
		let tSquared = t * t;
		let tCubed = tSquared * t;
		let vP0 = Vec3::from_point(self.p0);
		let vP1 = Vec3::from_point(self.p1);
		let vP2 = Vec3::from_point(self.p2);
		let vP3 = Vec3::from_point(self.p3);

		Point3::from_vec(
			vP0 * (-tCubed + 3.0 * tSquared - 3.0 * t + 1.0 ) +
				vP1 * (3.0 * tCubed - 6.0 * tSquared + 3.0 * t ) +
				vP2 * (-3.0 * tCubed + 3.0 * tSquared ) +
				vP3 * tCubed
		)
	}

}
