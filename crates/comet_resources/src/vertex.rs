use wgpu::Color;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct Vertex {
	position: [f32; 3],
	tex_coords: [f32; 2],
	color: [f32; 4]
}

impl Vertex {
	pub fn new(position: [f32; 3], tex_coords: [f32; 2], color: [f32; 4]) -> Self {
		Self {
			position,
			tex_coords,
			color
		}
	}

	pub fn set_position(&mut self, new_position: [f32;3]) {
		self.position = new_position
	}

	pub fn set_tex_coords(&mut self, new_tex_coords: [f32; 2]) {
		self.tex_coords = new_tex_coords
	}

	pub fn set_color(&mut self, new_color: [f32; 4]) {
		self.color = new_color
	}

	pub fn desc() -> wgpu::VertexBufferLayout<'static> {
		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &[
				wgpu::VertexAttribute {
					offset: 0,
					shader_location: 0,
					format: wgpu::VertexFormat::Float32x3,
				},
				wgpu::VertexAttribute {
					offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
					shader_location: 1,
					format: wgpu::VertexFormat::Float32x2,
				},
				wgpu::VertexAttribute {
					offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
					shader_location: 2,
					format: wgpu::VertexFormat::Float32x4,
				}
			]
		}
	}
}