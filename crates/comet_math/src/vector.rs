use crate::point::{p2, p3};
use crate::quaternion::Quat;
use crate::Point;
use std::ops::*;

pub trait InnerSpace:
    std::fmt::Debug
    + Copy
    + Clone
    + Neg<Output = Self>
    + Mul<f32, Output = Self>
    + Div<f32, Output = Self>
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
{
    fn dot(&self, other: &Self) -> f32;
    fn dist(&self, other: &Self) -> f32;
    fn angle(&self, other: &Self) -> f32;
    fn length(&self) -> f32;
    fn normalize(&self) -> Self;
    fn normalize_mut(&mut self);
    fn project_onto(&self, other: &Self) -> Self;
    fn reflect(&self, normal: &Self) -> Self;
    fn lerp(&self, other: &Self, t: f32) -> Self;
    fn to_point(&self) -> impl Point;
}

// ##################################################
// #                   VECTOR 2D                    #
// ##################################################

/// Representation of a 2D Vector
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[allow(non_camel_case_types)]
pub struct v2 {
    x: f32,
    y: f32,
}

impl v2 {
    pub const X: v2 = v2 { x: 1.0, y: 0.0 };
    pub const Y: v2 = v2 { x: 0.0, y: 1.0 };
    pub const ZERO: v2 = v2 { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        v2 { x, y }
    }

    pub fn from_point(p: p2) -> Self {
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
}

impl Add<v2> for v2 {
    type Output = v2;

    fn add(self, other: v2) -> v2 {
        v2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for v2 {
    fn add_assign(&mut self, other: v2) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Sub<v2> for v2 {
    type Output = v2;

    fn sub(self, other: v2) -> v2 {
        v2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl SubAssign for v2 {
    fn sub_assign(&mut self, other: v2) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl Neg for v2 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Mul<f32> for v2 {
    type Output = v2;

    fn mul(self, other: f32) -> v2 {
        v2 {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Mul<v2> for f32 {
    type Output = v2;

    fn mul(self, other: v2) -> v2 {
        v2 {
            x: self * other.x,
            y: self * other.y,
        }
    }
}

impl Div<f32> for v2 {
    type Output = v2;

    fn div(self, other: f32) -> v2 {
        v2 {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl Into<[f32; 2]> for v2 {
    fn into(self) -> [f32; 2] {
        [self.x, self.y]
    }
}

impl Into<v2> for [f32; 2] {
    fn into(self) -> v2 {
        v2 {
            x: self[0],
            y: self[1],
        }
    }
}

/// Representation of a 2D integer Vector
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[allow(non_camel_case_types)]
pub struct v2i {
    x: i64,
    y: i64,
}

impl v2i {
    pub const X: v2i = v2i { x: 1, y: 0 };
    pub const Y: v2i = v2i { x: 0, y: 1 };
    pub const ZERO: v2i = v2i { x: 0, y: 0 };

    pub const fn new(x: i64, y: i64) -> Self {
        v2i { x, y }
    }

    pub fn from_point(p: p2) -> Self {
        Self {
            x: p.x() as i64,
            y: p.y() as i64,
        }
    }

    pub fn as_point(&self) -> p2 {
        p2::new(self.x as f32, self.y as f32)
    }

    pub fn from_vec2(v: v2) -> Self {
        Self {
            x: v.x as i64,
            y: v.y as i64,
        }
    }

    pub fn as_vec2(&self) -> v2 {
        v2 {
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
        v2i {
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

impl Add<v2i> for v2i {
    type Output = v2i;

    fn add(self, other: v2i) -> v2i {
        v2i {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Add<v2i> for v2 {
    type Output = v2;

    fn add(self, other: v2i) -> v2 {
        v2 {
            x: self.x + other.x as f32,
            y: self.y + other.y as f32,
        }
    }
}

impl Add<v2> for v2i {
    type Output = v2;

    fn add(self, other: v2) -> v2 {
        v2 {
            x: self.x as f32 + other.x,
            y: self.y as f32 + other.y,
        }
    }
}

impl AddAssign for v2i {
    fn add_assign(&mut self, other: v2i) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Sub<v2i> for v2i {
    type Output = v2i;

    fn sub(self, other: v2i) -> v2i {
        v2i {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Sub<v2i> for v2 {
    type Output = v2;

    fn sub(self, other: v2i) -> v2 {
        v2 {
            x: self.x - other.x as f32,
            y: self.y - other.y as f32,
        }
    }
}

impl Sub<v2> for v2i {
    type Output = v2;

    fn sub(self, other: v2) -> v2 {
        v2 {
            x: self.x as f32 - other.x,
            y: self.y as f32 - other.y,
        }
    }
}

impl SubAssign for v2i {
    fn sub_assign(&mut self, other: v2i) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl Mul<f32> for v2i {
    type Output = v2i;

    fn mul(self, other: f32) -> v2i {
        v2i {
            x: self.x * other as i64,
            y: self.y * other as i64,
        }
    }
}

impl From<v2i> for v2 {
    fn from(v: v2i) -> v2 {
        v2 {
            x: v.x as f32,
            y: v.y as f32,
        }
    }
}

impl From<v2> for v2i {
    fn from(v: v2) -> v2i {
        v2i {
            x: v.x as i64,
            y: v.y as i64,
        }
    }
}

impl Into<[i64; 2]> for v2i {
    fn into(self) -> [i64; 2] {
        [self.x, self.y]
    }
}

impl Into<[f32; 2]> for v2i {
    fn into(self) -> [f32; 2] {
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
#[allow(non_camel_case_types)]
pub struct v3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl v3 {
    pub const X: v3 = v3 {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };
    pub const Y: v3 = v3 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    pub const Z: v3 = v3 {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };
    pub const ZERO: v3 = v3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        v3 { x, y, z }
    }

    pub fn from_point(p: p3) -> Self {
        Self {
            x: p.x(),
            y: p.y(),
            z: p.z(),
        }
    }

    pub fn as_point(&self) -> p3 {
        p3::new(self.x as f32, self.y as f32, self.z as f32)
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
}

impl Add<v3> for v3 {
    type Output = v3;

    fn add(self, other: v3) -> v3 {
        v3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl AddAssign for v3 {
    fn add_assign(&mut self, other: v3) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl Sub<v3> for v3 {
    type Output = v3;

    fn sub(self, other: v3) -> v3 {
        v3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl SubAssign for v3 {
    fn sub_assign(&mut self, other: v3) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl Neg for v3 {
    type Output = v3;

    fn neg(self) -> v3 {
        v3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Mul<f32> for v3 {
    type Output = v3;

    fn mul(self, other: f32) -> v3 {
        v3 {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}

impl Mul<v3> for f32 {
    type Output = v3;

    fn mul(self, other: v3) -> v3 {
        v3 {
            x: self * other.x,
            y: self * other.y,
            z: self * other.z,
        }
    }
}

impl Div<f32> for v3 {
    type Output = v3;

    fn div(self, other: f32) -> v3 {
        v3 {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
        }
    }
}

impl Into<Quat> for v3 {
    fn into(self) -> Quat {
        Quat::new(0.0, self)
    }
}

impl Into<[f32; 3]> for v3 {
    fn into(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}

impl Into<v3> for [f32; 3] {
    fn into(self) -> v3 {
        v3 {
            x: self[0],
            y: self[1],
            z: self[2],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[allow(non_camel_case_types)]
pub struct v3i {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl v3i {
    pub const X: v3i = v3i { x: 1, y: 0, z: 0 };
    pub const Y: v3i = v3i { x: 0, y: 1, z: 0 };
    pub const Z: v3i = v3i { x: 0, y: 0, z: 1 };
    pub const ZERO: v3i = v3i { x: 0, y: 0, z: 0 };

    pub const fn new(x: i64, y: i64, z: i64) -> Self {
        v3i { x, y, z }
    }

    pub fn from_point(p: p3) -> Self {
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
        v3i {
            x: factor * self.x,
            y: factor * self.y,
            z: factor * self.z,
        }
    }
}

impl Add<v3i> for v3i {
    type Output = v3i;

    fn add(self, other: v3i) -> v3i {
        v3i {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Add<v3i> for v3 {
    type Output = v3;

    fn add(self, other: v3i) -> v3 {
        v3 {
            x: self.x + other.x as f32,
            y: self.y + other.y as f32,
            z: self.z + other.z as f32,
        }
    }
}

impl Add<v3> for v3i {
    type Output = v3;

    fn add(self, other: v3) -> v3 {
        v3 {
            x: self.x as f32 + other.x,
            y: self.y as f32 + other.y,
            z: self.z as f32 + other.z,
        }
    }
}

impl AddAssign for v3i {
    fn add_assign(&mut self, other: v3i) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl Sub<v3i> for v3i {
    type Output = v3i;

    fn sub(self, other: v3i) -> v3i {
        v3i {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Sub<v3i> for v3 {
    type Output = v3;

    fn sub(self, other: v3i) -> v3 {
        v3 {
            x: self.x - other.x as f32,
            y: self.y - other.y as f32,
            z: self.z - other.z as f32,
        }
    }
}

impl Sub<v3> for v3i {
    type Output = v3;

    fn sub(self, other: v3) -> v3 {
        v3 {
            x: self.x as f32 - other.x,
            y: self.y as f32 - other.y,
            z: self.z as f32 - other.z,
        }
    }
}

impl SubAssign for v3i {
    fn sub_assign(&mut self, other: v3i) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl Mul<f32> for v3i {
    type Output = v3i;

    fn mul(self, other: f32) -> v3i {
        v3i {
            x: self.x * other as i64,
            y: self.y * other as i64,
            z: self.z * other as i64,
        }
    }
}

impl From<v3i> for v3 {
    fn from(v: v3i) -> v3 {
        v3 {
            x: v.x as f32,
            y: v.y as f32,
            z: v.z as f32,
        }
    }
}

impl From<v3> for v3i {
    fn from(v: v3) -> v3i {
        v3i {
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
#[allow(non_camel_case_types)]
pub struct v4 {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl v4 {
    pub const X: v4 = v4 {
        x: 1.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };
    pub const Y: v4 = v4 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
        w: 0.0,
    };
    pub const Z: v4 = v4 {
        x: 0.0,
        y: 0.0,
        z: 1.0,
        w: 0.0,
    };
    pub const W: v4 = v4 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    pub const ZERO: v4 = v4 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };

    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        v4 { x, y, z, w }
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

impl Add<v4> for v4 {
    type Output = v4;

    fn add(self, other: v4) -> v4 {
        v4 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl AddAssign for v4 {
    fn add_assign(&mut self, other: v4) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
        self.w += other.w;
    }
}

impl Sub<v4> for v4 {
    type Output = v4;

    fn sub(self, other: v4) -> v4 {
        v4 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

impl SubAssign for v4 {
    fn sub_assign(&mut self, other: v4) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
        self.w -= other.w;
    }
}

impl Neg for v4 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}

impl Mul<f32> for v4 {
    type Output = v4;

    fn mul(self, other: f32) -> v4 {
        v4 {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
            w: self.w * other,
        }
    }
}

impl Mul<v4> for f32 {
    type Output = v4;

    fn mul(self, other: v4) -> v4 {
        v4 {
            x: self * other.x,
            y: self * other.y,
            z: self * other.z,
            w: self * other.w,
        }
    }
}

impl MulAssign<f32> for v4 {
    fn mul_assign(&mut self, other: f32) {
        self.x *= other;
        self.y *= other;
        self.z *= other;
        self.w *= other;
    }
}

impl Div<f32> for v4 {
    type Output = v4;

    fn div(self, other: f32) -> v4 {
        v4 {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
            w: self.w / other,
        }
    }
}

impl Into<[f32; 4]> for v4 {
    fn into(self) -> [f32; 4] {
        [self.x, self.y, self.z, self.w]
    }
}

impl InnerSpace for v2 {
    fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    fn dist(&self, other: &Self) -> f32 {
        v2 {
            x: other.x - self.x,
            y: other.y - self.y,
        }
        .length()
    }

    fn angle(&self, other: &Self) -> f32 {
        (self.dot(other) / (self.length() * other.length())).acos()
    }

    fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn normalize(&self) -> Self {
        let factor = 1.0 / self.length();
        v2 {
            x: factor * self.x,
            y: factor * self.y,
        }
    }

    fn normalize_mut(&mut self) {
        let factor = 1.0 / self.length();
        self.x *= factor;
        self.y *= factor;
    }

    fn project_onto(&self, other: &Self) -> Self {
        let factor = self.dot(other) / other.dot(other);
        *other * factor
    }

    fn reflect(&self, normal: &Self) -> Self {
        *self - *normal * 2.0 * self.dot(normal)
    }

    fn lerp(&self, other: &Self, t: f32) -> Self {
        *self * (1.0 - t) + *other * t
    }

    fn to_point(&self) -> impl Point {
        p2::new(self.x, self.y)
    }
}

impl InnerSpace for v3 {
    fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn dist(&self, other: &Self) -> f32 {
        v3 {
            x: other.x - self.x,
            y: other.y - self.y,
            z: other.z - self.z,
        }
        .length()
    }

    fn angle(&self, other: &Self) -> f32 {
        (self.dot(other) / (self.length() * other.length())).acos()
    }

    fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    fn normalize(&self) -> Self {
        let factor = 1.0 / self.length();
        v3 {
            x: factor * self.x,
            y: factor * self.y,
            z: factor * self.z,
        }
    }

    fn normalize_mut(&mut self) {
        let factor = 1.0 / self.length();
        self.x *= factor;
        self.y *= factor;
        self.z *= factor;
    }

    fn project_onto(&self, other: &Self) -> Self {
        let factor = self.dot(other) / other.dot(other);
        *other * factor
    }

    fn reflect(&self, normal: &Self) -> Self {
        *self - *normal * 2.0 * self.dot(normal)
    }

    fn lerp(&self, other: &Self, t: f32) -> Self {
        *self * (1.0 - t) + *other * t
    }

    fn to_point(&self) -> impl Point {
        p3::new(self.x, self.y, self.z)
    }
}

impl InnerSpace for v4 {
    fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    fn dist(&self, other: &Self) -> f32 {
        v4 {
            x: other.x - self.x,
            y: other.y - self.y,
            z: other.z - self.z,
            w: other.w - self.w,
        }
        .length()
    }

    fn angle(&self, other: &Self) -> f32 {
        (self.dot(other) / (self.length() * other.length())).acos()
    }

    fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }

    fn normalize(&self) -> Self {
        let factor = 1.0 / self.length();
        v4 {
            x: factor * self.x,
            y: factor * self.y,
            z: factor * self.z,
            w: factor * self.w,
        }
    }

    fn normalize_mut(&mut self) {
        let factor = 1.0 / self.length();
        self.x *= factor;
        self.y *= factor;
        self.z *= factor;
        self.w *= factor;
    }

    fn project_onto(&self, other: &Self) -> Self {
        let factor = self.dot(other) / other.dot(other);
        *other * factor
    }

    fn reflect(&self, normal: &Self) -> Self {
        *self - *normal * 2.0 * self.dot(normal)
    }

    fn lerp(&self, other: &Self, t: f32) -> Self {
        *self * (1.0 - t) + *other * t
    }

    fn to_point(&self) -> impl Point {
        p3::new(self.x / self.w, self.y / self.w, self.z / self.w)
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

generate_swizzles2!(v2,
    xx => (x, x), xy => (x, y),
    yx => (y, x), yy => (y, y)
);

generate_swizzles3!(v3,
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

generate_swizzles4!(v4,
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
