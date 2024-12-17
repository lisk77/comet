use rand::Rng;

/// The WhiteNoise struct works a factory for generating white noise, given the size of the texture.
pub struct WhiteNoise {
	size: (usize, usize)
}

impl WhiteNoise {
	pub fn new(width: usize, height: usize) -> Self {
		Self {
			size: (width, height)
		}
	}

	pub fn set_width(&mut self, width: usize) {
		self.size.0 = width;
	}

	pub fn set_height(&mut self, height: usize) {
		self.size.1 = height;
	}

	pub fn set_size(&mut self, width: usize, height: usize) {
		self.size = (width, height);
	}

	/// Generates white noise as a `Vec<f32>`. Size of the vector is `width * height`.
	pub fn generate(&self) -> Vec<f32> {
		let mut rng = rand::rng();
		let mut noise = Vec::with_capacity(self.size.0 * self.size.1);

		for _ in 0..self.size.0 * self.size.1 {
			noise.push(rng.random_range(0.0..1.0));
		}

		noise
	}
}

