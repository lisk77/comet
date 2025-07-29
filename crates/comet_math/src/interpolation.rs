use std::f32::consts::PI;

#[inline(always)]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

#[inline(always)]
pub fn inverse_lerp(a: f32, b: f32, x: f32) -> f32 {
    (x - a) / (b - a)
}

#[inline(always)]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[inline(always)]
pub fn inverse_smoothstep(v: f32) -> f32 {
    (0.5 - (0.5 - v.sqrt()).sqrt()).clamp(0.0, 1.0)
}

#[inline(always)]
pub fn smootherstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline(always)]
pub fn cosine_interpolate(a: f32, b: f32, x: f32) -> f32 {
    let ft = x * PI;
    let f = (1.0 - ft.cos()) * 0.5;
    a * (1.0 - f) + b * f
}

#[inline(always)]
pub fn inverse_cosine_interpolate(a: f32, b: f32, v: f32) -> f32 {
    ((1.0 - 2.0 * (v - a) / (b - a)).acos()) / PI
}

#[inline(always)]
pub fn cubic_interpolate(a: f32, b: f32, c: f32, d: f32, t: f32) -> f32 {
    let p = (d - c) - (a - b);
    let q = (a - b) - p;
    let r = c - a;
    let s = b;
    p * t.powi(3) + q * t.powi(2) + r * t + s
}

#[inline(always)]
pub fn hermite_interpolate(p0: f32, p1: f32, m0: f32, m1: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    (2.0 * t3 - 3.0 * t2 + 1.0) * p0
        + (t3 - 2.0 * t2 + t) * m0
        + (-2.0 * t3 + 3.0 * t2) * p1
        + (t3 - t2) * m1
}

#[inline(always)]
pub fn catmull_rom_interpolate(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t * t
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t * t * t)
}
