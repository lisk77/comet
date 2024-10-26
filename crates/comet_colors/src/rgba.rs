use crate::{math::Vec4, Hsla, Hsva, Hwba, Laba, Lcha, LinearRgba, Oklaba, Oklcha, Xyza};

/// sRGB representation of color
/// There are two variants: `sRgba<u8>` and `sRgba<f32>`
/// The first one is your standard 0..255 RGB and the second is the normalized version with range 0..1
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub struct sRgba<T> {
	red: T,
	green: T,
	blue: T,
	alpha: T,
}

impl sRgba<u8> {
	pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
		assert!((0..=255).contains(&red) && (0..=255).contains(&green) && (0..=255).contains(&blue) && (0..=255).contains(&alpha), "Red needs to be in range 0..255\nGreen needs to be in range 0..255\nBlue needs to be in range 0..255\nAlpha needs to be in range 0..255");
		Self {
			red,
			green,
			blue,
			alpha
		}
	}

	pub fn red(&self) -> u8 {
		self.red
	}

	pub fn green(&self) -> u8 {
		self.green
	}

	pub fn blue(&self) -> u8 {
		self.blue
	}

	pub fn alpha(&self) -> u8 {
		self.alpha
	}

	pub fn from_hex(hex: &str) -> Self {
		let hex = hex.trim_start_matches("#");

		if hex.len() != 8 {
			panic!("The length of the hex string is not equal to 8!");
		}

		let red = match u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Red part is not a hex value!") {
			Ok(v) => v,
			Err(err) => panic!("{}", err)
		};

		let green = match u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Green part is not a hex value!") {
			Ok(v) => v,
			Err(err) => panic!("{}", err)
		};

		let blue = match u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Blue part is not a hex value!") {
			Ok(v) => v,
			Err(err) => panic!("{}", err)
		};

		let alpha = match u8::from_str_radix(&hex[6..8], 16).map_err(|_| "Alpha part is not a hex value!") {
			Ok(v) => v,
			Err(err) => panic!("{}", err)
		};

		Self {
			red,
			green,
			blue,
			alpha
		}
	}

	pub fn from_rgba(rgba: sRgba<f32>) -> Self {
		rgba.to_rgba8()
	}

	pub fn from_hwba(hwba: Hwba) -> Self {
		hwba.to_rgba8()
	}

	pub fn from_hsva(hvsa: Hsva) -> Self {
		hvsa.to_rgba8()
	}

	pub fn from_hsla(hsla: Hsla) -> Self {
		hsla.to_rgba8()
	}

	pub fn from_xyza(xyza: Xyza) -> Self {
		xyza.to_rgba8()
	}

	pub fn from_laba(laba: Laba) -> Self {
		laba.to_rgba8()
	}

	pub fn from_lcha(lcha: Lcha) -> Self {
		lcha.to_rgba8()
	}

	pub fn from_oklaba(oklaba: Oklaba) -> Self {
		oklaba.to_rgba8()
	}

	pub fn from_oklcha(oklcha: Oklcha) -> Self {
		oklcha.to_rgba8()
	}

	pub fn to_rbga(&self) -> sRgba<f32> {
		sRgba {
			red: self.red as f32/255.0,
			green: self.green as f32/255.0,
			blue: self.blue as f32/255.0,
			alpha: self.alpha as f32/255.0
		}
	}

	pub fn to_linear(&self) -> LinearRgba {
		self.to_rbga().to_linear()
	}

	pub fn to_hwba(&self) -> Hwba {
		self.to_rbga().to_hwba()
	}

	pub fn to_hsva(&self) -> Hsva {
		self.to_hwba().to_hsva()
	}

	pub fn to_hsla(&self) -> Hsla {
		self.to_hsva().to_hsla()
	}

	pub fn to_oklaba(&self) -> Oklaba {
		self.to_linear().to_oklaba()
	}

	pub fn to_vec(&self) -> Vec4 {
		Vec4::new(
			self.red as f32,
			self.green as f32,
			self.blue as f32,
			self.alpha as f32
		)
	}
}

impl sRgba<f32> {
	pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
		assert!((0.0..1.0).contains(&red) && (0.0..1.0).contains(&green) && (0.0..1.0).contains(&blue) && (0.0..1.0).contains(&alpha), "Red needs to be in range 0..1\nGreen needs to be in range 0..1\nBlue needs to be in range 0..1\nAlpha needs to be in range 0..1");
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

	pub fn alpha(&self) ->f32 {
		self.alpha
	}

	pub fn from_hex(hex: &str) -> Self {
		let hex = hex.trim_start_matches("#");

		if hex.len() != 8 {
			panic!("The length of the hex string is not equal to 6!");
		}

		let r = match u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Red is not a hex value!") {
			Ok(v) => v,
			Err(err) => panic!("{}", err)
		};

		let g = match u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Green is not a hex value!") {
			Ok(v) => v,
			Err(err) => panic!("{}", err)
		};

		let b = match u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Blue is not a hex value!") {
			Ok(v) => v,
			Err(err) => panic!("{}", err)
		};

		let a = match u8::from_str_radix(&hex[6..8], 16).map_err(|_| "Alpha is not a hex value!") {
			Ok(v) => v,
			Err(err) => panic!("{}", err)
		};

		Self {
			red: r as f32 / 255.0,
			green: g as f32 / 255.0,
			blue: b as f32 / 255.0,
			alpha: a as f32 / 255.0
		}
	}

	pub fn from_linear(linear: LinearRgba) -> Self {
		Self {
			red: if linear.red() <= 0.0031308 { 12.92 * linear.red() } else { 1.055 * linear.red().powf(1.0 / 2.4) - 0.055 },
			green: if linear.green() <= 0.0031308 { 12.92 * linear.green() } else { 1.055 * linear.green().powf(1.0 / 2.4) - 0.055 },
			blue: if linear.blue() <= 0.0031308 { 12.92 * linear.blue() } else { 1.055 * linear.blue().powf(1.0 / 2.4) - 0.055 },
			alpha: linear.alpha()
		}
	}

	pub fn from_rgba8(rgba: sRgba<u8>) -> Self {
		Self {
			red: rgba.red() as f32 / 255.0,
			green: rgba.green() as f32 / 255.0,
			blue: rgba.blue() as f32 / 255.0,
			alpha: rgba.alpha() as f32 / 255.0
		}
	}

	pub fn from_hwba(hwba: Hwba) -> Self {
		hwba.to_rgba()
	}

	pub fn from_hsva(hvsa: Hsva) -> Self {
		hvsa.to_rgba()
	}

	pub fn from_hsla(hsla: Hsla) -> Self {
		hsla.to_rgba()
	}

	pub fn from_xyza(xyza: Xyza) -> Self {
		xyza.to_rgba()
	}

	pub fn from_laba(laba: Laba) -> Self {
		laba.to_rgba()
	}

	pub fn from_lcha(lcha: Lcha) -> Self {
		lcha.to_rgba()
	}

	pub fn from_oklaba(oklaba: Oklaba) -> Self {
		oklaba.to_rgba()
	}

	pub fn from_oklcha(oklcha: Oklcha) -> Self {
		oklcha.to_rgba()
	}

	pub fn to_rgba8(&self) -> sRgba<u8> {
		sRgba {
			red: (self.red * 255.0) as u8,
			green: (self.green * 255.0) as u8,
			blue: (self.blue * 255.0) as u8,
			alpha: (self.alpha * 255.0) as u8
		}
	}

	pub fn to_linear(&self) -> LinearRgba {
		LinearRgba::new(
			if self.red() <= 0.04045 { self.red() / 12.92 } else { ( ( self.red() + 0.055 ) / 1.055 ).powf(2.4) },
			if self.green() <= 0.04045 { self.green() / 12.92 } else { ( ( self.green() + 0.055 ) / 1.055 ).powf(2.4) },
			if self.blue() <= 0.04045 { self.blue() / 12.92 } else { ( ( self.blue() + 0.055 ) / 1.055 ).powf(2.4) },
			self.alpha()
		)
	}

	pub fn to_oklaba(&self) -> Oklaba {
		self.to_linear().to_oklaba()
	}

	pub fn to_oklcha(&self) -> Oklcha {
		self.to_oklaba().to_oklcha()
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
		let w = self.red.min(self.green).min(self.blue);
		let v = self.red.max(self.green).max(self.blue);
		let b = 1.0 - v;

		if v == w {
			return Hwba::new(
				0.0,
				w,
				b,
				self.alpha()
			)
		}

		let f = if self.red == v {
			(self.green - self.blue) / (v - w)
		} else if self.green == v {
			(self.blue - self.red) / (v - w)
		} else {
			(self.red - self.green) / (v - w)
		};

		let h = if self.red == v {
			(f / 6.0) % 1.0
		} else if self.green == v {
			(f + 2.0) / 6.0
		} else {
			(f + 4.0) / 6.0
		};

		let mut h = if h < 0.0 { h + 1.0 } else { h };
		h *= 360.0;

		Hwba::new(
			h,
			w,
			b,
			self.alpha() as f32 / 255.0
		)
	}

	pub fn to_hsva(&self) -> Hsva {
		self.to_hwba().to_hsva()
	}

	pub fn to_hsla(&self) -> Hsla {
		self.to_hsva().to_hsla()
	}

	pub fn to_vec(&self) -> Vec4 {
		Vec4::new(
			self.red,
			self.green,
			self.blue,
			self.alpha
		)
	}
}