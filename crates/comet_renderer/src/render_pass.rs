use wgpu::{ShaderModule, BindGroup, BindGroupLayout, BufferUsages, Device, Queue, RenderPipeline, PipelineLayout, SurfaceConfiguration, TextureFormat};
use wgpu::util::DeviceExt;
use comet_resources::{Vertex, Texture};

#[derive(Debug, Clone)]
pub enum RenderPassType {
	Engine,
	User
}

pub struct RenderPassInfo {
	pass_name: String,
	pass_type: RenderPassType,
	texture_bind_group: BindGroup,
	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,
	vertex_data: Vec<Vertex>,
	index_data: Vec<u16>,
	num_indices: u32,
	pipeline: Option<RenderPipeline>
}

impl RenderPassInfo {
	pub fn new_user_pass(
		device: &Device,
		pass_name: String,
		texture_group_layout: &BindGroupLayout,
		texture: &Texture,
		shader: ShaderModule,
		vertex_data: Vec<Vertex>,
		index_data: Vec<u16>,
		pipeline_layout: &PipelineLayout,
		config: &SurfaceConfiguration
	) -> Self {
		let num_indices = index_data.len() as u32;
		let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some(format!("{} Vertex Buffer", pass_name).as_str()),
			contents: bytemuck::cast_slice(&vertex_data),
			usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
		});

		let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some(format!("{} Index Buffer", pass_name).as_str()),
			contents: bytemuck::cast_slice(&index_data),
			usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
		});

		let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &texture_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&texture.view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&texture.sampler),
				},
			],
			label: Some(format!("{} Texture Bind Group", pass_name).as_str()),
		});

		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[Vertex::desc()],
				compilation_options: Default::default(),
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fs_main",
				targets: &[Some(wgpu::ColorTargetState {
					format: config.format,
					blend: Some(wgpu::BlendState {
						color: wgpu::BlendComponent {
							src_factor: wgpu::BlendFactor::SrcAlpha,
							dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
							operation: wgpu::BlendOperation::Add,
						},
						alpha: wgpu::BlendComponent {
							src_factor: wgpu::BlendFactor::One,
							dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
							operation: wgpu::BlendOperation::Add,
						},
					}),
					write_mask: wgpu::ColorWrites::ALL,
				})],
				compilation_options: Default::default(),
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList,
				strip_index_format: None,
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: Some(wgpu::Face::Back),
				polygon_mode: wgpu::PolygonMode::Fill,
				unclipped_depth: false,
				conservative: false,
			},
			depth_stencil: None,
			multisample: wgpu::MultisampleState {
				count: 1,
				mask: !0,
				alpha_to_coverage_enabled: false,
			},
			multiview: None,
			cache: None,
		});

		Self {
			pass_name,
			pass_type: RenderPassType::User,
			texture_bind_group,
			vertex_buffer,
			index_buffer,
			vertex_data,
			index_data,
			num_indices,
			pipeline: Some(pipeline)
		}
	}

	pub fn new_engine_pass(
		device: &Device,
		pass_name: String,
		texture_group_layout: &BindGroupLayout,
		texture: &Texture,
		vertex_data: Vec<Vertex>,
		index_data: Vec<u16>,
	) -> Self {
		let num_indices = index_data.len() as u32;
		let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some(format!("{} Vertex Buffer", pass_name).as_str()),
			contents: bytemuck::cast_slice(&vertex_data),
			usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
		});

		let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some(format!("{} Index Buffer", pass_name).as_str()),
			contents: bytemuck::cast_slice(&index_data),
			usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
		});

		let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &texture_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&texture.view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&texture.sampler),
				},
			],
			label: Some(format!("{} Texture Bind Group", pass_name).as_str()),
		});
		Self {
			pass_name,
			pass_type: RenderPassType::Engine,
			texture_bind_group,
			vertex_buffer,
			index_buffer,
			vertex_data,
			index_data,
			num_indices,
			pipeline: None
		}
	}

	pub fn pass_name(&self) -> &str {
		&self.pass_name
	}

	pub fn pass_type(&self) -> RenderPassType {
		self.pass_type.clone()
	}

	pub fn set_shader(&mut self, device: &Device, config: &SurfaceConfiguration, pipeline_layout: &PipelineLayout, shader: &ShaderModule) {
		self.pipeline = Some(device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[Vertex::desc()],
				compilation_options: Default::default(),
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fs_main",
				targets: &[Some(wgpu::ColorTargetState {
					format: config.format,
					blend: Some(wgpu::BlendState {
						color: wgpu::BlendComponent {
							src_factor: wgpu::BlendFactor::SrcAlpha,
							dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
							operation: wgpu::BlendOperation::Add,
						},
						alpha: wgpu::BlendComponent {
							src_factor: wgpu::BlendFactor::One,
							dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
							operation: wgpu::BlendOperation::Add,
						},
					}),
					write_mask: wgpu::ColorWrites::ALL,
				})],
				compilation_options: Default::default(),
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList,
				strip_index_format: None,
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: Some(wgpu::Face::Back),
				polygon_mode: wgpu::PolygonMode::Fill,
				unclipped_depth: false,
				conservative: false,
			},
			depth_stencil: None,
			multisample: wgpu::MultisampleState {
				count: 1,
				mask: !0,
				alpha_to_coverage_enabled: false,
			},
			multiview: None,
			cache: None,
		}));
	}

	pub fn texture_bind_group(&self) -> &BindGroup {
		&self.texture_bind_group
	}

	pub fn set_texture(&mut self, device: &Device, layout: &BindGroupLayout, texture: &Texture) {
		self.texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&texture.view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&texture.sampler),
				},
			],
			label: Some(format!("{} Texture Bind Group", self.pass_name).as_str()),
		});
	}

	pub fn vertex_buffer(&self) -> &wgpu::Buffer {
		&self.vertex_buffer
	}

	pub fn vertex_data(&self) -> &Vec<Vertex> {
		&self.vertex_data
	}

	pub fn set_vertex_buffer(&mut self, device: &Device, queue: &Queue, vertex_data: Vec<Vertex>) {
		let new_vertex_size = vertex_data.len() as u64 * size_of::<Vertex>() as u64;
		match vertex_data == self.vertex_data {
			true => {},
			false => {
				match new_vertex_size > self.vertex_buffer.size() {
					false => queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertex_data)),
					true => {
						self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
							label: Some(format!("{} Vertex Buffer", self.pass_name).as_str()),
							contents: bytemuck::cast_slice(&vertex_data),
							usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
						});
					}
				}
				self.vertex_data = vertex_data;
			}
		}
	}

	pub fn push_to_vertex_buffer(&mut self, device: &Device, vertex_data: &mut Vec<Vertex>) {
		self.vertex_data.append(vertex_data);
		self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
					label: Some(format!("{} Vertex Buffer", self.pass_name).as_str()),
					contents: bytemuck::cast_slice(&vertex_data),
					usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
		});
	}

	pub fn index_buffer(&self) -> &wgpu::Buffer {
		&self.index_buffer
	}

	pub fn index_data(&self) -> &Vec<u16> {
		&self.index_data
	}

	pub fn num_indices(&self) -> u32 {
		self.num_indices
	}

	pub fn set_index_buffer(&mut self, device: &Device, queue: &Queue, index_data: Vec<u16>) {
		let new_index_size = index_data.len() as u64 * size_of::<u16>() as u64;
		match index_data == self.index_data {
			true => {},
			false => {
				match new_index_size > self.index_buffer.size() {
					false => queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&index_data)),
					true => {
						self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
							label: Some(format!("{} Index Buffer", self.pass_name).as_str()),
							contents: bytemuck::cast_slice(&index_data),
							usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
						});
					}
				}
				self.num_indices = index_data.len() as u32;
				self.index_data = index_data
			}
		}
	}

	pub fn push_to_index_buffer(&mut self, device: &Device, index_data: &mut Vec<u16>) {
		self.index_data.append(index_data);
		self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
					label: Some(format!("{} Index Buffer", self.pass_name).as_str()),
					contents: bytemuck::cast_slice(&index_data),
					usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
		});

		self.num_indices = self.index_data.len() as u32;
	}

	pub fn pipeline(&self) -> Option<&RenderPipeline> {
		self.pipeline.as_ref()
	}
}