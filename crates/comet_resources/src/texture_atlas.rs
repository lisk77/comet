use std::collections::HashMap;
use std::path::Path;
use chrono::Local;
use std::time::Instant;
use image::{DynamicImage, GenericImage, GenericImageView, ImageFormat};
use comet_log::*;
use wgpu::{Device, FilterMode, TextureFormat, TextureUsages};
use crate::Texture;

#[derive(Debug)]
pub struct TextureRegion {
	x0: f32,
	y0: f32,
	x1: f32,
	y1: f32,
	dimensions: (u32, u32)
}

impl TextureRegion {
	pub fn new(x0: f32, y0: f32, x1: f32, y1: f32, dimensions: (u32, u32)) -> Self {
		Self {
			x0,
			y0,
			x1,
			y1,
			dimensions
		}
	}

	pub fn x0(&self) -> f32 {
		self.x0
	}

	pub fn x1(&self) -> f32 {
		self.x1
	}

	pub fn y0(&self) -> f32 {
		self.y0
	}

	pub fn y1(&self) -> f32 {
		self.y1
	}

	pub fn dimensions(&self) -> (u32, u32) {
		self.dimensions
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
		//let t0 = Instant::now();

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
			//base.copy_from(texture, x_offset, y_offset).expect("Nope, you propably failed the offets");
			Self::insert_texture_at(&mut base, &texture, x_offset, y_offset);
			regions.insert(path.to_string(), TextureRegion::new(
				x_offset as f32 / width as f32,
				y_offset as f32 / height as f32,
				(x_offset + texture.width()) as f32 / width as f32,
				(y_offset + texture.height()) as f32 / height as f32,
				texture.dimensions()
			));
			x_offset += texture.width();
		}

		// Save the image to disk as a PNG
		//let output_path = Path::new(r"C:\Users\lisk77\Code Sharing\comet-engine\resources\textures\atlas.png");
		//base.save_with_format(output_path, ImageFormat::Png).expect("Failed to save texture atlas");

		info!("Texture atlas created!");
		debug!(format!("{:?}", regions));

		/*let t1 = Instant::now();
		let delta = t1.duration_since(t0);
		println!("{:?}", delta);*/

		TextureAtlas {
			atlas: base,
			textures: regions
		}
	}

	pub fn atlas(&self) -> &DynamicImage {
		&self.atlas
	}

	pub fn textures(&self) -> &HashMap<String, TextureRegion> {
		&self.textures
	}
}