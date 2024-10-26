use crate::point::{Point2, Point3};
use crate::vector::{Vec2, Vec3, Vec4, InnerSpace};

// ##################################################
// #                   CONSTANTS                    #
// ##################################################

static FAC: [i64; 21] = [
	1,1,2,6,24,120,720,5040,40320,362880,3628800,
	39916800,479001600,6227020800,87178291200,1307674368000,
	20922789888000,355687428096000,6402373705728000,
	121645100408832000,2432902008176640000
];

static iFAC: [f32; 6] = [
	1.0,1.0,0.5,0.1666666666666666667,0.04166666666666666667,0.00833333333333333334
];

pub static PI: f32 = std::f32::consts::PI;

// ##################################################
// #                GENERAL PURPOSE                 #
// ##################################################

pub fn fac(n: i64) -> i64 {
	match n {
		_ if n <= 21 => { FAC[n as usize] }
		_ => n * fac(n-1)
	}
}

pub fn sqrt(x: f32) -> f32 {
	x.sqrt()
}

pub fn ln(x: f32) -> f32 {
	x.ln()
}

pub fn log(x: f32) -> f32 {
	ln(x)/2.30258509299
}

pub fn log2(x: f32) -> f32 {
	ln(x)/0.69314718056
}

pub fn sin(x: f32) -> f32 {
	x.sin()
}

pub fn asin(x: f32) -> f32 {
	x.asin()
}

pub fn cos(x: f32) -> f32 {
	x.cos()
}

pub fn acos(x: f32) -> f32 {
	x.acos()
}

pub fn tan(x: f32) -> f32 {
	x.tan()
}

pub fn atan(x: f32) -> f32 {
	x.atan()
}

pub fn atan2(p: Point2) -> f32 {
	p.y().atan2(p.x())
}

pub fn sinh(x: f32) -> f32 {
	x.sinh()
}

pub fn cosh(x: f32) -> f32 {
	x.cosh()
}

pub fn tanh(x: f32) -> f32 {
	x.tanh()
}

pub fn clamp(start: f32, end: f32, value: f32) -> f32 {
	match value {
		_ if value > end => end,
		_ if start > value => start,
		_ => value
	}
}

pub fn pointDerivative(func: fn(f32) -> f32, x: f32, h: f32) -> f32 {
	(func(x+h) - func(x-h))/(2.0 * h)
}

// ##################################################
// #                 INTERPOLATION                  #
// ##################################################

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
	(1.0 - t) * a + t * b
}

pub fn invLerp(a: f32, b:f32, value: f32) -> f32 {
	(value - a) / (b - a)
}

pub fn lerp2(a: Vec2, b: Vec2, t: f32) -> Vec2 {
	a * (1.0 - t) + b * t
}

pub fn invLerp2(a: Vec2, b: Vec2, value: Vec2) -> Option<f32> {
	let tx = (value.x() - a.x()) / (b.x() - a.x());
	let ty = (value.y() - a.y()) / (b.y() - a.y());

	if tx == ty {
		return Some(tx);
	}
	None
}

pub fn lerp3(a: Vec3, b: Vec3, t: f32) -> Vec3 {
	a * (1.0 - t) + b * t
}

pub fn invLerp3(a: Vec3, b: Vec3, value: Vec3) -> Option<f32> {
	let tx = (value.x() - a.x())/(b.x() - a.x());
	let ty = (value.y() - a.y())/(b.y() - a.y());
	let tz = (value.z() - a.z())/(b.z() - a.z());

	if (tx == ty) && (ty == tz) {
		return Some(tx);
	}
	None
}

// ##################################################
// #                  BEZIER CURVES                 #
// ##################################################

/// Cubic Bézier Curve in R²
pub fn bezier2(p0: Point2, p1: Point2, p2: Point2, p3: Point2, t: f32) -> Point2 {
	let tSquared = t * t;
	let tCubed = tSquared * t;
	let vP0 = Vec2::from_point(p0);
	let vP1 = Vec2::from_point(p1);
	let vP2 = Vec2::from_point(p2);
	let vP3 = Vec2::from_point(p3);

	Point2::from_vec(vP0 * (-tCubed + 3.0 * tSquared - 3.0 * t + 1.0 ) +
		vP1 * (3.0 * tCubed - 6.0 * tSquared + 3.0 * t ) +
		vP2 * (-3.0 * tCubed + 3.0 * tSquared ) +
		vP3 * tCubed)
}

/// Cubic Bézier Curve in R³
pub fn bezier3(p0: Point3, p1: Point3, p2: Point3, p3: Point3, t: f32) -> Point3 {
	let tSquared = t * t;
	let tCubed = tSquared * t;
	let vP0 = Vec3::from_point(p0);
	let vP1 = Vec3::from_point(p1);
	let vP2 = Vec3::from_point(p2);
	let vP3 = Vec3::from_point(p3);

	Point3::from_vec(vP0 * (-tCubed + 3.0 * tSquared - 3.0 * t + 1.0 ) +
		vP1 * (3.0 * tCubed - 6.0 * tSquared + 3.0 * t ) +
		vP2 * (-3.0 * tCubed + 3.0 * tSquared ) +
		vP3 * tCubed)
}

// ##################################################
// #                    SPLINES                     #
// ##################################################
