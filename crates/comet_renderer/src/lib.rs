mod camera;

use core::default::Default;
use std::iter;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use cgmath::num_traits::FloatConst;
use image::GenericImageView;
use wgpu::Color;
use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalSize,
    window::Window
};
use winit::dpi::Position;
use comet_colors::LinearRgba;
use comet_ecs::{Component, ComponentSet, Render, Renderer2D, Transform2D, World};
use comet_log::*;
use comet_math;
use comet_math::{Mat4, Point3, Vec2, Vec3};
use comet_resources::{ResourceManager, texture, Vertex, Texture};
use comet_resources::texture_atlas::TextureRegion;
use crate::camera::{Camera, CameraUniform};

pub struct Projection {
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32
}

impl Projection {
    pub fn new(width: u32, height: u32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy,
            znear,
            zfar
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) { self.aspect = width as f32 / height as f32; }

    pub fn calc_matrix(&self) -> Mat4 {
        Mat4::perspective_matrix(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

pub struct Renderer<'a> {
    window: &'a Window,
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    //projection: Projection,
    render_pipeline: wgpu::RenderPipeline,
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
    resource_manager: ResourceManager,
    camera: Camera,
	camera_uniform: CameraUniform,
	camera_buffer: wgpu::Buffer,
	camera_bind_group: wgpu::BindGroup,
}

impl<'a> Renderer<'a> {
    pub async fn new(window: &'a Window, clear_color: Option<LinearRgba>) -> anyhow::Result<Renderer<'a>> {
        let vertex_data: Vec<Vertex> = vec![];
        let index_data: Vec<u16> = vec![];

        let size = PhysicalSize::<u32>::new(1920, 1080); //window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
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
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
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
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
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

        let resource_manager = ResourceManager::new();

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
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
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
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
            // Useful for optimizing shader compilation on Android
            cache: None,
        });

        let clear_color = match clear_color {
            Some(color) => color.to_wgpu(),
            None => wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            }
        };

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            //projection,
            render_pipeline,
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
            resource_manager,
            camera,
			camera_uniform,
			camera_buffer,
			camera_bind_group,
        })
    }

    pub fn dt(&self) -> f32 {
        self.deltatime
    }

    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    fn vertex_data_mut(&mut self) -> &mut Vec<Vertex> {
        &mut self.vertex_data
    }

    fn index_data_mut(&mut self) -> &mut Vec<u16> {
        &mut self.index_data
    }

    pub fn get_texture(&self, texture_path: String) -> &TextureRegion {
        assert!(self.resource_manager.texture_atlas().textures().contains_key(&texture_path), "Texture not found in atlas");
        self.resource_manager.texture_atlas().textures().get(&texture_path).unwrap()
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

    pub fn display_atlas(&mut self) {
        let atlas = vec![
            r"C:\Users\lisk77\Code Sharing\comet-engine\resources\textures\comet-128.png".to_string(),
            r"C:\Users\lisk77\Code Sharing\comet-engine\resources\textures\comet-256.png".to_string(),
        ];

        //self.diffuse_texture = Texture::from_image(&self.device, &self.queue, atlas.atlas(), None, false).unwrap();

        self.set_texture_atlas(atlas);

        let (bound_x, bound_y) =
            ((self.diffuse_texture.size.width as f32/ self.config.width as f32) * 0.5, (self.diffuse_texture.size.height as f32/ self.config.height as f32) * 0.5);

        let vertices: Vec<Vertex> = vec![
            Vertex :: new ( [-bound_x,  bound_y, 0.0], [0.0, 0.0], [0.0, 0.0, 0.0, 0.0] ),
            Vertex :: new ( [-bound_x, -bound_y, 0.0], [0.0, 1.0], [0.0, 0.0, 0.0, 0.0] ),
            Vertex :: new ( [ bound_x, -bound_y, 0.0], [1.0, 1.0], [0.0, 0.0, 0.0, 0.0] ),
            Vertex :: new ( [ bound_x,  bound_y, 0.0], [1.0, 0.0], [0.0, 0.0, 0.0, 0.0] )
        ];

        /*let vertices: Vec<Vertex> = vec![
			Vertex :: new ( [-1.0,  1.0, 0.0], [0.0, 0.0] ),
			Vertex :: new ( [-1.0, -1.0, 0.0], [0.0, 1.0] ),
			Vertex :: new ( [ 1.0, -1.0, 0.0], [1.0, 1.0]) ,
			Vertex :: new ( [ 1.0,  1.0, 0.0], [1.0, 0.0] )
		];*/

        let indices: Vec<u16> = vec![
            0, 1, 3,
            1, 2, 3
        ];

        self.set_buffers(vertices, indices)
    }

    pub fn set_texture_atlas(&mut self, paths: Vec<String>) {
        self.resource_manager.create_texture_atlas(paths);
        self.diffuse_texture = Texture::from_image(&self.device, &self.queue, self.resource_manager.texture_atlas().atlas(), None, false).unwrap();

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
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
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

    pub fn get_project_root() -> std::io::Result<PathBuf> {
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

    pub fn initialize_atlas(&mut self) {
        let texture_path = "resources/textures/".to_string();
        let mut paths: Vec<String> = Vec::new();

        for path in std::fs::read_dir(Self::get_project_root().unwrap().as_os_str().to_str().unwrap().to_string() + "\\resources\\textures").unwrap() {
            paths.push(texture_path.clone() + path.unwrap().file_name().to_str().unwrap());
        }

        self.set_texture_atlas(paths);
    }

    pub fn set_buffers(&mut self, new_vertex_buffer: Vec<Vertex>, new_index_buffer: Vec<u16>) {
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

    pub fn push_to_buffers(&mut self, new_vertex_buffer: &mut Vec<Vertex>, new_index_buffer: &mut Vec<u16>) {
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

    pub fn clear_buffers(&mut self) {
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

    pub fn draw_texture_at(&mut self, texture_path: String, position: Point3) {
        let region = self.resource_manager.texture_locations().get(&texture_path).unwrap();
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

    pub fn render_scene_2d(&mut self, world: &World) {
        let entities =  world.get_entities_with(ComponentSet::from_ids(vec![Renderer2D::type_id()]));
        let mut vertex_buffer: Vec<Vertex> = Vec::new();
        let mut index_buffer: Vec<u16> = Vec::new();

        for entity in entities {
            let renderer_component =  world.get_component::<Renderer2D>(entity as usize);
            let transform_component = world.get_component::<Transform2D>(entity as usize);

            if renderer_component.is_visible() {
                //renderer.draw_texture_at(renderer_component.get_texture(), Point3::new(transform_component.position().x(), transform_component.position().y(), 0.0));
                let mut position = transform_component.position().clone();
                position.set_x(position.x() / self.config().width as f32);
                position.set_y(position.y() / self.config().height as f32);
                let region = self.get_texture(renderer_component.get_texture().to_string());
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

    pub fn window(&self) -> &Window {
        &self.window
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

        {
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

            render_pass.set_pipeline(&self.render_pipeline);
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