use std::f32::consts::PI;

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
	a * (1.0 - t) + b * t
}

pub fn inverse_lerp(a: f32, b: f32, x: f32) -> f32 {
	(x - a) / (b - a)
}

pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
	let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
	t * t * (3.0 - 2.0 * t)
}

pub fn inverse_smoothstep(v: f32) -> f32 {
	(0.5 - (0.5 - v.sqrt()).sqrt()).clamp(0.0, 1.0)
}

pub fn smootherstep(edge0: f32, edge1: f32, x: f32) -> f32 {
	let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
	t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

pub fn cosine_interpolate(a: f32, b: f32, x: f32) -> f32 {
	let ft = x * PI;
	let f = (1.0 - ft.cos()) * 0.5;
	a * (1.0 - f) + b * f
}

pub fn inverse_cosine_interpolate(a: f32, b: f32, v: f32) -> f32 {
	((1.0 - 2.0 * (v - a) / (b - a)).acos()) / PI
}