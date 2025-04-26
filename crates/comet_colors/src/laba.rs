use comet_math::v4;
use crate::{sRgba, Color, Hsla, Hsva, Hwba, Lcha, LinearRgba, Oklaba, Oklcha, Xyza};

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Laba {
	lightness: f32,
	a: f32,
	b: f32,
	alpha: f32
}

impl Laba {
	pub fn new(lightness: f32, green_red: f32, blue_yellow: f32, alpha: f32) -> Self {
		assert!((0.0..=1.5).contains(&lightness) && (-1.5..=1.5).contains(&green_red) && (-1.5..=1.5).contains(&blue_yellow) && (0.0..=1.0).contains(&alpha), "Ligthness needs to be in range 0..1.5\nA needs to be in range -1.5..1.5\nB needs to be in range -1.5..1.5\nAlpha needs to be in range 0..1");
		Self {
			lightness,
			a: green_red,
			b: blue_yellow,
			alpha
		}
	}

	pub fn lightness(&self) -> f32 {
		self.lightness
	}

	pub fn a(&self) -> f32 {
		self.a
	}

	pub fn b(&self) -> f32 {
		self.b
	}

	pub fn alpha(&self) -> f32 {
		self.alpha
	}

	pub fn from_xyza(xyza: Xyza) -> Self {
		let reference_white = Xyza::new(0.95047, 1.0, 1.08883, 1.0);

		let x_r = xyza.x() / reference_white.x();
		let y_r = xyza.y() / reference_white.y();
		let z_r = xyza.z() / reference_white.z();

		let epsilon: f32 = 0.008856;
		let kappa: f32 = 903.3;

		let f_x = if x_r > epsilon { x_r.cbrt() } else { ( kappa*x_r + 16.0 ) / 116.0 };
		let f_y = if x_r > epsilon { y_r.cbrt() } else { ( kappa*y_r + 16.0 ) / 116.0 };
		let f_z = if x_r > epsilon { z_r.cbrt() } else { ( kappa*z_r + 16.0 ) / 116.0 };

		Self {
			lightness: 1.16*f_y-0.16,
			a: 5.0*( f_x - f_y ),
			b: 2.0*( f_y - f_z ),
			alpha: xyza.alpha()
		}
	}

	pub fn to_lcha(&self) -> Lcha {
		let hue: f32 = self.b.atan2(self.a).to_degrees();
		Lcha::new(
			self.lightness,
			(self.a*self.a + self.b*self.b).sqrt(),
			if hue < 0.0 { hue + 360.0 } else { hue },
			self.alpha
		)
	}

	pub fn to_xyza(&self) -> Xyza{
		let epsilon: f32 = 0.008856;
		let kappa: f32 = 903.3;

		let l = 100. * self.lightness;
		let a = 100. * self.a;
		let b = 100. * self.b;

		let fy = (l + 16.0) / 116.0;
		let fx = a / 500.0 + fy;
		let fz = fy - b / 200.0;
		let xr = {
			let fx3 = fx.powf(3.0);

			if fx3 > epsilon {
				fx3
			} else {
				(116.0 * fx - 16.0) / kappa
			}
		};
		let yr = if l > epsilon * kappa {
			((l + 16.0) / 116.0).powf(3.0)
		} else {
			l / kappa
		};
		let zr = {
			let fz3 = fz.powf(3.0);

			if fz3 > epsilon {
				fz3
			} else {
				(116.0 * fz - 16.0) / kappa
			}
		};

		let x = xr * 0.95047;
		let y = yr * 1.0;
		let z = zr * 1.08883;

		Xyza::new(x, y, z, self.alpha)
	}

	pub fn to_linear(&self) -> LinearRgba {
		self.to_xyza().to_linear()
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
		self.to_rgba().to_rgba8()
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

impl Color for Laba {
	fn to_wgpu(&self) -> wgpu::Color {
		self.to_linear().to_wgpu()
	}
	fn to_linear(&self) -> LinearRgba {
		self.to_linear()
	}

	fn to_vec(&self) -> v4 {
		v4::new(self.lightness, self.a, self.b, self.alpha)
	}

	fn from_vec(color: v4) -> Self {
		Self::new(color.x(), color.y(), color.z(), color.w())
	}
}