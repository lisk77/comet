use comet_math::v4;
use crate::{sRgba, Color, Hsla, Hsva, Hwba, Laba, Lcha, LinearRgba, Oklaba, Oklcha};

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Xyza {
	x: f32,
	y: f32,
	z: f32,
	alpha: f32
}

impl Xyza {
	pub fn new(x: f32, y: f32, z: f32, alpha: f32) -> Self {
		assert!((0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y) && (0.0..=1.5).contains(&z) && (0.0..=1.0).contains(&alpha), "X needs to be in range 0..1\nY needs to be in range 0..1\nZ needs to be in range 0..1\nAlpha needs to be in range 0..1");
		Self {
			x,
			y,
			z,
			alpha
		}
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

	pub fn alpha(&self) -> f32 {
		self.alpha
	}

	pub fn from_linear(linear: LinearRgba) -> Self {
		Self {
			x: 0.4124564 * linear.red() + 0.3575761 * linear.green() + 0.1804375 * linear.blue(),
			y: 0.2126729 * linear.red() + 0.7151522 * linear.green() + 0.0721750 * linear.blue(),
			z: 0.0193339 * linear.red() + 0.1191920 * linear.green() + 0.9503041 * linear.blue(),
			alpha: linear.alpha()
		}
	}

	pub fn to_linear(&self) -> LinearRgba {
		LinearRgba::new(
			3.2404542 * self.x + -1.5371385 * self.y + -0.4985314 * self.z,
			-0.9692660 * self.x +  1.8760108 * self.y +  0.0415560 * self.z,
			0.0556434 * self.x + -0.2040259 * self.y +  1.0572252 * self.z,
			self.alpha
		)
	}

	pub fn to_laba(&self) -> Laba {
		let reference_white = Xyza::new(0.95047, 1.0, 1.08883, 1.0);

		let x_r = self.x / reference_white.x;
		let y_r = self.y / reference_white.y;
		let z_r = self.z / reference_white.z;

		let epsilon: f32 = 0.008856;
		let kappa: f32 = 903.3;

		let f_x = if x_r > epsilon { x_r.cbrt() } else { ( kappa*x_r + 16.0 ) / 116.0 };
		let f_y = if x_r > epsilon { y_r.cbrt() } else { ( kappa*y_r + 16.0 ) / 116.0 };
		let f_z = if x_r > epsilon { z_r.cbrt() } else { ( kappa*z_r + 16.0 ) / 116.0 };

		Laba::new(
			1.16*f_y-0.16,
			5.0*( f_x - f_y ),
			2.0*( f_y - f_z ),
			self.alpha
		)
	}

	pub fn to_lcha(&self) -> Lcha {
		self.to_laba().to_lcha()
	}

	pub fn to_oklaba(&self) -> Oklaba {
		self.to_linear().to_oklaba()
	}

	pub fn to_oklcha(&self) -> Oklcha {
		self.to_oklaba().to_oklcha()
	}

	pub fn to_rgba(&self) -> sRgba<f32> {
		self.to_linear().to_rgba()
	}

	pub fn to_rgba8(&self) -> sRgba<u8> {
		self.to_linear().to_rgba8()
	}

	pub fn to_hwba(&self) -> Hwba {
		self.to_rgba().to_hwba()
	}

	pub fn to_hsva(&self) -> Hsva {
		self.to_hwba().to_hsva()
	}

	pub fn to_hsla(&self) -> Hsla {
		self.to_hsva().to_hsla()
	}
}

impl Color for Xyza {
	fn to_wgpu(&self) -> wgpu::Color {
		self.to_linear().to_wgpu()
	}

	fn to_vec(&self) -> v4 {
		v4::new(self.x, self.y, self.z, self.alpha)
	}

	fn from_vec(color: v4) -> Self {
		Self::new(color.x(), color.y(), color.z(), color.w())
	}
}