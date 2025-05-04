use std::iter;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use wgpu::BufferUsages;
use wgpu::core::command::DrawKind::Draw;
use wgpu::naga::ShaderStage;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::window::Window;
use comet_colors::Color;
use comet_ecs::{Camera2D, Component, Position2D, Render, Render2D, Transform2D, Scene, Text};
use comet_log::*;
use comet_math::{p2, p3, v2, v3};
use comet_resources::{graphic_resource_manager::GraphicResourceManager, Texture, Vertex};
use comet_resources::texture_atlas::TextureRegion;
use comet_structs::ComponentSet;
use crate::camera::{RenderCamera, CameraUniform};
use crate::draw_info::DrawInfo;
use crate::render_pass::{RenderPassInfo, RenderPassType};
use crate::renderer::Renderer;

pub struct Renderer2D<'a> {
	surface: wgpu::Surface<'a>,
	device: wgpu::Device,
	queue: wgpu::Queue,
	config: wgpu::SurfaceConfiguration,
	size: PhysicalSize<u32>,
	render_pipeline_layout: wgpu::PipelineLayout,
	universal_render_pipeline: wgpu::RenderPipeline,
	texture_bind_group_layout: wgpu::BindGroupLayout,
	dummy_texture_bind_group: wgpu::BindGroup,
	texture_sampler: wgpu::Sampler,
	camera: RenderCamera,
	camera_uniform: CameraUniform,
	camera_buffer: wgpu::Buffer,
	camera_bind_group: wgpu::BindGroup,
	render_pass: Vec<RenderPassInfo>,
	draw_info: Vec<DrawInfo>,
	graphic_resource_manager: GraphicResourceManager,
	delta_time: f32,
	last_frame_time: Instant,
	clear_color: wgpu::Color,
}

impl<'a> Renderer2D<'a> {
    pub fn new(window: Arc<Window>, clear_color: Option<impl Color>) -> Renderer2D<'a> {
		let size = PhysicalSize::<u32>::new(1920, 1080);

		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
			backends: wgpu::Backends::PRIMARY,
			..Default::default()
		});

		let surface = instance.create_surface(window).unwrap();

		let adapter = pollster::block_on(instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				compatible_surface: Some(&surface),
				force_fallback_adapter: false,
			}))
			.unwrap();

		let (device, queue) = pollster::block_on(adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					label: None,
					required_features: wgpu::Features::empty(),
					required_limits: wgpu::Limits::default(),
					memory_hints: Default::default(),
				},
				None, // Trace path
			))
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

		let graphic_resource_manager = GraphicResourceManager::new();

		let diffuse_bytes = include_bytes!(r"../../../resources/textures/comet_icon.png");
		let diffuse_texture =
			Texture::from_bytes(&device, &queue, diffuse_bytes, "comet_icon.png", false).unwrap();

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

		let camera = RenderCamera::new(1.0, v2::new(2.0, 2.0), v3::new(0.0, 0.0, 0.0));

		let mut camera_uniform = CameraUniform::new();
		camera_uniform.update_view_proj(&camera);

		let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Camera Buffer"),
			contents: bytemuck::cast_slice(&[camera_uniform]),
			usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
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

		let universal_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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

		let mut render_pass: Vec<RenderPassInfo> = Vec::new();
		/*render_pass.push(RenderPassInfo::new_engine_pass(
			&device,
			"Standard Render Pass".to_string(),
			&texture_bind_group_layout,
			&diffuse_texture,
			vec![],
			vec![],
		));*/

		let clear_color = match clear_color {
			Some(color) => color.to_wgpu(),
			None => wgpu::Color {
				r: 0.0,
				g: 0.0,
				b: 0.0,
				a: 1.0,
			}
		};

		let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Linear,
			mipmap_filter: wgpu::FilterMode::Linear,
			lod_min_clamp: 0.0,
			lod_max_clamp: 100.0,
			compare: None,
			anisotropy_clamp: 16,
			border_color: None,
			..Default::default()
		});

		let empty_texture = device.create_texture(&wgpu::TextureDescriptor {
			label: Some("Empty Texture"),
			size: wgpu::Extent3d {
				width: config.width,
				height: config.height,
				depth_or_array_layers: 1,
			},
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Bgra8UnormSrgb,
			usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
			view_formats: &[wgpu::TextureFormat::Bgra8UnormSrgb],
		});

		let dummy_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &texture_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&empty_texture.create_view(&wgpu::TextureViewDescriptor::default())),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&texture_sampler),
				},
			],
			label: Some("dummy_texture_bind_group"),
		});
		
		let mut draw_info: Vec<DrawInfo> = Vec::new();

		Self {
			surface,
			device,
			queue,
			config,
			size,
			render_pipeline_layout,
			universal_render_pipeline,
			texture_bind_group_layout,
			dummy_texture_bind_group,
			texture_sampler,
			camera,
			camera_uniform,
			camera_buffer,
			camera_bind_group,
			render_pass,
			draw_info,
			graphic_resource_manager,
			delta_time: 0.0,
			last_frame_time: Instant::now(),
			clear_color,
		}
	}
}