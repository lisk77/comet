use wgpu;
use comet_math::v4;
use crate::{sRgba, Color, Hsla, Hsva, Hwba, Laba, Lcha, Oklaba, Oklcha, Xyza};

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct LinearRgba {
	red: f32,
	green: f32,
	blue: f32,
	alpha: f32
}

impl LinearRgba {
	pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
		assert!((0.0..=1.0).contains(&red) && (0.0..=1.0).contains(&green) && (0.0..=1.0).contains(&blue) && (0.0..=1.0).contains(&alpha), "Red needs to be in range 0..1\nGreen needs to be in range 0..1\nBlue needs to be in range 0..1\nAlpha needs to be in range 0..1");
		Self {
			red,
			green,
			blue,
			alpha
		}
	}

	pub fn red(&self) -> f32 {
		self.red
	}

	pub fn green(&self) -> f32 {
		self.green
	}

	pub fn blue(&self) -> f32 {
		self.blue
	}

	pub fn alpha(&self) -> f32 {
		self.alpha
	}

	pub fn from_rgba(srgba: sRgba<f32>) -> Self {
		Self {
			red: if srgba.red() <= 0.04045 { srgba.red() / 12.92 } else { ( ( srgba.red() + 0.055 ) / 1.055 ).powf(2.4) },
			green: if srgba.green() <= 0.04045 { srgba.green() / 12.92 } else { ( ( srgba.green() + 0.055 ) / 1.055 ).powf(2.4) },
			blue: if srgba.blue() <= 0.04045 { srgba.blue() / 12.92 } else { ( ( srgba.blue() + 0.055 ) / 1.055 ).powf(2.4) },
			alpha: srgba.alpha()
		}
	}

	pub fn from_xyza(xyz: Xyza) -> Self {
		Self {
			red:  3.2404542 * xyz.x() + -1.5371385 * xyz.y() + -0.4985314 * xyz.z(),
			green: -0.9692660 * xyz.x() +  1.8760108 * xyz.y() +  0.0415560 * xyz.z(),
			blue:  0.0556434 * xyz.x() + -0.2040259 * xyz.y() +  1.0572252 * xyz.z(),
			alpha:  1.0
		}
	}

	pub fn to_rgba(&self) -> sRgba<f32> {
		sRgba::<f32>::new(
			if self.red <= 0.0031308 { self.red * 12.92 } else { 1.055 * self.red.powf( 1.0 / 2.4 ) - 0.055 },
			if self.green <= 0.0031308 { self.green * 12.92 } else { 1.055 * self.green.powf( 1.0 / 2.4 ) - 0.055 },
			if self.blue <= 0.0031308 { self.blue * 12.92 } else { 1.055 * self.blue.powf( 1.0 / 2.4 ) - 0.055 },
			self.alpha
		)
	}

	pub fn to_rgba8(&self) -> sRgba<u8> {
		let color = self.to_rgba();

		sRgba::<u8>::new(
			(color.red() * 255.0) as u8,
			(color.green() * 255.0) as u8,
			(color.blue() * 255.0) as u8,
			(color.alpha() * 255.0) as u8,
		)
	}

	pub fn to_oklaba(&self) -> Oklaba {
		let l = 0.4122214708 * self.red + 0.5363325363 * self.green + 0.0514459929 * self.blue;
		let m = 0.2119034982 * self.red + 0.6806995451 * self.green + 0.1073969566 * self.blue;
		let s = 0.0883024619 * self.red + 0.2817188376 * self.green + 0.6299787005 * self.blue;

		let l_ = l.cbrt();
		let m_ = m.cbrt();
		let s_ = s.cbrt();

		Oklaba::new(
			0.2104542553*l_ + 0.7936177850*m_ - 0.0040720468*s_,
			1.9779984951*l_ - 2.4285922050*m_ + 0.4505937099*s_,
			0.0259040371*l_ + 0.7827717662*m_ - 0.8086757660*s_,
			self.alpha
		)
	}

	pub fn to_oklcha(&self) -> Oklcha {
		self.to_oklaba().to_oklcha()
	}

	pub fn to_xyza(&self) -> Xyza {
		Xyza::new(
			0.4124564 * self.red + 0.3575761 * self.green + 0.1804375 * self.blue,
			0.2126729 * self.red + 0.7151522 * self.green + 0.0721750 * self.blue,
			0.0193339 * self.red + 0.1191920 * self.green + 0.9503041 * self.blue,
			self.alpha
		)
	}

	pub fn to_laba(&self) -> Laba {
		self.to_xyza().to_laba()
	}

	pub fn to_lcha(&self) -> Lcha {
		self.to_laba().to_lcha()
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

impl Color for LinearRgba {
	fn to_wgpu(&self) -> wgpu::Color {
		wgpu::Color {
			r: self.red as f64,
			g: self.green as f64,
			b: self.blue as f64,
			a: self.alpha as f64
		}
	}

	fn to_vec(&self) -> v4 {
		v4::new(self.red, self.green, self.blue, self.alpha)
	}

	fn from_vec(color: v4) -> Self {
		Self::new(color.x(), color.y(), color.z(), color.w())
	}
}