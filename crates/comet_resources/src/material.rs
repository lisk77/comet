use crate::texture;

pub struct Material {
	pub name: String,
	pub diffuse_texture: texture::Texture,
	pub normal_texture: texture::Texture,
	pub bind_group: wgpu::BindGroup,
}