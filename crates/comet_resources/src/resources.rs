use std::{
	collections::HashMap, path::Path
};

use wgpu::{Device, FilterMode, Queue, TextureFormat, TextureUsages};
use crate::{texture, Texture};
use crate::texture_atlas::{TextureAtlas, TextureRegion};

pub struct ResourceManager {
	texture_atlas: TextureAtlas,
	data_files: HashMap<String, String>
}

impl ResourceManager {
	pub fn new() -> Self {
		Self {
			texture_atlas: TextureAtlas::empty(),
			data_files: HashMap::new()
		}
	}

	pub fn texture_atlas(&self) -> &TextureAtlas {
		&self.texture_atlas
	}

	pub fn texture_locations(&self) -> &HashMap<String, TextureRegion> {
		&self.texture_atlas.textures()
	}

	pub fn data_files(&self) -> &HashMap<String, String> {
		&self.data_files
	}

	pub fn set_texture_atlas(&mut self, texture_atlas: TextureAtlas) {
		self.texture_atlas = texture_atlas;

		// This is just for testing purposes
		//self.texture_locations.insert("normal_comet.png".to_string(), ([0,0], [15,15]));
		//self.texture_locations.insert("green_comet.png".to_string(), ([0,15], [15,31]));
	}

	pub fn create_texture_atlas(&mut self, paths: Vec<String>) {
		self.texture_atlas = TextureAtlas::from_texture_paths(paths)
	}

	pub async fn load_string(&self, file_name: &str) -> anyhow::Result<String> {
		let path = Path::new(std::env::var("OUT_DIR")?.as_str())
			.join("res")
			.join(file_name);
		let txt = std::fs::read_to_string(path)?;

		Ok(txt)
	}

	pub async fn load_binary(&self, file_name: &str) -> anyhow::Result<Vec<u8>> {
		let path = Path::new(std::env::var("OUT_DIR").unwrap().as_str())
			.join("res")
			.join(file_name);
		let data = std::fs::read(path)?;

		Ok(data)
	}

	pub async fn load_texture(
		&self,
		file_name: &str,
		is_normal_map: bool,
		device: &wgpu::Device,
		queue: &wgpu::Queue,
	) -> anyhow::Result<texture::Texture> {
		let data = self.load_binary(file_name).await?;
		texture::Texture::from_bytes(device, queue, &data, file_name, is_normal_map)
	}

	/*pub async fn load_model(
		&self,
		file_name: &str,
		device: &wgpu::Device,
		queue: &wgpu::Queue,
		layout: &wgpu::BindGroupLayout,
	) -> anyhow::Result<model::Model> {
		let obj_text = self.load_string(file_name).await?;
		let obj_cursor = Cursor::new(obj_text);
		let mut obj_reader = BufReader::new(obj_cursor);

		let (models, obj_materials) = tobj::load_obj_buf_async(
			&mut obj_reader,
			&tobj::LoadOptions {
				triangulate: true,
				single_index: true,
				..Default::default()
			},
			|p| async move {
				let mat_text = self.load_string(&p).await.unwrap();
				tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
			},
		)
			.await?;

		let mut materials = Vec::new();
		for m in obj_materials? {
			let diffuse_texture = self.load_texture(&m.diffuse_texture, false, device, queue).await?;
			let normal_texture = self.load_texture(&m.normal_texture, true, device, queue).await?;
			let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
				layout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
					},
				],
				label: None,
			});

			materials.push(model::Material {
				name: m.name,
				diffuse_texture,
				bind_group,
			});
		}

		let meshes = models
			.into_iter()
			.map(|m| {
				let vertices = (0..m.mesh.positions.len() / 3)
					.map(|i| {
						if m.mesh.normals.is_empty() {
							model::ModelVertex {
								position: [
									m.mesh.positions[i * 3],
									m.mesh.positions[i * 3 + 1],
									m.mesh.positions[i * 3 + 2],
								],
								tex_coords: [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
								normal: [0.0, 0.0, 0.0],
							}
						} else {
							model::ModelVertex {
								position: [
									m.mesh.positions[i * 3],
									m.mesh.positions[i * 3 + 1],
									m.mesh.positions[i * 3 + 2],
								],
								tex_coords: [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
								normal: [
									m.mesh.normals[i * 3],
									m.mesh.normals[i * 3 + 1],
									m.mesh.normals[i * 3 + 2],
								],
							}
						}
					})
					.collect::<Vec<_>>();

				let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
					label: Some(&format!("{:?} Vertex Buffer", file_name)),
					contents: bytemuck::cast_slice(&vertices),
					usage: wgpu::BufferUsages::VERTEX,
				});
				let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
					label: Some(&format!("{:?} Index Buffer", file_name)),
					contents: bytemuck::cast_slice(&m.mesh.indices),
					usage: wgpu::BufferUsages::INDEX,
				});

				model::Mesh {
					name: file_name.to_string(),
					vertex_buffer,
					index_buffer,
					num_elements: m.mesh.indices.len() as u32,
					material: m.mesh.material_id.unwrap_or(0),
				}
			})
			.collect::<Vec<_>>();

		Ok(model::Model { meshes, materials })
	}*/
}
