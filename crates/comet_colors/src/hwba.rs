use comet_math::v4;
use crate::{sRgba, Color, Hsla, Hsva, Laba, Lcha, LinearRgba, Oklaba, Oklcha, Xyza};

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Hwba {
	hue: f32,
	whiteness: f32,
	blackness: f32,
	alpha: f32
}

impl Hwba {
	pub fn new(hue: f32, whiteness: f32, blackness: f32, alpha: f32) -> Self {
		assert!((0.0..=360.0).contains(&hue) && (0.0..=1.0).contains(&whiteness) && (0.0..=1.0).contains(&blackness) && (0.0..=1.0).contains(&alpha), "Hue needs to be in range 0..360\nWhiteness needs to be in range 0..1\nBlackness needs to be in range 0..1\nAlpha needs to be in range 0..1");
		Self {
			hue,
			whiteness,
			blackness,
			alpha
		}
	}

	pub fn hue(&self) -> f32 {
		self.hue
	}

	pub fn whiteness(&self) -> f32 {
		self.whiteness
	}

	pub fn blackness(&self) -> f32 {
		self.blackness
	}

	pub fn alpha(&self) -> f32 {
		self.alpha
	}

	pub fn from_rgba8(&self, rgba: sRgba<u8>) -> Self {
		let rgba = rgba.to_rbga();
		self.from_rgba(rgba)
	}

	pub fn from_rgba(&self, rgba: sRgba<f32>) -> Self {
		let max_rgb = rgba.red().max(rgba.green()).max(rgba.blue());
		let whiteness = rgba.red().min(rgba.green()).min(rgba.blue());

		let blackness = 1.0 - max_rgb;
		let chroma = max_rgb - whiteness;

		let hue = if chroma == 0.0 {
			0.0
		}
		else if max_rgb == rgba.red() {
			60.0 * ((rgba.green() - rgba.blue()) / chroma % 6.0)
		}
		else if max_rgb == rgba.green() {
			60.0 * ((rgba.blue() - rgba.red()) / chroma + 2.0)
		}
		else {
			60.0 * ((rgba.red() - rgba.green()) / chroma + 4.0)
		};

		let hue = if hue < 0.0 { hue + 360.0 } else { hue };

		Self {
			hue,
			whiteness,
			blackness,
			alpha: rgba.alpha()
		}
	}

	pub fn from_hsva(hsva: Hsva) -> Hwba {
		Hwba::new(
			hsva.hue(),
			(1.0 - hsva.saturation()) * hsva.value(),
			1.0 - hsva.value(),
			hsva.alpha()
		)
	}

	pub fn to_rgba8(&self) -> sRgba<u8> {
		let rgba = self.to_rgba();

		sRgba::<u8>::new(
			(rgba.red() * 255.0) as u8,
			(rgba.green() * 255.0) as u8,
			(rgba.blue() * 255.0) as u8,
			(rgba.alpha() * 255.0) as u8
		)
	}

	pub fn to_rgba(&self) -> sRgba<f32> {
		let w = self.whiteness.min(1.0 - self.blackness);
		let c = 1.0 - self.whiteness - self.blackness;

		let hue = (self.hue % 360.0 + 360.0) % 360.0;
		let h_prime = hue / 60.0;

		let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());

		let (r1, g1, b1) = match h_prime.floor() as u32 {
			0 => (c, x, 0.0),
			1 => (x, c, 0.0),
			2 => (0.0, c, x),
			3 => (0.0, x, c),
			4 => (x, 0.0, c),
			5 => (c, 0.0, x),
			_ => (0.0, 0.0, 0.0)
		};

		sRgba::<f32>::new(
			(r1 + w).min(1.0),
			(g1 + w).min(1.0),
			(b1 + w).min(1.0),
			self.alpha()
		)

	}

	pub fn to_hsva(&self) -> Hsva {
		let value = 1.0 - self.blackness;
		let saturation = 1.0 - (self.whiteness / value);

		Hsva::new(
			self.hue,
			saturation,
			value,
			self.alpha
		)

	}

	pub fn to_hsla(&self) -> Hsla {
		self.to_hsva().to_hsla()
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

impl Color for Hwba {
	fn to_wgpu(&self) -> wgpu::Color {
		self.to_linear().to_wgpu()
	}

	fn to_vec(&self) -> v4 {
		v4::new(self.hue, self.whiteness, self.blackness, self.alpha)
	}

	fn from_vec(color: v4) -> Self {
		Self::new(color.x(), color.y(), color.z(), color.w())
	}
}