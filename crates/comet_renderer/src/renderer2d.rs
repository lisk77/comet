use crate::{
    camera::{CameraUniform, RenderCamera},
    draw_info::DrawInfo,
    renderer::Renderer,
};
use comet_colors::Color;
use comet_ecs::{Camera2D, Component, Position2D, Render, Render2D, Scene, Text, Transform2D};
use comet_log::*;
use comet_math::{p2, v2, v3};
use comet_resources::texture_atlas::TextureRegion;
use comet_resources::{graphic_resource_manager::GraphicResourceManager, Texture, Vertex};
use comet_structs::ComponentSet;
use std::iter;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use wgpu::naga::ShaderStage;
use wgpu::util::DeviceExt;
use wgpu::BufferUsages;
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct Renderer2D<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    universal_render_pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_sampler: wgpu::Sampler,
    camera: RenderCamera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    draw_info: Vec<DrawInfo>,
    graphic_resource_manager: GraphicResourceManager,
    delta_time: f32,
    last_frame_time: Instant,
    clear_color: wgpu::Color,
}

impl<'a> Renderer2D<'a> {
    pub fn new(window: Arc<Window>, clear_color: Option<impl Color>) -> Renderer2D<'a> {
        let size = window.inner_size(); //PhysicalSize::<u32>::new(1920, 1080);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
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
            label: Some("Universal Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("base2d.wgsl").into()),
        });

        let graphic_resource_manager = GraphicResourceManager::new();

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
                label: Some("Universal Texture Bind Group Layout"),
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
                label: Some("Universal Camera Bind Group Layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("Universal Camera Bind Group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Universal Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let universal_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Universal Render Pipeline"),
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

        let clear_color = match clear_color {
            Some(color) => color.to_wgpu(),
            None => wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
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

        let mut draw_info: Vec<DrawInfo> = Vec::new();
        draw_info.push(DrawInfo::new(
            "Universal Draw".to_string(),
            &device,
            &Texture::from_image(
                &device,
                &queue,
                &image::DynamicImage::new(1, 1, image::ColorType::Rgba8),
                None,
                false,
            )
            .unwrap(),
            &texture_bind_group_layout,
            &texture_sampler,
            vec![],
            vec![],
        ));

        Self {
            surface,
            device,
            queue,
            config,
            size,
            universal_render_pipeline,
            texture_bind_group_layout,
            texture_sampler,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            draw_info,
            graphic_resource_manager,
            delta_time: 0.0,
            last_frame_time: Instant::now(),
            clear_color,
        }
    }

    pub fn dt(&self) -> f32 {
        self.delta_time
    }

    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn add_draw_call(&mut self, draw_call: String, texture: Texture) {
        let draw_info = DrawInfo::new(
            draw_call,
            &self.device,
            &texture,
            &self.texture_bind_group_layout,
            &self.texture_sampler,
            vec![],
            vec![],
        );
        self.draw_info.push(draw_info);
    }

    /// A function that loads a shader from the resources/shaders dir given the full name of the shader file.
    pub fn load_shader(&mut self, file_name: &str, shader_stage: Option<ShaderStage>) {
        self.graphic_resource_manager
            .load_shader(
                shader_stage,
                ((Self::get_project_root()
                    .unwrap()
                    .as_os_str()
                    .to_str()
                    .unwrap()
                    .to_string()
                    + "/res/shaders/")
                    .as_str()
                    .to_string()
                    + file_name)
                    .as_str(),
                &self.device,
            )
            .unwrap();
        info!("Shader ({}) loaded successfully", file_name);
    }

    /// A function that loads a list of shaders from the given filenames out of the resources/shaders dir
    pub fn load_shaders(&mut self, shader_stages: Vec<Option<ShaderStage>>, file_names: Vec<&str>) {
        for (i, file_name) in file_names.iter().enumerate() {
            self.load_shader(file_name, shader_stages[i].clone());
            info!("Shader ({}) loaded successfully", file_name);
        }
    }

    /// A function that applies a shader to the entire surface of the `Renderer2D` if the shader is loaded.
    pub fn apply_shader(&mut self, shader: &str) {
        let module = match self.graphic_resource_manager.get_shader(shader) {
            Some(module) => module,
            None => {
                error!("Shader not found");
                return;
            }
        };
    }

    /// A function to revert back to the base shader of the `Renderer2D`
    pub fn apply_base_shader(&mut self) {
        todo!()
    }

    /// A function to load a TTF font from the specified path
    pub fn load_font(&mut self, path: &str, size: f32) {
        self.graphic_resource_manager.load_font(path, size);
        let atlas = self
            .graphic_resource_manager
            .fonts()
            .iter()
            .find(|f| f.name() == path)
            .unwrap()
            .glyphs()
            .atlas();
        let font_info = DrawInfo::new(
            format!("{}", path),
            &self.device,
            &Texture::from_image(&self.device, &self.queue, atlas, None, false).unwrap(),
            &self.texture_bind_group_layout,
            &self.texture_sampler,
            vec![],
            vec![],
        );

        self.draw_info.push(font_info);
    }

    /// An interface for getting the location of the texture in the texture atlas.
    pub fn get_texture_region(&self, texture_path: String) -> Option<&TextureRegion> {
        if !self
            .graphic_resource_manager
            .texture_atlas()
            .textures()
            .contains_key(&texture_path)
        {
            error!("Texture {} not found in atlas", &texture_path);
        }
        self.graphic_resource_manager
            .texture_atlas()
            .textures()
            .get(&texture_path)
    }

    /// A function to get the `TextureRegion` of a specified glyph
    pub fn get_glyph_region(&self, glyph: char, font: String) -> &TextureRegion {
        let font_atlas = self
            .graphic_resource_manager
            .fonts()
            .iter()
            .find(|f| f.name() == font)
            .unwrap();
        font_atlas.get_glyph(glyph).unwrap()
    }

    /// A function that allows you to set the texture atlas with a list of paths to the textures.
    /// The old texture atlas will be replaced with the new one.
    pub fn set_texture_atlas_by_paths(&mut self, paths: Vec<String>) {
        self.graphic_resource_manager.create_texture_atlas(paths);
        self.draw_info[0].set_texture(
            &self.device,
            &self.texture_bind_group_layout,
            &Texture::from_image(
                &self.device,
                &self.queue,
                self.graphic_resource_manager.texture_atlas().atlas(),
                None,
                false,
            )
            .unwrap(),
        );
    }

    fn set_texture_atlas(&mut self, texture_atlas: Texture) {
        self.draw_info[0].set_texture(
            &self.device,
            &self.texture_bind_group_layout,
            &texture_atlas,
        );
    }

    fn get_project_root() -> std::io::Result<PathBuf> {
        let path = std::env::current_dir()?;
        let mut path_ancestors = path.as_path().ancestors();

        while let Some(p) = path_ancestors.next() {
            let has_cargo = std::fs::read_dir(p)?
                .into_iter()
                .any(|p| p.unwrap().file_name() == std::ffi::OsString::from("Cargo.lock"));
            if has_cargo {
                return Ok(PathBuf::from(p));
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Ran out of places to find Cargo.toml",
        ))
    }

    /// A function that takes all the textures inside the resources/textures folder and creates a texture atlas from them.
    pub fn initialize_atlas(&mut self) {
        let texture_path = "res/textures/".to_string();
        let mut paths: Vec<String> = Vec::new();

        for path in std::fs::read_dir(
            Self::get_project_root()
                .unwrap()
                .as_os_str()
                .to_str()
                .unwrap()
                .to_string()
                + "/res/textures",
        )
        .unwrap()
        {
            paths.push(texture_path.clone() + path.unwrap().file_name().to_str().unwrap());
        }

        self.set_texture_atlas_by_paths(paths);
    }

    /// A function that writes on the buffers and sets the geometry and index buffer of the `Renderer2D` with the given data.
    fn set_buffers(&mut self, new_geometry_buffer: Vec<Vertex>, new_index_buffer: Vec<u16>) {
        self.draw_info[0].update_vertex_buffer(&self.device, &self.queue, new_geometry_buffer);
        self.draw_info[0].update_index_buffer(&self.device, &self.queue, new_index_buffer);
    }

    fn add_text_to_buffers(
        &self,
        text: String,
        font: String,
        size: f32,
        position: p2,
        color: wgpu::Color,
    ) -> (Vec<Vertex>, Vec<u16>) {
        let vert_color = [
            color.r as f32,
            color.g as f32,
            color.b as f32,
            color.a as f32,
        ];

        let screen_position = p2::new(
            position.x() / self.config.width as f32,
            position.y() / self.config.height as f32,
        );
        let scale_factor = size
            / self
                .graphic_resource_manager
                .fonts()
                .iter()
                .find(|f| f.name() == font)
                .unwrap()
                .size();

        let line_height = (self
            .graphic_resource_manager
            .fonts()
            .iter()
            .find(|f| f.name() == font)
            .unwrap()
            .line_height()
            / self.config.height as f32)
            * scale_factor;
        let lines = text
            .split("\n")
            .map(|s| {
                s.split("")
                    .map(|escape| match escape {
                        _ if escape == "\t" => "  ",
                        _ => escape,
                    })
                    .collect::<String>()
            })
            .collect::<Vec<String>>();

        let mut x_offset = 0.0;
        let mut y_offset = 0.0;

        let mut vertex_data = Vec::new();
        let mut index_data = Vec::new();

        for line in lines {
            for c in line.chars() {
                let region = self.get_glyph_region(c, font.clone());
                let (dim_x, dim_y) = region.dimensions();

                let w = (dim_x as f32 / self.config.width as f32) * scale_factor;
                let h = (dim_y as f32 / self.config.height as f32) * scale_factor;

                let offset_x_px = (region.offset_x() / self.config.width as f32) * scale_factor;
                let offset_y_px = (region.offset_y() / self.config.height as f32) * scale_factor;

                let glyph_left = screen_position.x() + x_offset + offset_x_px;
                let glyph_top = screen_position.y() - offset_y_px - y_offset;
                let glyph_right = glyph_left + w;
                let glyph_bottom = glyph_top - h;

                let vertices: &mut Vec<Vertex> = &mut vec![
                    Vertex::new(
                        [glyph_left, glyph_top, 0.0],
                        [region.u0(), region.v0()],
                        vert_color,
                    ),
                    Vertex::new(
                        [glyph_left, glyph_bottom, 0.0],
                        [region.u0(), region.v1()],
                        vert_color,
                    ),
                    Vertex::new(
                        [glyph_right, glyph_bottom, 0.0],
                        [region.u1(), region.v1()],
                        vert_color,
                    ),
                    Vertex::new(
                        [glyph_right, glyph_top, 0.0],
                        [region.u1(), region.v0()],
                        vert_color,
                    ),
                ];

                let buffer_size = vertex_data.len() as u16;
                let indices: &mut Vec<u16> = &mut vec![
                    buffer_size,
                    buffer_size + 1,
                    buffer_size + 3,
                    buffer_size + 1,
                    buffer_size + 2,
                    buffer_size + 3,
                ];

                x_offset += (region.advance() / self.config.width as f32) * scale_factor;

                vertex_data.append(vertices);
                index_data.append(indices);
            }

            y_offset += line_height;
            x_offset = 0.0;
        }

        (vertex_data, index_data)
    }

    fn find_priority_camera(&self, cameras: Vec<Camera2D>) -> usize {
        let mut priority = 0;
        let mut position = 0;
        for (i, camera) in cameras.iter().enumerate() {
            if camera.priority() < priority {
                priority = camera.priority();
                position = i;
            }
        }
        position
    }

    fn setup_camera<'b>(
        &mut self,
        cameras: Vec<usize>,
        scene: &'b Scene,
    ) -> (&'b Position2D, &'b Camera2D) {
        let cam = cameras
            .get(
                self.find_priority_camera(
                    cameras
                        .iter()
                        .map(|e| *scene.get_component::<Camera2D>(*e).unwrap())
                        .collect::<Vec<Camera2D>>(),
                ),
            )
            .unwrap();

        let camera_component = scene.get_component::<Camera2D>(*cam).unwrap();
        let camera_position = scene.get_component::<Transform2D>(*cam).unwrap().position();

        let camera = RenderCamera::new(
            camera_component.zoom(),
            camera_component.dimensions(),
            v3::new(
                camera_position.as_vec().x(),
                camera_position.as_vec().y(),
                0.0,
            ),
        );
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Universal Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

        let camera_bind_group_layout =
            self.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    label: Some("Universal Camera Bind Group Layout"),
                });

        let camera_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("Universal Camera Bind Group"),
        });

        self.camera = camera;
        self.camera_buffer = camera_buffer;
        self.camera_uniform = camera_uniform;
        self.camera_bind_group = camera_bind_group;

        (camera_position, camera_component)
    }

    /// A function to automatically render all the entities of the `Scene` struct.
    /// The entities must have the `Render2D` and `Transform2D` components to be rendered as well as set visible.
    pub fn render_scene_2d(&mut self, scene: &Scene) {
        let cameras = scene.get_entities_with(vec![Transform2D::type_id(), Camera2D::type_id()]);

        if cameras.is_empty() {
            return;
        }

        let entities = scene.get_entities_with(vec![Transform2D::type_id(), Render2D::type_id()]);
        let texts =
            scene.get_entities_with(vec![Transform2D::type_id(), comet_ecs::Text::type_id()]);

        self.setup_camera(cameras, scene);

        let mut vertex_buffer: Vec<Vertex> = Vec::new();
        let mut index_buffer: Vec<u16> = Vec::new();

        for entity in entities {
            let renderer_component = scene.get_component::<Render2D>(entity).unwrap();
            let transform_component = scene.get_component::<Transform2D>(entity).unwrap();

            if renderer_component.is_visible() {
                let world_position = transform_component.position().clone();
                let rotation_angle = transform_component.rotation().to_radians();

                let mut t_region: Option<&TextureRegion> = None;
                match self.get_texture_region(renderer_component.get_texture().to_string()) {
                    Some(texture_region) => {
                        t_region = Some(texture_region);
                    }
                    None => continue,
                }
                let region = t_region.unwrap();
                let (dim_x, dim_y) = region.dimensions();

                let half_width = dim_x as f32 * 0.5;
                let half_height = dim_y as f32 * 0.5;

                let buffer_size = vertex_buffer.len() as u16;

                let world_corners = [
                    (-half_width, half_height),
                    (-half_width, -half_height),
                    (half_width, -half_height),
                    (half_width, half_height),
                ];

                let cos_angle = rotation_angle.cos();
                let sin_angle = rotation_angle.sin();

                let mut rotated_world_corners = [(0.0f32, 0.0f32); 4];
                for i in 0..4 {
                    let (x, y) = world_corners[i];
                    rotated_world_corners[i] = (
                        x * cos_angle - y * sin_angle + world_position.x(),
                        x * sin_angle + y * cos_angle + world_position.y(),
                    );
                }

                let mut screen_corners = [(0.0f32, 0.0f32); 4];
                for i in 0..4 {
                    screen_corners[i] = (
                        rotated_world_corners[i].0 / self.config().width as f32,
                        rotated_world_corners[i].1 / self.config().height as f32,
                    );
                }

                vertex_buffer.append(&mut vec![
                    Vertex::new(
                        [screen_corners[0].0, screen_corners[0].1, 0.0],
                        [region.u0(), region.v0()],
                        [1.0, 1.0, 1.0, 1.0],
                    ),
                    Vertex::new(
                        [screen_corners[1].0, screen_corners[1].1, 0.0],
                        [region.u0(), region.v1()],
                        [1.0, 1.0, 1.0, 1.0],
                    ),
                    Vertex::new(
                        [screen_corners[2].0, screen_corners[2].1, 0.0],
                        [region.u1(), region.v1()],
                        [1.0, 1.0, 1.0, 1.0],
                    ),
                    Vertex::new(
                        [screen_corners[3].0, screen_corners[3].1, 0.0],
                        [region.u1(), region.v0()],
                        [1.0, 1.0, 1.0, 1.0],
                    ),
                ]);

                index_buffer.append(&mut vec![
                    0 + buffer_size,
                    1 + buffer_size,
                    3 + buffer_size,
                    1 + buffer_size,
                    2 + buffer_size,
                    3 + buffer_size,
                ]);
            }
        }

        for text in texts {
            let component = scene.get_component::<Text>(text).unwrap();
            let transform = scene.get_component::<Transform2D>(text).unwrap();

            if component.is_visible() {
                let (vertices, indices) = self.add_text_to_buffers(
                    component.content().to_string(),
                    component.font().to_string(),
                    component.font_size(),
                    p2::from_vec(transform.position().as_vec()),
                    component.color().to_wgpu(),
                );
                let draw = self
                    .draw_info
                    .iter_mut()
                    .find(|d| d.name() == &format!("{}", component.font()))
                    .unwrap();
                draw.update_vertex_buffer(&self.device, &self.queue, vertices);
                draw.update_index_buffer(&self.device, &self.queue, indices);
            }
        }

        self.set_buffers(vertex_buffer, index_buffer);
    }

    fn sort_entities_by_position(&self, entity_data: Vec<(usize, Position2D)>) -> Vec<usize> {
        let mut sorted_entities: Vec<usize> = vec![];

        let mut entity_data = entity_data.clone();
        entity_data.sort_by(|a, b| a.1.x().partial_cmp(&b.1.x()).unwrap());

        for (i, _) in entity_data {
            sorted_entities.push(i);
        }

        sorted_entities
    }

    pub fn update(&mut self) -> f32 {
        let now = Instant::now();
        self.delta_time = now.duration_since(self.last_frame_time).as_secs_f32(); // Time delta in seconds
        self.last_frame_time = now;
        self.delta_time
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Universal Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
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

            render_pass.set_pipeline(&self.universal_render_pipeline);

            for i in 0..self.draw_info.len() {
                render_pass.set_bind_group(0, self.draw_info[i].texture(), &[]);
                render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.draw_info[i].vertex_buffer().slice(..));
                render_pass.set_index_buffer(
                    self.draw_info[i].index_buffer().slice(..),
                    wgpu::IndexFormat::Uint16,
                );
                render_pass.draw_indexed(0..self.draw_info[i].num_indices(), 0, 0..1);
            }
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

impl<'a> Renderer for Renderer2D<'a> {
    fn new(window: Arc<Window>, clear_color: Option<impl Color>) -> Renderer2D<'a> {
        Self::new(window, clear_color)
    }

    fn size(&self) -> PhysicalSize<u32> {
        self.size()
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.resize(new_size)
    }

    fn update(&mut self) -> f32 {
        self.update()
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.render()
    }
}
