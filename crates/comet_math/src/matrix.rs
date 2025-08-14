use crate::vector::{v2, v3, v4};
use std::ops::*;

trait LinearTransformation {
    fn det(&self) -> f32;
}

// ##################################################
// #                   MATRIX 2D                    #
// ##################################################

/// Representation of a 2x2 matrix.
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct m2 {
    x00: f32,
    x01: f32,
    x10: f32,
    x11: f32,
}

impl m2 {
    /// The zero matrix.
    pub const ZERO: Self = Self {
        x00: 0.0,
        x01: 0.0,
        x10: 0.0,
        x11: 0.0,
    };

    /// The identity matrix.
    pub const IDENTITY: Self = Self {
        x00: 1.0,
        x01: 0.0,
        x10: 0.0,
        x11: 1.0,
    };

    /// Creates a new 2x2 matrix with the given elements.
    pub fn new(x00: f32, x01: f32, x10: f32, x11: f32) -> Self {
        Self { x00, x01, x10, x11 }
    }

    /// Creates a new 2x2 matrix with the given vectors as its columns.
    pub fn from_cols(col1: v2, col2: v2) -> Self {
        Self {
            x00: col1.x(),
            x01: col1.y(),
            x10: col2.x(),
            x11: col2.y(),
        }
    }

    /// Creates a new 2x2 matrix with the given vectors as its rows.
    pub fn from_rows(row1: v2, row2: v2) -> Self {
        Self {
            x00: row1.x(),
            x01: row2.x(),
            x10: row1.y(),
            x11: row2.y(),
        }
    }

    /// Gets the element at the specified row and column.
    pub fn get(&self, row: usize, col: usize) -> Option<f32> {
        match (row, col) {
            (0, 0) => Some(self.x00),
            (0, 1) => Some(self.x01),
            (1, 0) => Some(self.x10),
            (1, 1) => Some(self.x11),
            _ => None,
        }
    }

    /// Sets the element at the specified row and column.
    pub fn set(&mut self, row: usize, col: usize, value: f32) {
        match (row, col) {
            (0, 0) => self.x00 = value,
            (0, 1) => self.x01 = value,
            (1, 0) => self.x10 = value,
            (1, 1) => self.x11 = value,
            _ => {}
        }
    }

    /// Gets the entire column at the specified index.
    pub fn col(&self, index: usize) -> Option<v2> {
        match index {
            0 => Some(v2::new(self.x00, self.x01)),
            1 => Some(v2::new(self.x10, self.x11)),
            _ => None,
        }
    }

    /// Gets the entire row at the specified index.
    pub fn row(&self, index: usize) -> Option<v2> {
        match index {
            0 => Some(v2::new(self.x00, self.x10)),
            1 => Some(v2::new(self.x01, self.x11)),
            _ => None,
        }
    }

    /// Returns the transpose of the matrix.
    pub fn transpose(&self) -> Self {
        Self {
            x00: self.x00,
            x01: self.x10,
            x10: self.x01,
            x11: self.x11,
        }
    }

    /// Returns a matrix with the same elements as the original matrix but in homogeneous form.
    pub fn to_homogeneous(&self) -> m3 {
        m3::new(
            self.x00, self.x01, 0.0, self.x10, self.x11, 0.0, 0.0, 0.0, 1.0,
        )
    }
}

impl Add for m2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            x00: self.x00 + rhs.x00,
            x01: self.x01 + rhs.x01,
            x10: self.x10 + rhs.x10,
            x11: self.x11 + rhs.x11,
        }
    }
}

impl Sub for m2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            x00: self.x00 - rhs.x00,
            x01: self.x01 - rhs.x01,
            x10: self.x10 - rhs.x10,
            x11: self.x11 - rhs.x11,
        }
    }
}

impl Mul<f32> for m2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self {
            x00: self.x00 * rhs,
            x01: self.x01 * rhs,
            x10: self.x10 * rhs,
            x11: self.x11 * rhs,
        }
    }
}

impl Mul<v2> for m2 {
    type Output = v2;

    fn mul(self, rhs: v2) -> v2 {
        v2::new(
            self.x00 * rhs.x() + self.x01 * rhs.y(),
            self.x10 * rhs.x() + self.x11 * rhs.y(),
        )
    }
}

impl Mul<m2> for m2 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self {
            x00: self.x00 * rhs.x00 + self.x01 * rhs.x10,
            x01: self.x00 * rhs.x01 + self.x01 * rhs.x11,
            x10: self.x10 * rhs.x00 + self.x11 * rhs.x10,
            x11: self.x10 * rhs.x01 + self.x11 * rhs.x11,
        }
    }
}

impl Div<f32> for m2 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self {
        Self {
            x00: self.x00 / rhs,
            x01: self.x01 / rhs,
            x10: self.x10 / rhs,
            x11: self.x11 / rhs,
        }
    }
}

impl Into<[[f32; 2]; 2]> for m2 {
    fn into(self) -> [[f32; 2]; 2] {
        [[self.x00, self.x01], [self.x10, self.x11]]
    }
}

// ##################################################
// #                   MATRIX 3D                    #
// ##################################################

/// Representation of a 3x3 matrix.
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct m3 {
    x00: f32,
    x01: f32,
    x02: f32,
    x10: f32,
    x11: f32,
    x12: f32,
    x20: f32,
    x21: f32,
    x22: f32,
}

impl m3 {
    /// The zero matrix.
    pub const ZERO: Self = Self {
        x00: 0.0,
        x01: 0.0,
        x02: 0.0,
        x10: 0.0,
        x11: 0.0,
        x12: 0.0,
        x20: 0.0,
        x21: 0.0,
        x22: 0.0,
    };

    /// The identity matrix.
    pub const IDENTITY: Self = Self {
        x00: 1.0,
        x01: 0.0,
        x02: 0.0,
        x10: 0.0,
        x11: 1.0,
        x12: 0.0,
        x20: 0.0,
        x21: 0.0,
        x22: 1.0,
    };

    /// Creates a new 3x3 matrix with the given elements.
    pub fn new(
        x00: f32,
        x01: f32,
        x02: f32,
        x10: f32,
        x11: f32,
        x12: f32,
        x20: f32,
        x21: f32,
        x22: f32,
    ) -> Self {
        Self {
            x00,
            x01,
            x02,
            x10,
            x11,
            x12,
            x20,
            x21,
            x22,
        }
    }

    /// Creates a new 3x3 matrix from the given columns.
    pub fn from_cols(col1: v3, col2: v3, col3: v3) -> Self {
        Self {
            x00: col1.x(),
            x01: col1.y(),
            x02: col1.z(),
            x10: col2.x(),
            x11: col2.y(),
            x12: col2.z(),
            x20: col3.x(),
            x21: col3.y(),
            x22: col3.z(),
        }
    }

    /// Creates a new 3x3 matrix from the given rows.
    pub fn from_rows(row1: v3, row2: v3, row3: v3) -> Self {
        Self {
            x00: row1.x(),
            x01: row2.x(),
            x02: row3.x(),
            x10: row1.y(),
            x11: row2.y(),
            x12: row3.y(),
            x20: row1.z(),
            x21: row2.z(),
            x22: row3.z(),
        }
    }

    /// Gets the element at the given row and column.
    pub fn get(&self, row: usize, col: usize) -> Option<f32> {
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

    /// Sets the element at the given row and column.
    pub fn set(&mut self, row: usize, col: usize, value: f32) {
        match (row, col) {
            (0, 0) => self.x00 = value,
            (0, 1) => self.x01 = value,
            (0, 2) => self.x02 = value,
            (1, 0) => self.x10 = value,
            (1, 1) => self.x11 = value,
            (1, 2) => self.x12 = value,
            (2, 0) => self.x20 = value,
            (2, 1) => self.x21 = value,
            (2, 2) => self.x22 = value,
            _ => {}
        }
    }

    /// Gets the entire column at the given index.
    pub fn col(&self, index: usize) -> Option<v3> {
        match index {
            0 => Some(v3::new(self.x00, self.x01, self.x02)),
            1 => Some(v3::new(self.x10, self.x11, self.x12)),
            2 => Some(v3::new(self.x20, self.x21, self.x22)),
            _ => None,
        }
    }

    /// Gets the entire row at the given index.
    pub fn row(&self, index: usize) -> Option<v3> {
        match index {
            0 => Some(v3::new(self.x00, self.x10, self.x20)),
            1 => Some(v3::new(self.x01, self.x11, self.x21)),
            2 => Some(v3::new(self.x02, self.x12, self.x22)),
            _ => None,
        }
    }

    /// Returns the transpose of the matrix.
    pub fn transpose(&self) -> Self {
        Self {
            x00: self.x00,
            x01: self.x10,
            x02: self.x20,
            x10: self.x01,
            x11: self.x11,
            x12: self.x21,
            x20: self.x02,
            x21: self.x12,
            x22: self.x22,
        }
    }

    /// Returns a matrix with the same elements as the original matrix but in homogeneous form.
    pub fn to_homogeneous(&self) -> m4 {
        m4::new(
            self.x00, self.x01, self.x02, 0.0, self.x10, self.x11, self.x12, 0.0, self.x20,
            self.x21, self.x22, 0.0, 0.0, 0.0, 0.0, 1.0,
        )
    }
}

impl Add for m3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            x00: self.x00 + rhs.x00,
            x01: self.x01 + rhs.x01,
            x02: self.x02 + rhs.x02,
            x10: self.x10 + rhs.x10,
            x11: self.x11 + rhs.x11,
            x12: self.x12 + rhs.x12,
            x20: self.x20 + rhs.x20,
            x21: self.x21 + rhs.x21,
            x22: self.x22 + rhs.x22,
        }
    }
}

impl Sub for m3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            x00: self.x00 - rhs.x00,
            x01: self.x01 - rhs.x01,
            x02: self.x02 - rhs.x02,
            x10: self.x10 - rhs.x10,
            x11: self.x11 - rhs.x11,
            x12: self.x12 - rhs.x12,
            x20: self.x20 - rhs.x20,
            x21: self.x21 - rhs.x21,
            x22: self.x22 - rhs.x22,
        }
    }
}

impl Mul<f32> for m3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self {
            x00: self.x00 * rhs,
            x01: self.x01 * rhs,
            x02: self.x02 * rhs,
            x10: self.x10 * rhs,
            x11: self.x11 * rhs,
            x12: self.x12 * rhs,
            x20: self.x20 * rhs,
            x21: self.x21 * rhs,
            x22: self.x22 * rhs,
        }
    }
}

impl Mul<v3> for m3 {
    type Output = v3;

    fn mul(self, rhs: v3) -> v3 {
        v3::new(
            self.x00 * rhs.x() + self.x01 * rhs.y() + self.x02 * rhs.z(),
            self.x10 * rhs.x() + self.x11 * rhs.y() + self.x12 * rhs.z(),
            self.x20 * rhs.x() + self.x21 * rhs.y() + self.x22 * rhs.z(),
        )
    }
}

impl Mul<m3> for m3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self {
            x00: self.x00 * rhs.x00 + self.x01 * rhs.x10 + self.x02 * rhs.x20,
            x01: self.x00 * rhs.x01 + self.x01 * rhs.x11 + self.x02 * rhs.x21,
            x02: self.x00 * rhs.x02 + self.x01 * rhs.x12 + self.x02 * rhs.x22,
            x10: self.x10 * rhs.x00 + self.x11 * rhs.x10 + self.x12 * rhs.x20,
            x11: self.x10 * rhs.x01 + self.x11 * rhs.x11 + self.x12 * rhs.x21,
            x12: self.x10 * rhs.x02 + self.x11 * rhs.x12 + self.x12 * rhs.x22,
            x20: self.x20 * rhs.x00 + self.x21 * rhs.x10 + self.x22 * rhs.x20,
            x21: self.x20 * rhs.x01 + self.x21 * rhs.x11 + self.x22 * rhs.x21,
            x22: self.x20 * rhs.x02 + self.x21 * rhs.x12 + self.x22 * rhs.x22,
        }
    }
}

impl Div<f32> for m3 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self {
        Self {
            x00: self.x00 / rhs,
            x01: self.x01 / rhs,
            x02: self.x02 / rhs,
            x10: self.x10 / rhs,
            x11: self.x11 / rhs,
            x12: self.x12 / rhs,
            x20: self.x20 / rhs,
            x21: self.x21 / rhs,
            x22: self.x22 / rhs,
        }
    }
}

impl Into<[[f32; 3]; 3]> for m3 {
    fn into(self) -> [[f32; 3]; 3] {
        [
            [self.x00, self.x01, self.x02],
            [self.x10, self.x11, self.x12],
            [self.x20, self.x21, self.x22],
        ]
    }
}

// ##################################################
// #                   MATRIX 4D                    #
// ##################################################

/// Representation of a 4x4 matrix.
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct m4 {
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
    x33: f32,
}

impl m4 {
    /// The zero matrix.
    pub const ZERO: Self = Self {
        x00: 0.0,
        x01: 0.0,
        x02: 0.0,
        x03: 0.0,
        x10: 0.0,
        x11: 0.0,
        x12: 0.0,
        x13: 0.0,
        x20: 0.0,
        x21: 0.0,
        x22: 0.0,
        x23: 0.0,
        x30: 0.0,
        x31: 0.0,
        x32: 0.0,
        x33: 0.0,
    };

    /// The identity matrix.
    pub const IDENTITY: Self = Self {
        x00: 1.0,
        x01: 0.0,
        x02: 0.0,
        x03: 0.0,
        x10: 0.0,
        x11: 1.0,
        x12: 0.0,
        x13: 0.0,
        x20: 0.0,
        x21: 0.0,
        x22: 1.0,
        x23: 0.0,
        x30: 0.0,
        x31: 0.0,
        x32: 0.0,
        x33: 1.0,
    };

    /// The OpenGL conversion matrix.
    pub const OPENGL_CONV: Self = Self {
        x00: 1.0,
        x01: 0.0,
        x02: 0.0,
        x03: 0.0,
        x10: 0.0,
        x11: 1.0,
        x12: 0.0,
        x13: 0.0,
        x20: 0.0,
        x21: 0.0,
        x22: 0.5,
        x23: 0.5,
        x30: 0.0,
        x31: 0.0,
        x32: 0.0,
        x33: 1.0,
    };

    /// Creates a new matrix with the given elements.
    pub fn new(
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
        x33: f32,
    ) -> Self {
        Self {
            x00,
            x01,
            x02,
            x03,
            x10,
            x11,
            x12,
            x13,
            x20,
            x21,
            x22,
            x23,
            x30,
            x31,
            x32,
            x33,
        }
    }

    /// Creates a new matrix from the given columns.
    pub fn from_cols(col1: v4, col2: v4, col3: v4, col4: v4) -> Self {
        Self {
            x00: col1.x(),
            x01: col1.y(),
            x02: col1.z(),
            x03: col1.w(),
            x10: col2.x(),
            x11: col2.y(),
            x12: col2.z(),
            x13: col2.w(),
            x20: col3.x(),
            x21: col3.y(),
            x22: col3.z(),
            x23: col3.w(),
            x30: col4.x(),
            x31: col4.y(),
            x32: col4.z(),
            x33: col4.w(),
        }
    }

    /// Creates a new matrix from the given rows.
    pub fn from_rows(row1: v4, row2: v4, row3: v4, row4: v4) -> Self {
        Self {
            x00: row1.x(),
            x01: row2.x(),
            x02: row3.x(),
            x03: row4.x(),
            x10: row1.y(),
            x11: row2.y(),
            x12: row3.y(),
            x13: row4.y(),
            x20: row1.z(),
            x21: row2.z(),
            x22: row3.z(),
            x23: row4.z(),
            x30: row1.w(),
            x31: row2.w(),
            x32: row3.w(),
            x33: row4.w(),
        }
    }

    /// Gets the element at the given row and column.
    pub fn get(&self, row: usize, col: usize) -> Option<f32> {
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

    /// Sets the element at the given row and column.
    pub fn set(&mut self, row: usize, col: usize, value: f32) {
        match (row, col) {
            (0, 0) => self.x00 = value,
            (0, 1) => self.x01 = value,
            (0, 2) => self.x02 = value,
            (0, 3) => self.x03 = value,
            (1, 0) => self.x10 = value,
            (1, 1) => self.x11 = value,
            (1, 2) => self.x12 = value,
            (1, 3) => self.x13 = value,
            (2, 0) => self.x20 = value,
            (2, 1) => self.x21 = value,
            (2, 2) => self.x22 = value,
            (2, 3) => self.x23 = value,
            (3, 0) => self.x30 = value,
            (3, 1) => self.x31 = value,
            (3, 2) => self.x32 = value,
            (3, 3) => self.x33 = value,
            _ => {}
        }
    }

    /// Gets the entire column at the given index.
    pub fn col(&self, index: usize) -> Option<v4> {
        match index {
            0 => Some(v4::new(self.x00, self.x01, self.x02, self.x03)),
            1 => Some(v4::new(self.x10, self.x11, self.x12, self.x13)),
            2 => Some(v4::new(self.x20, self.x21, self.x22, self.x23)),
            3 => Some(v4::new(self.x30, self.x31, self.x32, self.x33)),
            _ => None,
        }
    }

    /// Gets the entire row at the given index.
    pub fn row(&self, index: usize) -> Option<v4> {
        match index {
            0 => Some(v4::new(self.x00, self.x10, self.x20, self.x30)),
            1 => Some(v4::new(self.x01, self.x11, self.x21, self.x31)),
            2 => Some(v4::new(self.x02, self.x12, self.x22, self.x32)),
            3 => Some(v4::new(self.x03, self.x13, self.x23, self.x33)),
            _ => None,
        }
    }

    /// Returns the transpose of the matrix.
    pub fn transpose(&self) -> Self {
        Self {
            x00: self.x00,
            x01: self.x10,
            x02: self.x20,
            x03: self.x30,
            x10: self.x01,
            x11: self.x11,
            x12: self.x21,
            x13: self.x31,
            x20: self.x02,
            x21: self.x12,
            x22: self.x22,
            x23: self.x32,
            x30: self.x03,
            x31: self.x13,
            x32: self.x23,
            x33: self.x33,
        }
    }

    /// Generates the orthographic projection matrix.
    pub fn orthographic_projection(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Self {
        let mut m = Self::IDENTITY;

        m.x00 = 2.0 / (right - left);
        m.x11 = 2.0 / (top - bottom);
        m.x22 = -2.0 / (far - near);
        m.x03 = -(right + left) / (right - left);
        m.x13 = -(top + bottom) / (top - bottom);
        m.x32 = -(far + near) / (far - near);

        m
    }
}

impl Add for m4 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            x00: self.x00 + rhs.x00,
            x01: self.x01 + rhs.x01,
            x02: self.x02 + rhs.x02,
            x03: self.x03 + rhs.x03,
            x10: self.x10 + rhs.x10,
            x11: self.x11 + rhs.x11,
            x12: self.x12 + rhs.x12,
            x13: self.x13 + rhs.x13,
            x20: self.x20 + rhs.x20,
            x21: self.x21 + rhs.x21,
            x22: self.x22 + rhs.x22,
            x23: self.x23 + rhs.x23,
            x30: self.x30 + rhs.x30,
            x31: self.x31 + rhs.x31,
            x32: self.x32 + rhs.x32,
            x33: self.x33 + rhs.x33,
        }
    }
}

impl Sub for m4 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            x00: self.x00 - rhs.x00,
            x01: self.x01 - rhs.x01,
            x02: self.x02 - rhs.x02,
            x03: self.x03 - rhs.x03,
            x10: self.x10 - rhs.x10,
            x11: self.x11 - rhs.x11,
            x12: self.x12 - rhs.x12,
            x13: self.x13 - rhs.x13,
            x20: self.x20 - rhs.x20,
            x21: self.x21 - rhs.x21,
            x22: self.x22 - rhs.x22,
            x23: self.x23 - rhs.x23,
            x30: self.x30 - rhs.x30,
            x31: self.x31 - rhs.x31,
            x32: self.x32 - rhs.x32,
            x33: self.x33 - rhs.x33,
        }
    }
}

impl Mul<f32> for m4 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self {
            x00: self.x00 * rhs,
            x01: self.x01 * rhs,
            x02: self.x02 * rhs,
            x03: self.x03 * rhs,
            x10: self.x10 * rhs,
            x11: self.x11 * rhs,
            x12: self.x12 * rhs,
            x13: self.x13 * rhs,
            x20: self.x20 * rhs,
            x21: self.x21 * rhs,
            x22: self.x22 * rhs,
            x23: self.x23 * rhs,
            x30: self.x30 * rhs,
            x31: self.x31 * rhs,
            x32: self.x32 * rhs,
            x33: self.x33 * rhs,
        }
    }
}

impl Mul<v4> for m4 {
    type Output = v4;

    fn mul(self, rhs: v4) -> v4 {
        v4::new(
            self.x00 * rhs.x() + self.x01 * rhs.y() + self.x02 * rhs.z() + self.x03 * rhs.w(),
            self.x10 * rhs.x() + self.x11 * rhs.y() + self.x12 * rhs.z() + self.x13 * rhs.w(),
            self.x20 * rhs.x() + self.x21 * rhs.y() + self.x22 * rhs.z() + self.x23 * rhs.w(),
            self.x30 * rhs.x() + self.x31 * rhs.y() + self.x32 * rhs.z() + self.x33 * rhs.w(),
        )
    }
}

impl Mul<m4> for m4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self {
            x00: self.x00 * rhs.x00 + self.x01 * rhs.x10 + self.x02 * rhs.x20 + self.x03 * rhs.x30,
            x01: self.x00 * rhs.x01 + self.x01 * rhs.x11 + self.x02 * rhs.x21 + self.x03 * rhs.x31,
            x02: self.x00 * rhs.x02 + self.x01 * rhs.x12 + self.x02 * rhs.x22 + self.x03 * rhs.x32,
            x03: self.x00 * rhs.x03 + self.x01 * rhs.x13 + self.x02 * rhs.x23 + self.x03 * rhs.x33,
            x10: self.x10 * rhs.x00 + self.x11 * rhs.x10 + self.x12 * rhs.x20 + self.x13 * rhs.x30,
            x11: self.x10 * rhs.x01 + self.x11 * rhs.x11 + self.x12 * rhs.x21 + self.x13 * rhs.x31,
            x12: self.x10 * rhs.x02 + self.x11 * rhs.x12 + self.x12 * rhs.x22 + self.x13 * rhs.x32,
            x13: self.x10 * rhs.x03 + self.x11 * rhs.x13 + self.x12 * rhs.x23 + self.x13 * rhs.x33,
            x20: self.x20 * rhs.x00 + self.x21 * rhs.x10 + self.x22 * rhs.x20 + self.x23 * rhs.x30,
            x21: self.x20 * rhs.x01 + self.x21 * rhs.x11 + self.x22 * rhs.x21 + self.x23 * rhs.x31,
            x22: self.x20 * rhs.x02 + self.x21 * rhs.x12 + self.x22 * rhs.x22 + self.x23 * rhs.x32,
            x23: self.x20 * rhs.x03 + self.x21 * rhs.x13 + self.x22 * rhs.x23 + self.x23 * rhs.x33,
            x30: self.x30 * rhs.x00 + self.x31 * rhs.x10 + self.x32 * rhs.x20 + self.x33 * rhs.x30,
            x31: self.x30 * rhs.x01 + self.x31 * rhs.x11 + self.x32 * rhs.x21 + self.x33 * rhs.x31,
            x32: self.x30 * rhs.x02 + self.x31 * rhs.x12 + self.x32 * rhs.x22 + self.x33 * rhs.x32,
            x33: self.x30 * rhs.x03 + self.x31 * rhs.x13 + self.x32 * rhs.x23 + self.x33 * rhs.x33,
        }
    }
}

impl Div<f32> for m4 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self {
        Self {
            x00: self.x00 / rhs,
            x01: self.x01 / rhs,
            x02: self.x02 / rhs,
            x03: self.x03 / rhs,
            x10: self.x10 / rhs,
            x11: self.x11 / rhs,
            x12: self.x12 / rhs,
            x13: self.x13 / rhs,
            x20: self.x20 / rhs,
            x21: self.x21 / rhs,
            x22: self.x22 / rhs,
            x23: self.x23 / rhs,
            x30: self.x30 / rhs,
            x31: self.x31 / rhs,
            x32: self.x32 / rhs,
            x33: self.x33 / rhs,
        }
    }
}

impl Into<[[f32; 4]; 4]> for m4 {
    fn into(self) -> [[f32; 4]; 4] {
        [
            [self.x00, self.x01, self.x02, self.x03],
            [self.x10, self.x11, self.x12, self.x13],
            [self.x20, self.x21, self.x22, self.x23],
            [self.x30, self.x31, self.x32, self.x33],
        ]
    }
}

impl LinearTransformation for m2 {
    fn det(&self) -> f32 {
        self.x00 * self.x11 - self.x01 * self.x10
    }
}

impl LinearTransformation for m3 {
    fn det(&self) -> f32 {
        self.x00 * (self.x11 * self.x22 - self.x12 * self.x21)
            - self.x01 * (self.x10 * self.x22 - self.x12 * self.x20)
            + self.x02 * (self.x10 * self.x21 - self.x11 * self.x20)
    }
}

impl LinearTransformation for m4 {
    fn det(&self) -> f32 {
        self.x00 * self.x11 * self.x22 * self.x33
            + self.x00 * self.x12 * self.x23 * self.x31
            + self.x00 * self.x13 * self.x21 * self.x32
            + self.x01 * self.x10 * self.x23 * self.x32
            + self.x01 * self.x12 * self.x20 * self.x33
            + self.x01 * self.x13 * self.x22 * self.x30
            + self.x02 * self.x10 * self.x21 * self.x33
            + self.x02 * self.x11 * self.x23 * self.x30
            + self.x02 * self.x13 * self.x20 * self.x31
            + self.x03 * self.x10 * self.x22 * self.x31
            + self.x03 * self.x11 * self.x20 * self.x32
            + self.x03 * self.x12 * self.x21 * self.x30
            - self.x00 * self.x11 * self.x23 * self.x32
            - self.x00 * self.x12 * self.x21 * self.x33
            - self.x00 * self.x13 * self.x22 * self.x31
            - self.x01 * self.x10 * self.x22 * self.x33
            - self.x01 * self.x12 * self.x23 * self.x30
            - self.x01 * self.x13 * self.x20 * self.x32
            - self.x02 * self.x10 * self.x23 * self.x31
            - self.x02 * self.x11 * self.x20 * self.x33
            - self.x02 * self.x13 * self.x21 * self.x30
            - self.x03 * self.x10 * self.x21 * self.x32
            - self.x03 * self.x11 * self.x22 * self.x30
            - self.x03 * self.x12 * self.x20 * self.x31
    }
}
