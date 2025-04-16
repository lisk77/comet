use comet_math::v4;
use crate::{sRgba, Color, Hsva, Hwba, Laba, Lcha, LinearRgba, Oklaba, Oklcha, Xyza};

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Hsla {
	hue: f32,
	saturation: f32,
	lightness: f32,
	alpha: f32
}

impl Hsla {
	pub fn new(hue: f32, saturation: f32, lightness: f32, alpha: f32) -> Self {
		assert!((0.0..=360.0).contains(&hue) && (0.0..=1.0).contains(&saturation) && (0.0..=1.0).contains(&lightness) && (0.0..=1.0).contains(&alpha), "Hue needs to be in range 0..360\nSaturation needs to be in range 0..1\nLightness needs to be in range 0..1\nAlpha needs to be in range 0..1");
		Self {
			hue,
			saturation,
			lightness,
			alpha
		}
	}

	pub fn hue(&self) -> f32 {
		self.hue
	}

	pub fn saturation(&self) -> f32 {
		self.saturation
	}

	pub fn lightness(&self) -> f32 {
		self.lightness
	}

	pub fn alpha(&self) -> f32 {
		self.alpha
	}

	pub fn from_hsva(hsva: Hsva) -> Self {
		let lightness = hsva.value() * (1.0 - hsva.saturation() * 0.5);
		Self {
			hue: hsva.hue(),
			saturation: if lightness == 0.0 || lightness == 1.0 { 0.0 } else { (hsva.value() - lightness) / lightness.min(1.0 - lightness) },
			lightness,
			alpha: hsva.alpha()
		}
	}

	pub fn from_rgba(rgba: sRgba<f32>) -> Self {
		rgba
			.to_hwba()
			.to_hsva()
			.to_hsla()

	}

	pub fn to_hsva(&self) -> Hsva {
		let value = self.lightness() + self.saturation() * self.lightness().min(1.0 - self.lightness());
		Hsva::new(
			self.hue(),
			if value == 0.0 { 0.0 } else { 2.0 * (1.0 - self.lightness() / value) },
			value,
			self.alpha()
		)
	}

	pub fn to_hwba(&self) -> Hwba {
		self.to_hsva().to_hwba()
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

impl Color for Hsla {
	fn to_wgpu(&self) -> wgpu::Color {
		self.to_linear().to_wgpu()
	}

	fn to_vec(&self) -> v4 {
		v4::new(self.hue, self.saturation, self.lightness, self.alpha)
	}

	fn from_vec(color: v4) -> Self {
		Self::new(color.x(), color.y(), color.z(), color.w())
	}
}