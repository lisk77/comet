use comet_resources::Vertex;

pub struct RenderPassInfo {
	shader: &'static str,
	vertex_buffer: Vec<Vertex>,
	index_buffer: Vec<u16>,
}

impl RenderPassInfo {
	pub fn new(shader: &'static str, vertex_buffer: Vec<Vertex>, index_buffer: Vec<u16>) -> Self {
		Self {
			shader,
			vertex_buffer,
			index_buffer
		}
	}

	pub fn shader(&self) -> &'static str {
		self.shader
	}

	pub fn vertex_buffer(&self) -> &Vec<Vertex> {
		&self.vertex_buffer
	}

	pub fn index_buffer(&self) -> &Vec<u16> {
		&self.index_buffer
	}
}