//! This crate provides a set of mathematical utilities for game development.
//! It includes definitions for points, vectors, matrices, quaternions, bezier curves, easing functions, noise generation, polynomials, and interpolation utilities.

pub use bezier::*;
pub use easings::*;
pub use interpolation::*;
pub use matrix::*;
pub use point::*;
pub use polynomial::*;
pub use vector::*;

pub mod bezier;
pub mod easings;
pub mod interpolation;
pub mod matrix;
pub mod noise;
pub mod point;
pub mod polynomial;
pub mod quaternion;
pub mod vector;
