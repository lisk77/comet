use image::{DynamicImage, GenericImage, GenericImageView, Rgba};
use rand::Rng;
use comet_log::debug;
use crate::v2;
use crate::lerp;

// TODO
// Make noise struct keep their generated noise
// Create noise trait as a common interface for all noise types
// Use noise trait to let the generated noise be outputed in different ways like images or Vec<f32>

pub trait NoiseGenerator {
	fn generate(&self) -> Vec<f32>;
	fn generate_image(&self) -> DynamicImage;
}

pub struct WhiteNoise {
	size: (usize, usize),
}

impl WhiteNoise {
	/// Creates a white noise generator ideal for multiple uses.
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
	pub fn generate(size: (usize, usize)) -> Vec<f32> {
		let mut rng = rand::rng();
		let mut noise = Vec::with_capacity(size.0 * size.1);

		for _ in 0..size.0 * size.1 {
			noise.push(rng.random_range(0.0..1.0));
		}

		noise
	}

	/// Generates white noise as a `DynamicImage`.
	pub fn generate_image(size: (usize, usize)) -> DynamicImage {
		let mut rng = rand::rng();
		let mut image = DynamicImage::new_rgb8(size.0 as u32, size.1 as u32);

		for y in 0..size.1 {
			for x in 0..size.0 {
				let value = (rng.random_range(0.0..1.0) * 255.0) as u8;
				image.put_pixel(x as u32, y as u32, Rgba([value, value, value, 255]));
			}
		}

		image
	}
}

impl NoiseGenerator for WhiteNoise {
	/// Generates white noise as a `Vec<f32>`. Size of the vector is `width * height`.
	fn generate(&self) -> Vec<f32> {
		let mut rng = rand::rng();
		let mut noise = Vec::with_capacity(self.size.0 * self.size.1);

		for _ in 0..self.size.0 * self.size.1 {
			noise.push(rng.random_range(0.0..1.0));
		}

		noise
	}

	/// Generates white noise as a `DynamicImage`.
	fn generate_image(&self) -> DynamicImage {
		let mut rng = rand::rng();
		let mut image = DynamicImage::new_rgb8(self.size.0 as u32, self.size.1 as u32);

		for y in 0..self.size.1 {
			for x in 0..self.size.0 {
				let value = (rng.random_range(0.0..1.0) * 255.0) as u8;
				image.put_pixel(x as u32, y as u32, Rgba([value, value, value, 255]));
			}
		}

		image
	}
}

pub struct PerlinNoise {
	size: (usize, usize),
	frequency: f64,
	seed: u32,
}

impl PerlinNoise {
	pub fn new(width: usize, height: usize, frequency: f64, seed: u32) -> Self {
		Self {
			size: (width, height),
			frequency,
			seed,
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

	pub fn set_frequency(&mut self, frequency: f64) {
		self.frequency = frequency;
	}

	pub fn set_seed(&mut self, seed: u32) {
		self.seed = seed;
	}

	/// Generates Perlin noise as a `Vec<f32>`. Size of the vector is `width * height`.
	pub fn generate(&self) -> Vec<f32> {
		let mut noise = Vec::with_capacity(self.size.0 * self.size.1);

		for y in 0..self.size.1 {
			for x in 0..self.size.0 {
				let nx = x as f64 / self.size.0 as f64;
				let ny = y as f64 / self.size.1 as f64;
				let value = self.perlin(nx * self.frequency, ny * self.frequency);
				noise.push((value+1.0) * 0.5);
			}
		}

		noise
	}

	/// Generates Perlin noise with multiple octaves as a `Vec<f32>`.
	pub fn generate_with_octaves(&self, octaves: u32, persistence: f64) -> Vec<f32> {
		let mut noise = vec![0.0; self.size.0 * self.size.1];
		let mut amplitude = 1.0;
		let mut frequency = self.frequency;
		let mut max_value = 0.0; // Used for normalization

		for _ in 0..octaves {
			for y in 0..self.size.1 {
				for x in 0..self.size.0 {
					let nx = x as f64 / self.size.0 as f64;
					let ny = y as f64 / self.size.1 as f64;
					noise[y * self.size.0 + x] += self.perlin(nx * frequency, ny * frequency) as f32 * amplitude as f32;
				}
			}
			max_value += amplitude;
			amplitude *= persistence; // Reduce amplitude for next octave
			frequency *= 2.0;         // Double frequency for next octave
		}

		// Normalize the noise to the range [0, 1]
		noise.iter_mut().for_each(|value| *value /= max_value as f32);

		noise.iter_mut().for_each(|value| *value = (*value + 1.0) * 0.5);

		noise
	}


	/// A raw Perlin noise function implementation.
	fn perlin(&self, x: f64, y: f64) -> f32 {
		let xi = x.floor() as i32 & 255;
		let yi = y.floor() as i32 & 255;

		let xf = x - x.floor();
		let yf = y - y.floor();

		let u = Self::fade(xf);
		let v = Self::fade(yf);

		let a = self.permutation(xi) + yi;
		let b = self.permutation(xi + 1) + yi;

		let aa = self.permutation(a);
		let ab = self.permutation(a + 1);
		let ba = self.permutation(b);
		let bb = self.permutation(b + 1);

		let x1 = lerp(u as f32, Self::grad(self.permutation(aa), xf, yf) as f32, Self::grad(self.permutation(ba), xf - 1.0, yf) as f32);
		let x2 = lerp(u as f32, Self::grad(self.permutation(ab), xf, yf - 1.0) as f32, Self::grad(self.permutation(bb), xf - 1.0, yf - 1.0) as f32);

		lerp(v as f32, x1, x2)
	}

	fn fade(t: f64) -> f64 {
		t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
	}

	fn lerp(t: f64, a: f64, b: f64) -> f64 {
		a + t * (b - a)
	}

	fn grad(hash: i32, x: f64, y: f64) -> f64 {
		let h = hash & 3;
		let u = if h & 2 == 0 { x } else { -x };
		let v = if h & 1 == 0 { y } else { -y };
		u + v
	}

	fn permutation(&self, value: i32) -> i32 {
		const P: [i32; 256] = [
			151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30, 69, 142, 8, 99, 37, 240,
			21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219, 203, 117, 35, 11, 32, 57, 177, 33, 88,
			237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68, 175, 74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231,
			83, 111, 229, 122, 60, 211, 133, 230, 220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161,
			1, 216, 80, 73, 209, 76, 132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109,
			198, 173, 186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212, 207, 206,
			59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213, 119, 248, 152, 2, 44, 154, 163, 70, 221, 153,
			101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232, 178, 185, 112, 104, 218,
			246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162, 241, 81, 51, 145, 235, 249, 14, 239, 107,
			49, 192, 214, 31, 181, 199, 106, 157, 184, 84, 204, 176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205,
			93, 222, 114, 67, 29, 24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180
		];

		P[((value ^ self.seed as i32) & 255) as usize]
	}
}

pub struct ValueNoise {
	size: (usize, usize),
	frequency: f64,
	seed: u32,
}

impl ValueNoise {
	pub fn new(width: usize, height: usize, frequency: f64, seed: u32) -> Self {
		Self {
			size: (width, height),
			frequency,
			seed,
		}
	}

	fn permutation(&self, value: i32) -> i32 {
		const P: [i32; 256] = [
			151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30, 69, 142, 8, 99, 37, 240,
			21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219, 203, 117, 35, 11, 32, 57, 177, 33, 88,
			237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68, 175, 74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231,
			83, 111, 229, 122, 60, 211, 133, 230, 220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161,
			1, 216, 80, 73, 209, 76, 132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109,
			198, 173, 186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212, 207, 206,
			59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213, 119, 248, 152, 2, 44, 154, 163, 70, 221, 153,
			101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232, 178, 185, 112, 104, 218,
			246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162, 241, 81, 51, 145, 235, 249, 14, 239, 107,
			49, 192, 214, 31, 181, 199, 106, 157, 184, 84, 204, 176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205,
			93, 222, 114, 67, 29, 24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180
		];

		P[((value ^ self.seed as i32) & 255) as usize]
	}

	fn noise(&self, p: (f32, f32)) -> f32 {
		let i = (p.0.floor() as i32, p.1.floor() as i32);
		let f = (p.0.fract(), p.1.fract());

		// cubic interpolant
		let u = (
			f.0 * f.0 * (3.0 - 2.0 * f.0),
			f.1 * f.1 * (3.0 - 2.0 * f.1)
		);

		let a = self.permutation(i.0) + i.1;
		let b = self.permutation(i.0 + 1) + i.1;

		lerp(
			lerp(
				self.permutation(a) as f32 / 255.0 * 2.0 - 1.0,
				self.permutation(b) as f32 / 255.0 * 2.0 - 1.0,
				u.0
			),
			lerp(
				self.permutation(a + 1) as f32 / 255.0 * 2.0 - 1.0,
				self.permutation(b + 1) as f32 / 255.0 * 2.0 - 1.0,
				u.0
			),
			u.1
		)
	}

	pub fn generate(&self) -> Vec<f32> {
		let mut noise = Vec::with_capacity(self.size.0 * self.size.1);
		let mut max_amplitude = 0.0;
		let mut amplitude = 0.5;

		// Calculate max amplitude for normalization
		for _ in 0..4 {
			max_amplitude += amplitude;
			amplitude *= 0.5;
		}

		for y in 0..self.size.1 {
			for x in 0..self.size.0 {
				let mut uv = (
					x as f32 / self.size.0 as f32 * self.frequency as f32,
					y as f32 / self.size.1 as f32 * self.frequency as f32,
				);

				let mut f = 0.0;
				let mut amplitude = 0.5;

				/*for _ in 0..4 {  // 4 octaves*/
					f += amplitude * self.noise(uv);

					// Double frequency for next octave
					uv = (uv.0 * 2.0, uv.1 * 2.0);

					// Reduce amplitude (persistence)
					amplitude *= 0.5;
				/*}*/

				// Normalize and convert to [0, 1]
				f = ((f / max_amplitude) + 1.0) * 0.5;

				noise.push(f);
			}
		}

		noise
	}


	pub fn generate_with_octaves(&self, octaves: u32, persistence: f64) -> Vec<f32> {
		let mut noise = Vec::with_capacity(self.size.0 * self.size.1);
		let mut max_amplitude = 0.0;
		let mut amplitude = 1.0;

		// Calculate max amplitude for normalization
		for _ in 0..octaves {
			max_amplitude += amplitude;
			amplitude *= persistence;
		}

		for y in 0..self.size.1 {
			for x in 0..self.size.0 {
				// Convert to UV space and scale by frequency
				let mut uv = (
					x as f32 / self.size.0 as f32 * self.frequency as f32,
					y as f32 / self.size.1 as f32 * self.frequency as f32,
				);

				let mut f = 0.0;
				let mut amplitude = 1.0;

				for _ in 0..octaves {
					f += amplitude * self.noise(uv);

					// Double frequency for next octave
					uv = (uv.0 * 2.0, uv.1 * 2.0);

					// Reduce amplitude (persistence)
					amplitude *= persistence as f32;
				}

				// Normalize and convert to [0, 1]
				f = ((f / max_amplitude as f32) + 1.0) * 0.5;

				noise.push(f);
			}
		}

		noise
	}
}