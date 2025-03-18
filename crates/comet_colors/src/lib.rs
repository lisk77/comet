pub use comet_math as math;
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

pub trait Color {
	fn to_wgpu(&self) -> wgpu::Color;
}