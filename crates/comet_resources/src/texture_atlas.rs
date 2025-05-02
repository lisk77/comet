use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
use image::{DynamicImage, GenericImage, GenericImageView, ImageFormat};
use comet_log::*;
use wgpu::{Device, FilterMode, TextureFormat, TextureUsages};
use crate::font::GlyphData;
use crate::Texture;

#[derive(Debug)]
pub struct TextureRegion {
	u0: f32,
	v0: f32,
	u1: f32,
	v1: f32,
	advance: f32,
	offset_x: f32,
	offset_y: f32,
	dimensions: (u32, u32),
}

impl TextureRegion {
	pub fn new(u0: f32, v0: f32, u1: f32, v1: f32, dimensions: (u32, u32), advance: f32, offset_x: f32, offset_y: f32) -> Self {
		Self {
			u0,
			v0,
			u1,
			v1,
			advance,
			offset_x,
			offset_y,
			dimensions
		}
	}

	pub fn u0(&self) -> f32 {
		self.u0
	}

	pub fn u1(&self) -> f32 {
		self.u1
	}

	pub fn v0(&self) -> f32 {
		self.v0
	}

	pub fn v1(&self) -> f32 {
		self.v1
	}

	pub fn dimensions(&self) -> (u32, u32) {
		self.dimensions
	}

	pub fn advance(&self) -> f32 {
		self.advance
	}

	pub fn offset_x(&self) -> f32 {
		self.offset_x
	}

	pub fn offset_y(&self) -> f32 {
		self.offset_y
	}
}

#[derive(Debug)]
pub struct TextureAtlas {
	atlas: DynamicImage,
	textures: HashMap<String, TextureRegion>,
}

impl TextureAtlas {
	pub fn empty() -> Self {
		Self {
			atlas: DynamicImage::new(1,1, image::ColorType::Rgb8),
			textures: HashMap::new()
		}
	}

	pub fn texture_paths(&self) -> Vec<String> {
		self.textures.keys().map(|k| k.to_string()).collect()
	}

	fn calculate_atlas_width(textures: &Vec<DynamicImage>) -> u32 {
		let mut last_height: u32 = textures.get(0).unwrap().height();
		let mut widths: Vec<u32> = Vec::new();
		let mut current_width: u32 = 0;

		for texture in textures {
			if last_height != texture.height() {
				widths.push(current_width);
				current_width = 0;
				last_height = texture.height();
			}
			current_width += texture.width();
		}

		widths.push(current_width);

		*widths.iter().max().unwrap()
	}

	fn calculate_atlas_height(textures: &Vec<DynamicImage>) -> u32 {
		let last_height: u32 = textures.get(0).unwrap().height();
		let mut height: u32 = 0;
		height += last_height;

		for texture in textures {
			if last_height == texture.height() {
				continue;
			}

			height += texture.height();
		}

		height
	}

	fn insert_texture_at(base: &mut DynamicImage, texture: &DynamicImage, x_pos: u32, y_pos: u32) {
		for y in 0..texture.height() {
			for x in 0..texture.width() {
				let pixel = texture.get_pixel(x,y);
				base.put_pixel(x + x_pos, y + y_pos, pixel);
			}
		}
	}

	pub fn from_texture_paths(
		paths: Vec<String>,
	) -> Self {
		let mut textures: Vec<DynamicImage> = Vec::new();
		let mut regions: HashMap<String, TextureRegion> = HashMap::new();

		info!("Loading textures...");

		for path in &paths {
			textures.push(image::open(&Path::new(path.as_str())).expect("Failed to load texture"));
		}

		info!("Textures loaded!");
		info!("Sorting textures by height...");

		let mut texture_path_pairs: Vec<(&DynamicImage, &String)> = textures.iter().zip(paths.iter()).collect();
		texture_path_pairs.sort_by(|a, b| b.0.height().cmp(&a.0.height()));
		let (sorted_textures, sorted_paths): (Vec<&DynamicImage>, Vec<&String>) = texture_path_pairs.into_iter().unzip();
		let sorted_textures: Vec<DynamicImage> = sorted_textures.into_iter().map(|t| t.clone()).collect();
		let sorted_paths: Vec<String> = sorted_paths.into_iter().map(|s| s.to_string()).collect();

		let (height, width) = (Self::calculate_atlas_height(&sorted_textures), Self::calculate_atlas_width(&sorted_textures));
		let mut base = DynamicImage::new_rgba8(width,height);

		let mut previous = sorted_textures.get(0).unwrap().height();
		let mut x_offset: u32 = 0;
		let mut y_offset: u32 = 0;

		info!("Creating texture atlas...");

		for (texture, path) in sorted_textures.iter().zip(sorted_paths.iter()) {
			if texture.height() != previous {
				y_offset += previous;
				x_offset = 0;
				previous = texture.height();
			}

			Self::insert_texture_at(&mut base, &texture, x_offset, y_offset);
			regions.insert(path.to_string(), TextureRegion::new(
				x_offset as f32 / width as f32,
				y_offset as f32 / height as f32,
				(x_offset + texture.width()) as f32 / width as f32,
				(y_offset + texture.height()) as f32 / height as f32,
				texture.dimensions(),
				0.0,
				0.0,
				0.0
			));
			x_offset += texture.width();
		}

		info!("Texture atlas created!");

		TextureAtlas {
			atlas: base,
			textures: regions
		}
	}

	pub fn from_textures(
		names: Vec<String>,
		textures: Vec<DynamicImage>,
	) -> Self {
		let mut regions: HashMap<String, TextureRegion> = HashMap::new();

		info!("Sorting textures by height...");

		let mut texture_path_pairs: Vec<(&DynamicImage, &String)> = textures.iter().zip(names.iter()).collect();
		texture_path_pairs.sort_by(|a, b| b.0.height().cmp(&a.0.height()));
		let (sorted_textures, sorted_paths): (Vec<&DynamicImage>, Vec<&String>) = texture_path_pairs.into_iter().unzip();
		let sorted_textures: Vec<DynamicImage> = sorted_textures.into_iter().map(|t| t.clone()).collect();
		let sorted_paths: Vec<String> = sorted_paths.into_iter().map(|s| s.to_string()).collect();

		let (height, width) = (Self::calculate_atlas_height(&sorted_textures), Self::calculate_atlas_width(&sorted_textures));
		let mut base = DynamicImage::new_rgba8(width,height);

		let mut previous = sorted_textures.get(0).unwrap().height();
		let mut x_offset: u32 = 0;
		let mut y_offset: u32 = 0;

		info!("Creating texture atlas...");

		for (texture, name) in sorted_textures.iter().zip(sorted_paths.iter()) {
			if texture.height() != previous {
				y_offset += previous;
				x_offset = 0;
				previous = texture.height();
			}

			Self::insert_texture_at(&mut base, &texture, x_offset, y_offset);
			regions.insert(name.to_string(), TextureRegion::new(
				x_offset as f32 / width as f32,
				y_offset as f32 / height as f32,
				(x_offset + texture.width()) as f32 / width as f32,
				(y_offset + texture.height()) as f32 / height as f32,
				texture.dimensions(),
				0.0,
				0.0,
				0.0
			));
			x_offset += texture.width();
		}

		info!("Texture atlas created!");

		TextureAtlas {
			atlas: base,
			textures: regions
		}
	}

	pub fn from_glyphs(mut glyphs: Vec<GlyphData>) -> Self {
		glyphs.sort_by(|a, b| b.render.height().cmp(&a.render.height()));

		let height = Self::calculate_atlas_height(
			&glyphs.iter().map(|g| g.render.clone()).collect::<Vec<_>>()
		);
		let width = Self::calculate_atlas_width(
			&glyphs.iter().map(|g| g.render.clone()).collect::<Vec<_>>()
		);

		let padding = (glyphs.len() * 3) as u32;

		let mut base = DynamicImage::new_rgba8(width + padding, height);
		let mut regions = HashMap::new();
		let mut current_row_height = glyphs[0].render.height();
		let mut x_offset: u32 = 0;
		let mut y_offset: u32 = 0;


		for g in glyphs.iter() {
			let glyph_w = g.render.width();
			let glyph_h = g.render.height();

			if glyph_h != current_row_height {
				y_offset += current_row_height + 3;
				x_offset = 0;
				current_row_height = glyph_h;
			}

			Self::insert_texture_at(&mut base, &g.render, x_offset, y_offset);

			let u0 = x_offset as f32 / (width + padding) as f32;
			let v0 = y_offset as f32 / height as f32;
			let u1 = (x_offset + glyph_w) as f32 / (width + padding) as f32;
			let v1 = (y_offset + glyph_h) as f32 / height as f32;

			let region = TextureRegion::new(
				u0, v0, u1, v1,
				(glyph_w, glyph_h),
				g.advance,
				g.offset_x,
				g.offset_y,
			);

			regions.insert(g.name.clone(), region);

			x_offset += glyph_w + 3;
		}

		TextureAtlas {
			atlas: base,
			textures: regions,
		}
	}

	pub fn atlas(&self) -> &DynamicImage {
		&self.atlas
	}

	pub fn textures(&self) -> &HashMap<String, TextureRegion> {
		&self.textures
	}
}