pub use comet_math as math;
use comet_math::v4;
pub use hsla::*;
pub use hsva::*;
pub use hwba::*;
pub use laba::*;
pub use lcha::*;
pub use linear_rgba::*;
pub use oklaba::*;
pub use oklcha::*;
pub use rgba::*;
pub use xyza::*;

mod hsla;
mod hsva;
mod hwba;
mod laba;
mod lcha;
mod linear_rgba;
mod oklaba;
mod oklcha;
mod rgba;
mod xyza;

pub trait Color: Copy {
    fn to_wgpu(&self) -> wgpu::Color;
    fn to_linear(&self) -> LinearRgba;
    fn to_vec(&self) -> v4;
    fn from_vec(color: v4) -> Self;
}
