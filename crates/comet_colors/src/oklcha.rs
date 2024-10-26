use crate::{sRgba, Hsla, Hsva, Hwba, Laba, Lcha, LinearRgba, Oklaba, Xyza};

#[derive(Debug, Clone, PartialEq)]
pub struct Oklcha {
	lightness: f32,
	chroma: f32,
	hue: f32,
	alpha: f32
}

impl Oklcha {
	pub fn new(lightness: f32, chroma: f32, hue: f32, alpha: f32) -> Self {
		assert!((0.0..=1.0).contains(&lightness) && (0.0..=1.0).contains(&chroma) && (0.0..=360.0).contains(&hue) && (0.0..=1.0).contains(&alpha), "Ligthness needs to be in range 0..1\nChroma needs to be in range 0..1\nHue needs to be in range 0..360\nAlpha needs to be in range 0..1");
		Self {
			lightness,
			chroma,
			hue,
			alpha
		}
	}

	pub fn lightness(&self) -> f32 {
		self.lightness
	}

	pub fn chroma(&self) -> f32 {
		self.chroma
	}

	pub fn hue(&self) -> f32 {
		self.hue
	}

	pub fn alpha(&self) -> f32 {
		self.alpha
	}

	pub fn from_oklaba(oklaba: Oklaba) -> Self {
		Self {
			lightness: oklaba.lightness(),
			chroma: (oklaba.a()*oklaba.a() + oklaba.b()*oklaba.b()).sqrt(),
			hue: oklaba.b().atan2(oklaba.a()),
			alpha: oklaba.alpha()
		}
	}

	pub fn to_oklaba(&self) -> Oklaba {
		Oklaba::new(
			self.lightness(),
			self.chroma() * self.hue().cos(),
			self.chroma() * self.hue().sin(),
			self.alpha()
		)
	}

	pub fn to_linear(&self) -> LinearRgba {
		self.to_oklaba().to_linear()
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