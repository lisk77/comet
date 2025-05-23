pub use comet_math as math;
use comet_math::v4;
pub use linear_rgba::*;
pub use rgba::*;
pub use hwba::*;
pub use hsva::*;
pub use hsla::*;
pub use xyza::*;
pub use laba::*;
pub use lcha::*;
pub use oklaba::*;
pub use oklcha::*;

mod rgba;
mod linear_rgba;
mod hwba;
mod hsva;
mod hsla;
mod xyza;
mod laba;
mod lcha;
mod oklaba;
mod oklcha;

pub trait Color: Copy {
	fn to_wgpu(&self) -> wgpu::Color;
	fn to_linear(&self) -> LinearRgba;
	fn to_vec(&self) -> v4;
	fn from_vec(color: v4) -> Self;
}