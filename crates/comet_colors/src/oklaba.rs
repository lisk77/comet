use crate::{sRgba, Hsla, Hsva, Hwba, Laba, Lcha, LinearRgba, Oklcha, Xyza};

#[derive(Debug, Clone, PartialEq)]
pub struct Oklaba {
	lightness: f32,
	a: f32,
	b: f32,
	alpha: f32
}

impl Oklaba {
	pub fn new(lightness: f32, green_red: f32, blue_yellow: f32, alpha: f32) -> Self {
		assert!((0.0..=1.0).contains(&lightness) && (-1.0..=1.0).contains(&green_red) && (-1.0..=1.0).contains(&blue_yellow) && (0.0..=1.0).contains(&alpha), "Ligthness needs to be in range 0..1.0\nA needs to be in range -1.0..1.0\nB needs to be in range -1.0..1.0\nAlpha needs to be in range 0..1");
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

	pub fn from_linear(linear: LinearRgba) -> Self {
		let l = 0.4122214708 * linear.red() + 0.5363325363 * linear.green() + 0.0514459929 * linear.blue();
		let m = 0.2119034982 * linear.red() + 0.6806995451 * linear.green() + 0.1073969566 * linear.blue();
		let s = 0.0883024619 * linear.red() + 0.2817188376 * linear.green() + 0.6299787005 * linear.blue();

		let l_ = l.cbrt();
		let m_ = m.cbrt();
		let s_ = s.cbrt();

		Self {
			lightness: 0.2104542553*l_ + 0.7936177850*m_ - 0.0040720468*s_,
			a: 1.9779984951*l_ - 2.4285922050*m_ + 0.4505937099*s_,
			b: 0.0259040371*l_ + 0.7827717662*m_ - 0.8086757660*s_,
			alpha: linear.alpha()
		}
	}

	pub fn to_linear(&self) -> LinearRgba {
		let l_ = self.lightness + 0.3963377774 * self.a + 0.2158037573 * self.b;
		let m_ = self.lightness - 0.1055613458 * self.a - 0.0638541728 * self.b;
		let s_ = self.lightness - 0.0894841775 * self.a - 1.2914855480 * self.b;

		let l = l_*l_*l_;
		let m = m_*m_*m_;
		let s = s_*s_*s_;

		LinearRgba::new(
			4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
			-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
			-0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
			self.alpha
		)
	}

	pub fn to_rgba(&self) -> sRgba<f32> {
		self.to_linear().to_rgba()
	}

	pub fn to_rgba8(&self) -> sRgba<u8> {
		self.to_rgba().to_rgba8()
	}

	pub fn to_oklcha(&self) -> Oklcha {
		Oklcha::new(
			self.lightness,
			(self.a*self.a + self.b*self.b).sqrt(),
			self.b.atan2(self.a),
			self.alpha
		)
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