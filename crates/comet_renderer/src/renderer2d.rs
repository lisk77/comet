use std::iter;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use wgpu::{Color, ShaderModule};
use wgpu::naga::ShaderStage;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::window::Window;
use comet_colors::LinearRgba;
use comet_ecs::{Component, ComponentSet, Render, Render2D, Transform2D, World};
use comet_log::{debug, info};
use comet_math::{Point3, Vec2, Vec3};
use comet_resources::{texture, graphic_resource_manager::GraphicResorceManager, Texture, Vertex};
use comet_resources::texture_atlas::TextureRegion;
use crate::camera::{Camera, CameraUniform};
use crate::render_pass::RenderPassInfo;
use crate::renderer::Renderer;

pub struct Renderer2D<'a> {
	surface: wgpu::Surface<'a>,
	device: wgpu::Device,
	queue: wgpu::Queue,
	config: wgpu::SurfaceConfiguration,
	size: winit::dpi::PhysicalSize<u32>,
	render_pipeline_layout: wgpu::PipelineLayout,
	pipelines: Vec<wgpu::RenderPipeline>,
	render_pass: Vec<RenderPassInfo>,
	last_frame_time: Instant,
	deltatime: f32,
	vertex_buffer: wgpu::Buffer,
	vertex_data: Vec<Vertex>,
	index_buffer: wgpu::Buffer,
	index_data: Vec<u16>,
	num_indices: u32,
	clear_color: Color,
	diffuse_texture: texture::Texture,
	diffuse_bind_group: wgpu::BindGroup,
	graphic_resource_manager: GraphicResorceManager,
	camera: Camera,
	camera_uniform: CameraUniform,
	camera_buffer: wgpu::Buffer,
	camera_bind_group: wgpu::BindGroup,
}

impl<'a> Renderer2D<'a> {
	pub async fn new(window: Arc<Window>, clear_color: Option<LinearRgba>) -> Renderer2D<'a> {
		let vertex_data: Vec<Vertex> = vec![];
		let index_data: Vec<u16> = vec![];

		let size = PhysicalSize::<u32>::new(1920, 1080);

		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
			backends: wgpu::Backends::PRIMARY,
			..Default::default()
		});

		let surface = instance.create_surface(window).unwrap();

		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				compatible_surface: Some(&surface),
				force_fallback_adapter: false,
			})
			.await
			.unwrap();

		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					label: None,
					required_features: wgpu::Features::empty(),
					required_limits: wgpu::Limits::default(),
					memory_hints: Default::default(),
				},
				None, // Trace path
			)
			.await
			.unwrap();

		let surface_caps = surface.get_capabilities(&adapter);
		let surface_format = surface_caps
			.formats
			.iter()
			.copied()
			.find(|f| f.is_srgb())
			.unwrap_or(surface_caps.formats[0]);
		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface_format,
			width: size.width,
			height: size.height,
			present_mode: surface_caps.present_modes[0],
			alpha_mode: surface_caps.alpha_modes[0],
			view_formats: vec![],
			desired_maximum_frame_latency: 2,
		};

		let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("Shader"),
			source: wgpu::ShaderSource::Wgsl(include_str!("base2d.wgsl").into()),
		});

		let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Vertex Buffer"),
			contents: bytemuck::cast_slice(&vertex_data),
			usage: wgpu::BufferUsages::VERTEX,
		});

		let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Index Buffer"),
			contents: bytemuck::cast_slice(&index_data),
			usage: wgpu::BufferUsages::INDEX
		});

		let num_indices = index_data.len() as u32;

		let graphic_resource_manager = GraphicResorceManager::new();

		let diffuse_bytes = include_bytes!(r"../../../resources/textures/comet_icon.png");
		let diffuse_texture =
			texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "comet_icon.png", false).unwrap();

		let texture_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Texture {
							multisampled: false,
							view_dimension: wgpu::TextureViewDimension::D2,
							sample_type: wgpu::TextureSampleType::Float { filterable: true },
						},
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
						count: None,
					},
				],
				label: Some("texture_bind_group_layout"),
			});

		let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &texture_bind_group_layout,
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
			label: Some("diffuse_bind_group"),
		});

		let camera = Camera::new(1.0, Vec2::new(2.0, 2.0), Vec3::new(0.0, 0.0, 0.0));

		let mut camera_uniform = CameraUniform::new();
		camera_uniform.update_view_proj(&camera);

		let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Camera Buffer"),
			contents: bytemuck::cast_slice(&[camera_uniform]),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		});

		let camera_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				entries: &[wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::VERTEX,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				}],
				label: Some("camera_bind_group_layout"),
			});

		let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &camera_bind_group_layout,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: camera_buffer.as_entire_binding(),
			}],
			label: Some("camera_bind_group"),
		});

		let render_pipeline_layout =
			device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("Render Pipeline Layout"),
				bind_group_layouts: &[
					&texture_bind_group_layout,
					&camera_bind_group_layout,
				],
				push_constant_ranges: &[],
			});

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&render_pipeline_layout),
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

		let mut pipelines = Vec::new();
		pipelines.push(render_pipeline);

		let clear_color = match clear_color {
			Some(color) => color.to_wgpu(),
			None => wgpu::Color {
				r: 0.1,
				g: 0.2,
				b: 0.3,
				a: 1.0,
			}
		};

		Self {
			surface,
			device,
			queue,
			config,
			size,
			render_pipeline_layout,
			pipelines,
			render_pass: vec![],
			last_frame_time: Instant::now(),
			deltatime: 0.0,
			vertex_buffer,
			vertex_data,
			index_buffer,
			index_data,
			num_indices,
			clear_color,
			diffuse_texture,
			diffuse_bind_group,
			graphic_resource_manager,
			camera,
			camera_uniform,
			camera_buffer,
			camera_bind_group,
		}
	}

	pub fn dt(&self) -> f32 {
		self.deltatime
	}

	pub fn config(&self) -> &wgpu::SurfaceConfiguration {
		&self.config
	}

	pub fn size(&self) -> PhysicalSize<u32> {
		self.size
	}

	pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		if new_size.width > 0 && new_size.height > 0 {
			//self.projection.resize(new_size.width, new_size.height);
			self.size = new_size;
			self.config.width = new_size.width;
			self.config.height = new_size.height;
			self.surface.configure(&self.device, &self.config);
		}
	}

	pub fn add_render_pass(&mut self, render_pass_info: RenderPassInfo) {
		self.render_pass.push(render_pass_info);
	}

	/// A function that loads a shader from the resources/shaders folder given the full name of the shader file.
	pub fn load_shader(&mut self, shader_stage: Option<ShaderStage>, file_name: &str) {
		self.graphic_resource_manager.load_shader(shader_stage, ((Self::get_project_root().unwrap().as_os_str().to_str().unwrap().to_string() + "\\resources\\shaders\\").as_str().to_string() + file_name.clone()).as_str(), &self.device).unwrap();
		info!("Shader ({}) loaded successfully", file_name);
	}

	pub fn load_shaders(&mut self, shader_stages: Vec<Option<ShaderStage>>, file_names: Vec<&str>) {
		for (i, file_name) in file_names.iter().enumerate() {
			self.load_shader(shader_stages[i].clone(), file_name);
			info!("Shader ({}) loaded successfully", file_name);
		}
	}

	/// A function that applies a shader to the entire surface of the `Renderer2D` if the shader is loaded.
	pub fn apply_shader(&mut self, shader: &str) {
		let shader_module = self.graphic_resource_manager.get_shader(((Self::get_project_root().unwrap().as_os_str().to_str().unwrap().to_string() + "\\resources\\shaders\\").as_str().to_string() + shader).as_str()).unwrap();
		let texture_bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						multisampled: false,
						view_dimension: wgpu::TextureViewDimension::D2,
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},
			],
			label: Some("texture_bind_group_layout"),
		});

		let camera_bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			entries: &[wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::VERTEX,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			}],
			label: Some("camera_bind_group_layout"),
		});

		let render_pipeline_layout =
			self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("Render Pipeline Layout"),
				bind_group_layouts: &[
					&texture_bind_group_layout,
					&camera_bind_group_layout,
				],
				push_constant_ranges: &[],
			});

		self.pipelines[0] = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader_module,
				entry_point: "vs_main",
				buffers: &[Vertex::desc()],
				compilation_options: Default::default(),
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader_module,
				entry_point: "fs_main",
				targets: &[Some(wgpu::ColorTargetState {
					format: self.config.format,
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

		info!("Applied shader ({})!", shader);
	}

	/// A function to revert back to the base shader of the `Renderer2D`
	pub fn apply_base_shader(&mut self) {
		let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("Shader"),
			source: wgpu::ShaderSource::Wgsl(include_str!("base2d.wgsl").into()),
		});

		let texture_bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						multisampled: false,
						view_dimension: wgpu::TextureViewDimension::D2,
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},
			],
			label: Some("texture_bind_group_layout"),
		});

		let camera_bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			entries: &[wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::VERTEX,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			}],
			label: Some("camera_bind_group_layout"),
		});

		let render_pipeline_layout =
			self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("Render Pipeline Layout"),
				bind_group_layouts: &[
					&texture_bind_group_layout,
					&camera_bind_group_layout,
				],
				push_constant_ranges: &[],
			});

		self.pipelines[0] = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&render_pipeline_layout),
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
					format: self.config.format,
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
		info!("Applied base shader!");
	}

	/// An interface for getting the location of the texture in the texture atlas.
	pub fn get_texture_region(&self, texture_path: String) -> &TextureRegion {
		assert!(self.graphic_resource_manager.texture_atlas().textures().contains_key(&texture_path), "Texture not found in atlas");
		self.graphic_resource_manager.texture_atlas().textures().get(&texture_path).unwrap()
	}

	fn create_rectangle(&self, width: f32, height: f32) -> Vec<Vertex> {
		let (bound_x, bound_y) =
			((width/ self.config.width as f32) * 0.5, (height/ self.config.height as f32) * 0.5);

		vec![
			Vertex :: new ( [-bound_x,  bound_y, 0.0], [0.0, 0.0], [0.0, 0.0, 0.0, 0.0] ),
			Vertex :: new ( [-bound_x, -bound_y, 0.0], [0.0, 1.0], [0.0, 0.0, 0.0, 0.0] ),
			Vertex :: new ( [ bound_x, -bound_y, 0.0], [1.0, 1.0], [0.0, 0.0, 0.0, 0.0] ),
			Vertex :: new ( [ bound_x,  bound_y, 0.0], [1.0, 0.0], [0.0, 0.0, 0.0, 0.0] )
		]
	}

	/// A function that allows you to set the texture atlas with a list of paths to the textures.
	/// The old texture atlas will be replaced with the new one.
	pub fn set_texture_atlas(&mut self, paths: Vec<String>) {
		self.graphic_resource_manager.create_texture_atlas(paths);
		self.diffuse_texture = Texture::from_image(&self.device, &self.queue, self.graphic_resource_manager.texture_atlas().atlas(), None, false).unwrap();

		let texture_bind_group_layout =
			self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Texture {
							multisampled: false,
							view_dimension: wgpu::TextureViewDimension::D2,
							sample_type: wgpu::TextureSampleType::Float { filterable: true },
						},
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
						count: None,
					},
				],
				label: Some("texture_bind_group_layout"),
			});

		let diffuse_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &texture_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&self.diffuse_texture.view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&self.diffuse_texture.sampler),
				},
			],
			label: Some("diffuse_bind_group"),
		});

		self.diffuse_bind_group = diffuse_bind_group;
	}

	fn get_project_root() -> std::io::Result<PathBuf> {
		let path = std::env::current_dir()?;
		let mut path_ancestors = path.as_path().ancestors();

		while let Some(p) = path_ancestors.next() {
			let has_cargo =
				std::fs::read_dir(p)?
					.into_iter()
					.any(|p| p.unwrap().file_name() == std::ffi::OsString::from("Cargo.lock"));
			if has_cargo {
				return Ok(PathBuf::from(p))
			}
		}
		Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Ran out of places to find Cargo.toml"))
	}

	/// A function that takes all of the textures inside of the resources/textures folder and creates a texture atlas from them.
	pub fn initialize_atlas(&mut self) {
		let texture_path = "resources/textures/".to_string();
		let mut paths: Vec<String> = Vec::new();

		for path in std::fs::read_dir(Self::get_project_root().unwrap().as_os_str().to_str().unwrap().to_string() + "\\resources\\textures").unwrap() {
			paths.push(texture_path.clone() + path.unwrap().file_name().to_str().unwrap());
		}

		self.set_texture_atlas(paths);
	}

	/// A function that clears the buffers and sets the vertex and index buffer of the `Renderer2D` with the given data.
	fn set_buffers(&mut self, new_vertex_buffer: Vec<Vertex>, new_index_buffer: Vec<u16>) {
		match new_vertex_buffer == self.vertex_data {
			true => return,
			false => {
				self.vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
					label: Some("Updated Vertex Buffer"),
					contents: bytemuck::cast_slice(&new_vertex_buffer),
					usage: wgpu::BufferUsages::VERTEX,
				});
				self.vertex_data = new_vertex_buffer;
			}
		}

		match new_index_buffer == self.index_data {
			true => return,
			false => {
				self.index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
					label: Some("Updated Index Buffer"),
					contents: bytemuck::cast_slice(&new_index_buffer),
					usage: wgpu::BufferUsages::INDEX,
				});
				self.index_data = new_index_buffer.clone();
				self.num_indices = new_index_buffer.len() as u32;
			}
		}
	}

	/// A function that adds data to the already existing vertex and index buffers of the `Renderer2D`.
	fn push_to_buffers(&mut self, new_vertex_buffer: &mut Vec<Vertex>, new_index_buffer: &mut Vec<u16>) {
		self.vertex_data.append(new_vertex_buffer);
		self.index_data.append(new_index_buffer);

		self.vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Updated Vertex Buffer"),
			contents: bytemuck::cast_slice(&self.vertex_data),
			usage: wgpu::BufferUsages::VERTEX,
		});

		self.index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Updated Index Buffer"),
			contents: bytemuck::cast_slice(&self.index_data),
			usage: wgpu::BufferUsages::INDEX,
		});

		self.num_indices = self.index_data.len() as u32;
	}

	/// A function that clears the vertex and index buffers of the `Renderer2D`.
	fn clear_buffers(&mut self) {
		self.vertex_data = vec![];
		self.index_data = vec![];

		self.vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Updated Vertex Buffer"),
			contents: bytemuck::cast_slice(&self.vertex_data),
			usage: wgpu::BufferUsages::VERTEX,
		});

		self.index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Updated Index Buffer"),
			contents: bytemuck::cast_slice(&self.index_data),
			usage: wgpu::BufferUsages::INDEX,
		});

		self.num_indices = self.index_data.len() as u32;
	}

	/// A function to just draw a textured quad at a given position.
	pub fn draw_texture_at(&mut self, texture_path: String, position: Point3) {
		let region = self.graphic_resource_manager.texture_locations().get(&texture_path).unwrap();
		let (dim_x, dim_y) = region.dimensions();

		let (bound_x, bound_y) =
			((dim_x as f32/ self.config.width as f32) * 0.5, (dim_y as f32/ self.config.height as f32) * 0.5);

		let vertices: &mut Vec<Vertex> = &mut vec![
			Vertex :: new ( [-bound_x + position.x(),  bound_y + position.y(), 0.0 + position.z()], [region.x0(), region.y0()], [0.0, 0.0, 0.0, 0.0] ),
			Vertex :: new ( [-bound_x + position.x(), -bound_y + position.y(), 0.0 + position.z()], [region.x0(), region.y1()], [0.0, 0.0, 0.0, 0.0] ),
			Vertex :: new ( [ bound_x + position.x(), -bound_y + position.y(), 0.0 + position.z()], [region.x1(), region.y1()], [0.0, 0.0, 0.0, 0.0] ) ,
			Vertex :: new ( [ bound_x + position.x(),  bound_y + position.y(), 0.0 + position.z()], [region.x1(), region.y0()], [0.0, 0.0, 0.0, 0.0] )
		];

		let buffer_size = self.vertex_data.len() as u16;

		let indices: &mut Vec<u16> = &mut vec![
			0 + buffer_size, 1 + buffer_size, 3 + buffer_size,
			1 + buffer_size, 2 + buffer_size, 3 + buffer_size
		];

		self.push_to_buffers(vertices, indices)
	}

	/// A function to draw text at a given position.
	pub fn draw_text_at(&mut self, text: &str, position: Point3) {
		todo!()
	}

	/// A function to automatically render all the entities of the `World` struct.
	/// The entities must have the `Render2D` and `Transform2D` components to be rendered as well as set visible.
	pub fn render_scene_2d(&mut self, world: &World) {
		let entities =  world.get_entities_with(ComponentSet::from_ids(vec![Render2D::type_id()]));
		let mut vertex_buffer: Vec<Vertex> = Vec::new();
		let mut index_buffer: Vec<u16> = Vec::new();

		for entity in entities {
			let renderer_component =  world.get_component::<Render2D>(entity as usize);
			let transform_component = world.get_component::<Transform2D>(entity as usize);

			if renderer_component.is_visible() {
				//renderer.draw_texture_at(renderer_component.get_texture(), Point3::new(transform_component.position().x(), transform_component.position().y(), 0.0));
				let mut position = transform_component.position().clone();
				position.set_x(position.x() / self.config().width as f32);
				position.set_y(position.y() / self.config().height as f32);
				let region = self.get_texture_region(renderer_component.get_texture().to_string());
				let (dim_x, dim_y) = region.dimensions();

				let (bound_x, bound_y) =
					((dim_x as f32/ self.config().width as f32) * 0.5, (dim_y as f32/ self.config().height as f32) * 0.5);

				let buffer_size = vertex_buffer.len() as u16;

				vertex_buffer.append(&mut vec![
					Vertex :: new ( [-bound_x + position.x(),  bound_y + position.y(), 0.0], [region.x0(), region.y0()], [0.0, 0.0, 0.0, 0.0] ),
					Vertex :: new ( [-bound_x + position.x(), -bound_y + position.y(), 0.0], [region.x0(), region.y1()], [0.0, 0.0, 0.0, 0.0] ),
					Vertex :: new ( [ bound_x + position.x(), -bound_y + position.y(), 0.0], [region.x1(), region.y1()], [0.0, 0.0, 0.0, 0.0] ) ,
					Vertex :: new ( [ bound_x + position.x(),  bound_y + position.y(), 0.0], [region.x1(), region.y0()], [0.0, 0.0, 0.0, 0.0] )
				]);

				index_buffer.append(&mut vec![
					0 + buffer_size, 1 + buffer_size, 3 + buffer_size,
					1 + buffer_size, 2 + buffer_size, 3 + buffer_size
				]);
			}
		}

		self.set_buffers(vertex_buffer, index_buffer);
	}

	pub fn update(&mut self) -> f32 {
		let now = Instant::now();
		self.deltatime = now.duration_since(self.last_frame_time).as_secs_f32();  // Time delta in seconds
		self.last_frame_time = now;
		self.deltatime
	}

	pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
		let output = self.surface.get_current_texture()?;
		let view = output
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor {
				label: Some("Render Encoder"),
			});

		for pipeline in &self.pipelines {
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(self.clear_color),
						store: wgpu::StoreOp::Store,
					},
				})],
				depth_stencil_attachment: None,
				occlusion_query_set: None,
				timestamp_writes: None,
			});

			render_pass.set_pipeline(pipeline);
			render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
			render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
			render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
			render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
			render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
		}

		self.queue.submit(iter::once(encoder.finish()));
		output.present();

		Ok(())
	}
}

impl<'a> Renderer for Renderer2D<'a> {

	async fn new(window: Arc<Window>, clear_color: Option<LinearRgba>) -> Renderer2D<'a> {
		Self::new(window, clear_color).await
	}

	fn size(&self) -> PhysicalSize<u32> {
		self.size()
	}

	fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		self.resize(new_size)
	}

	fn update(&mut self) -> f32 {
		self.update()
	}

	fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
		self.render()
	}
}