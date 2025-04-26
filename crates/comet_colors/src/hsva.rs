use comet_math::v4;
use crate::{sRgba, Color, Hsla, Hwba, Laba, Lcha, LinearRgba, Oklaba, Oklcha, Xyza};

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Hsva {
	hue: f32,
	saturation: f32,
	value: f32,
	alpha: f32
}

impl Hsva {
	pub fn new(hue: f32, saturation: f32, value: f32, alpha: f32) -> Self {
		assert!((0.0..=360.0).contains(&hue) && (0.0..=1.0).contains(&saturation) && (0.0..=1.0).contains(&value) && (0.0..=1.0).contains(&alpha), "Hue needs to be in range 0..1\nSaturation needs to be in range 0..1\nValue needs to be in range 0..1\nAlpha needs to be in range 0..1");
		Self {
			hue,
			saturation,
			value,
			alpha
		}
	}

	pub fn hue(&self) -> f32 {
		self.hue
	}

	pub fn saturation(&self) -> f32 {
		self.saturation
	}

	pub fn value(&self) -> f32 {
		self.value
	}

	pub fn alpha(&self) -> f32 {
		self.alpha
	}

	pub fn from_hwba(hwba: Hwba) -> Self {
		Self {
			hue: hwba.hue(),
			saturation: 1.0 - hwba.hue() / (1.0 - hwba.blackness()),
			value: 1.0 - hwba.blackness(),
			alpha: hwba.alpha()
		}
	}

	pub fn to_hsva(hsla: Hsla) -> Self {
		let value = hsla.lightness() + hsla.saturation() * hsla.lightness().min(1.0 - hsla.lightness());
		Self {
			hue: hsla.hue(),
			saturation: if value == 0.0 { 0.0 } else { 2.0 * (1.0 - hsla.lightness() / value) },
			value,
			alpha: hsla.alpha()
		}


	}

	pub fn to_hwba(&self) -> Hwba {
		Hwba::new(self.hue, (1.0 - self.saturation) * self.value, 1.0 - self.value, self.alpha)
	}

	pub fn to_hsla(&self) -> Hsla {
		let l = self.value * (1.0 - self.saturation * 0.5);
		Hsla::new(self.hue, if l == 0.0 || l == 1.0 { 0.0 } else { (self.value - l) / l.min(1.0 - l) }, l, self.alpha)
	}

	pub fn to_rgba(&self) -> sRgba<f32> {
		self.to_hwba().to_rgba()
	}

	pub fn to_rgba8(&self) -> sRgba<u8> {
		self.to_hwba().to_rgba8()
	}

	pub fn to_linear(&self) -> LinearRgba {
		self.to_rgba().to_linear()
	}

	pub fn to_xyza(&self) -> Xyza {
		self.to_linear().to_xyza()
	}

	pub fn to_laba(&self) -> Laba {
		self.to_xyza().to_laba()
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
}

impl Color for Hsva {
	fn to_wgpu(&self) -> wgpu::Color {
		self.to_linear().to_wgpu()
	}
	fn to_linear(&self) -> LinearRgba {
		self.to_linear()
	}

	fn to_vec(&self) -> v4 {
		v4::new(self.hue, self.saturation, self.value, self.alpha)
	}

	fn from_vec(color: v4) -> Self {
		Self::new(color.x(), color.y(), color.z(), color.w())
	}
}