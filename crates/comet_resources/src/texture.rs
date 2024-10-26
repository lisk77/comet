use anyhow::*;
use image::{DynamicImage, GenericImageView, RgbaImage};
use wgpu::{Device, Queue};

#[derive(Debug)]
pub struct Texture {
	#[allow(unused)]
	pub texture: wgpu::Texture,
	pub view: wgpu::TextureView,
	pub sampler: wgpu::Sampler,
	pub size: wgpu::Extent3d,
}

impl Texture {
	pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

	pub fn create_depth_texture(
		device: &wgpu::Device,
		config: &wgpu::SurfaceConfiguration,
		label: &str,
	) -> Self {
		let size = wgpu::Extent3d {
			width: config.width.max(1),
			height: config.height.max(1),
			depth_or_array_layers: 1,
		};
		let desc = wgpu::TextureDescriptor {
			label: Some(label),
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: Self::DEPTH_FORMAT,
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
			view_formats: &[Self::DEPTH_FORMAT],
		};
		let texture = device.create_texture(&desc);
		let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
		let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Linear,
			mipmap_filter: wgpu::FilterMode::Nearest,
			compare: Some(wgpu::CompareFunction::LessEqual),
			lod_min_clamp: 0.0,
			lod_max_clamp: 100.0,
			..Default::default()
		});

		Self {
			texture,
			view,
			sampler,
			size, // NEW!
		}
	}

	#[allow(dead_code)]
	pub fn from_bytes(
		device: &wgpu::Device,
		queue: &wgpu::Queue,
		bytes: &[u8],
		label: &str,
		is_normal_map: bool,
	) -> Result<Self> {
		let img = image::load_from_memory(bytes)?;
		Self::from_image(device, queue, &img, Some(label), is_normal_map)
	}

	pub fn from_image(
		device: &wgpu::Device,
		queue: &wgpu::Queue,
		img: &image::DynamicImage,
		label: Option<&str>,
		is_normal_map: bool,
	) -> Result<Self> {
		let dimensions = img.dimensions();
		let rgba = img.to_rgba8();

		let format = if is_normal_map {
			wgpu::TextureFormat::Rgba8Unorm
		} else {
			wgpu::TextureFormat::Rgba8UnormSrgb
		};
		let usage = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
		let size = wgpu::Extent3d {
			width: img.width(),
			height: img.height(),
			depth_or_array_layers: 1,
		};
		let texture = Self::create_2d_texture(
			device,
			size.width,
			size.height,
			format,
			usage,
			wgpu::FilterMode::Nearest,
			label,
		);

		queue.write_texture(
			wgpu::ImageCopyTexture {
				aspect: wgpu::TextureAspect::All,
				texture: &texture.texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
			},
			&rgba,
			wgpu::ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(4 * dimensions.0),
				rows_per_image: Some(dimensions.1),
			},
			size,
		);

		Ok(texture)
	}

	pub(crate) fn create_2d_texture(
		device: &wgpu::Device,
		width: u32,
		height: u32,
		format: wgpu::TextureFormat,
		usage: wgpu::TextureUsages,
		mag_filter: wgpu::FilterMode,
		label: Option<&str>,
	) -> Self {
		let size = wgpu::Extent3d {
			width,
			height,
			depth_or_array_layers: 1,
		};
		Self::create_texture(
			device,
			label,
			size,
			format,
			usage,
			wgpu::TextureDimension::D2,
			mag_filter,
		)
	}

	pub fn create_texture(
		device: &wgpu::Device,
		label: Option<&str>,
		size: wgpu::Extent3d,
		format: wgpu::TextureFormat,
		usage: wgpu::TextureUsages,
		dimension: wgpu::TextureDimension,
		mag_filter: wgpu::FilterMode,
	) -> Self {
		let texture = device.create_texture(&wgpu::TextureDescriptor {
			label,
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension,
			format,
			usage,
			view_formats: &[],
		});

		let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
		let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter,
			min_filter: wgpu::FilterMode::Nearest,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		});

		Self {
			texture,
			view,
			sampler,
			size, // NEW!
		}
	}

	pub fn to_image(
		&self,
		device: &wgpu::Device,
		queue: &wgpu::Queue,
	) -> Result<DynamicImage> {
		// Size of the texture
		let width = self.size.width;
		let height = self.size.height;

		// Calculate the size of the texture in bytes
		let texture_size_bytes = (4 * width * height) as wgpu::BufferAddress;

		// Create a buffer for reading the texture data back from the GPU
		let buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Texture Readback Buffer"),
			size: texture_size_bytes,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
			mapped_at_creation: false,
		});

		// Create a command encoder to copy the texture data to the buffer
		let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("Texture to Buffer Encoder"),
		});

		// Define the copy operation from the texture to the buffer
		encoder.copy_texture_to_buffer(
			wgpu::ImageCopyTexture {
				texture: &self.texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			wgpu::ImageCopyBuffer {
				buffer: &buffer,
				layout: wgpu::ImageDataLayout {
					offset: 0,
					bytes_per_row: Some(4 * width),
					rows_per_image: Some(height),
				},
			},
			self.size,
		);

		// Submit the command to the queue
		queue.submit(Some(encoder.finish()));

		// Wait for the GPU to finish the operation
		let buffer_slice = buffer.slice(..);
		buffer_slice.map_async(wgpu::MapMode::Read, |result| {
			if let Err(e) = result {
				eprintln!("Failed to map buffer: {:?}", e);
			}
		});

		// Get the buffer data
		let data = buffer_slice.get_mapped_range();

		// Convert the raw data into an image::RgbaImage
		let image = RgbaImage::from_raw(width, height, data.to_vec())
			.ok_or_else(|| anyhow!("Failed to create image from raw texture data"))?;

		// Unmap the buffer now that we're done with it
		buffer.unmap();

		// Convert the RgbaImage into a DynamicImage
		Ok(DynamicImage::ImageRgba8(image))
	}
}

pub struct CubeTexture {
	texture: wgpu::Texture,
	sampler: wgpu::Sampler,
	view: wgpu::TextureView,
}

impl CubeTexture {
	pub fn create_2d(
		device: &wgpu::Device,
		width: u32,
		height: u32,
		format: wgpu::TextureFormat,
		mip_level_count: u32,
		usage: wgpu::TextureUsages,
		mag_filter: wgpu::FilterMode,
		label: Option<&str>,
	) -> Self {
		let texture = device.create_texture(&wgpu::TextureDescriptor {
			label,
			size: wgpu::Extent3d {
				width,
				height,
				// A cube has 6 sides, so we need 6 layers
				depth_or_array_layers: 6,
			},
			mip_level_count,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format,
			usage,
			view_formats: &[],
		});

		let view = texture.create_view(&wgpu::TextureViewDescriptor {
			label,
			dimension: Some(wgpu::TextureViewDimension::Cube),
			array_layer_count: Some(6),
			..Default::default()
		});

		let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			label,
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter,
			min_filter: wgpu::FilterMode::Nearest,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		});

		Self {
			texture,
			sampler,
			view,
		}
	}

	pub fn texture(&self) -> &wgpu::Texture {
		&self.texture
	}

	pub fn view(&self) -> &wgpu::TextureView {
		&self.view
	}

	pub fn sampler(&self) -> &wgpu::Sampler {
		&self.sampler
	}
}