use image::{DynamicImage, Rgba, RgbaImage};
use ab_glyph::{FontArc, PxScale, ScaleFont, Glyph, point, Font as AbFont};
use crate::texture_atlas::{TextureAtlas, TextureRegion};

pub struct Font {
	name: String,
	glyphs: TextureAtlas,
}

impl Font {
	pub fn new(path: &str, size: f32) -> Self {
		Font {
			name: path.to_string(),
			glyphs: Self::generate_atlas(path, size)
		}
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn glyphs(&self) -> &TextureAtlas {
		&self.glyphs
	}

	pub fn get_glyph(&self, ch: char) -> Option<&TextureRegion> {
		self.glyphs.textures().get(&ch.to_string())
	}

	fn generate_atlas(path: &str, size: f32) -> TextureAtlas {
		let font_data = std::fs::read(path).expect("Failed to read font file");
		let font = FontArc::try_from_vec(font_data).expect("Failed to load font");

		let scale = PxScale::from(size);
		let scaled_font = font.as_scaled(scale);

		let mut names = Vec::new();
		let mut images = Vec::new();

		for code_point in 0x0020..=0x007E {
			if let Some(ch) = std::char::from_u32(code_point) {
				if font.glyph_id(ch).0 == 0 {
					continue;
				}

				names.push(ch.to_string());

				let glyph = Glyph {
					id: font.glyph_id(ch),
					scale,
					position: point(0.0, 0.0),
				};

				if let Some(outline) = scaled_font.outline_glyph(glyph) {
					let bounds = outline.px_bounds();
					let width = bounds.width().ceil() as u32;
					let height = bounds.height().ceil() as u32;

					if width == 0 || height == 0 {
						continue;
					}

					let mut image = RgbaImage::new(width, height);
					for pixel in image.pixels_mut() {
						*pixel = Rgba([0, 0, 0, 0]);
					}

					outline.draw(|x, y, v| {
						let alpha = (v * 255.0) as u8;
						image.put_pixel(x, y, Rgba([255, 255, 255, alpha]));
					});

					images.push(DynamicImage::ImageRgba8(image));
				}
			}
		}

		TextureAtlas::from_textures(names, images)
	}
}