use crate::renderer::Renderer;
use crate::{
    camera::{CameraManager, RenderCamera},
    render_context::RenderContext,
    render_pass::{universal_execute, RenderPass},
};
use comet_colors::Color;
use comet_ecs::{Camera, Camera2D, Component, Render, Render2D, Transform2D};
use comet_log::{debug, error, info};
use comet_math::v3;
use comet_resources::{
    graphic_resource_manager::GraphicResourceManager, texture_atlas::TextureRegion, Texture, Vertex,
};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

pub struct Renderer2D<'a> {
    render_context: RenderContext<'a>,
    resource_manager: GraphicResourceManager,
    camera_manager: CameraManager,
    render_passes: Vec<RenderPass>,
    last_frame_time: std::time::Instant,
    delta_time: f32,
}

impl<'a> Renderer2D<'a> {
    pub fn init_atlas(&mut self) {
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

        self.resource_manager.create_texture_atlas(paths.clone());
        self.init_atlas_by_paths(paths);
    }

    pub fn init_atlas_by_paths(&mut self, paths: Vec<String>) {
        self.resource_manager.create_texture_atlas(paths);

        let texture_bind_group_layout =
            Arc::new(self.render_context.device().create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("Texture Bind Group Layout"),
                    entries: &[
                        // Texture view (binding = 0)
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
                        // Sampler (binding = 1)
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                },
            ));

        let texture_sampler =
            self.render_context
                .device()
                .create_sampler(&wgpu::SamplerDescriptor {
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

        let camera_bind_group_layout =
            Arc::new(self.render_context.device().create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("Camera Bind Group Layout"),
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
                },
            ));

        self.new_render_pass(
            "Universal".to_string(),
            Box::new(universal_execute),
            "res/shaders/base2d.wgsl",
            None,
            &Texture::from_image(
                self.render_context.device(),
                self.render_context.queue(),
                self.resource_manager.texture_atlas().atlas(),
                Some("Universal"),
                false,
            )
            .unwrap(),
            texture_bind_group_layout,
            texture_sampler,
            Vec::new(),
            &[camera_bind_group_layout],
        );
    }

    pub fn new_render_pass(
        &mut self,
        label: String,
        execute: Box<
            dyn Fn(String, &mut RenderContext, &mut wgpu::CommandEncoder, &wgpu::TextureView)
                + Send
                + Sync,
        >,
        shader_path: &str,
        shader_stage: Option<wgpu::naga::ShaderStage>,
        texture: &Texture,
        texture_bind_group_layout: Arc<wgpu::BindGroupLayout>,
        texture_sampler: wgpu::Sampler,
        bind_groups: Vec<Arc<wgpu::BindGroup>>,
        extra_bind_group_layouts: &[Arc<wgpu::BindGroupLayout>],
    ) {
        info!("Creating render pass {}", label);

        if let Err(e) = self.resource_manager.load_shader(
            shader_stage,
            shader_path,
            self.render_context.device(),
        ) {
            error!("Aborting render pass creation: {}", e);
            return;
        }

        let texture_bind_group = Arc::new({
            let device = self.render_context.device();
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture_sampler),
                    },
                ],
                label: Some(&format!("{} Texture Bind Group", label)),
            })
        });

        let render_pipeline = {
            let device = self.render_context.device();

            let mut bind_layout_refs: Vec<&wgpu::BindGroupLayout> = Vec::new();
            bind_layout_refs.push(&texture_bind_group_layout);
            for layout in extra_bind_group_layouts {
                bind_layout_refs.push(layout);
            }

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{} Pipeline Layout", label)),
                bind_group_layouts: &bind_layout_refs,
                push_constant_ranges: &[],
            });

            let shader_module = self.resource_manager.get_shader(shader_path).unwrap();
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&format!("{} Render Pipeline", label)),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: shader_module,
                    entry_point: "vs_main",
                    buffers: &[comet_resources::Vertex::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader_module,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: self.render_context.config().format,
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
            })
        };

        self.render_context
            .insert_pipeline(label.clone(), render_pipeline);

        {
            let resources = self.render_context.resources_mut();
            resources.insert_bind_group(label.clone(), texture_bind_group);
            for group in bind_groups {
                resources.insert_bind_group(label.clone(), group);
            }
            resources.insert_bind_group_layout(label.clone(), texture_bind_group_layout);
            for layout in extra_bind_group_layouts {
                resources.insert_bind_group_layout(label.clone(), layout.clone());
            }
            resources.insert_sampler(label.clone(), texture_sampler);
        }

        self.render_passes
            .push(RenderPass::new(label.clone(), execute));

        self.render_context.new_batch(label, Vec::new(), Vec::new());
    }

    fn get_project_root() -> std::io::Result<std::path::PathBuf> {
        let path = std::env::current_dir()?;
        let mut path_ancestors = path.as_path().ancestors();

        while let Some(p) = path_ancestors.next() {
            let has_cargo = std::fs::read_dir(p)?
                .into_iter()
                .any(|p| p.unwrap().file_name() == std::ffi::OsString::from("Cargo.lock"));
            if has_cargo {
                return Ok(std::path::PathBuf::from(p));
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Ran out of places to find Cargo.toml",
        ))
    }

    pub fn render_scene_2d(&mut self, scene: &mut comet_ecs::Scene) {
        let cameras = scene.get_entities_with(vec![
            comet_ecs::Transform2D::type_id(),
            comet_ecs::Camera2D::type_id(),
        ]);

        if cameras.is_empty() {
            return;
        }

        let mut entities = scene.get_entities_with(vec![
            comet_ecs::Transform2D::type_id(),
            comet_ecs::Render2D::type_id(),
        ]);

        entities.sort_by(|&a, &b| {
            let ra = scene.get_component::<comet_ecs::Render2D>(a).unwrap();
            let rb = scene.get_component::<comet_ecs::Render2D>(b).unwrap();
            ra.draw_index().cmp(&rb.draw_index())
        });

        self.setup_camera(scene, cameras);

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

                let scale = renderer_component.scale();
                let half_width = dim_x as f32 * 0.5 * scale.x();
                let half_height = dim_y as f32 * 0.5 * scale.y();

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
                        rotated_world_corners[i].0 / self.render_context.config().width as f32,
                        rotated_world_corners[i].1 / self.render_context.config().height as f32,
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

        self.render_context.update_batch_buffers(
            "Universal".to_string(),
            vertex_buffer,
            index_buffer,
        );
    }

    pub fn get_texture_region(&self, texture_path: String) -> Option<&TextureRegion> {
        if !self
            .resource_manager
            .texture_atlas()
            .textures()
            .contains_key(&texture_path)
        {
            error!("Texture {} not found in atlas", &texture_path);
        }
        self.resource_manager
            .texture_atlas()
            .textures()
            .get(&texture_path)
    }

    fn setup_camera(&mut self, scene: &comet_ecs::Scene, cameras: Vec<usize>) {
        if cameras.is_empty() {
            return;
        }

        self.camera_manager.update_from_scene(scene, cameras);

        if !self.camera_manager.has_active_camera() {
            error!("No active camera found");
            return;
        }

        let active_camera = self.camera_manager.get_camera();

        let mut camera_uniform = crate::camera::CameraUniform::new();
        camera_uniform.update_view_proj(active_camera);

        let buffer = Arc::new(self.render_context.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Uniform Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        ));

        let layout = self
            .render_context
            .resources()
            .get_bind_group_layout("Universal")
            .unwrap()[1]
            .clone();

        let bind_group = Arc::new(self.render_context.device().create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label: Some("Camera Bind Group"),
            },
        ));

        let resources = self.render_context.resources_mut();

        match resources.get_buffer("Universal") {
            None => resources.insert_buffer("Universal".to_string(), buffer),
            Some(_) => resources.replace_buffer("Universal".to_string(), 0, buffer),
        }

        if let Some(v) = resources.get_bind_groups("Universal") {
            if v.len() < 2 {
                resources.insert_bind_group("Universal".to_string(), bind_group);
            } else {
                resources.replace_bind_group("Universal".to_string(), 1, bind_group);
            }
        }
    }
}

impl<'a> Renderer for Renderer2D<'a> {
    fn new(window: Arc<Window>, clear_color: Option<impl Color>) -> Self {
        Self {
            render_context: RenderContext::new(window, clear_color),
            resource_manager: GraphicResourceManager::new(),
            camera_manager: CameraManager::new(),
            render_passes: Vec::new(),
            last_frame_time: std::time::Instant::now(),
            delta_time: 0.0,
        }
    }

    fn size(&self) -> PhysicalSize<u32> {
        self.render_context.size()
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.render_context.set_size(new_size);
            self.render_context.config_mut().width = new_size.width;
            self.render_context.config_mut().height = new_size.height;
            self.render_context.configure_surface();
        }
    }

    fn scale_factor(&self) -> f64 {
        self.render_context.scale_factor()
    }

    fn set_scale_factor(&mut self, scale_factor: f64) {
        self.render_context.set_scale_factor(scale_factor);
    }

    fn update(&mut self) -> f32 {
        let now = std::time::Instant::now();
        self.delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        self.delta_time
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.render_context.surface().get_current_texture()?;
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.render_context
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        for pass in &self.render_passes {
            let label = pass.label.clone();
            (pass.execute)(label, &mut self.render_context, &mut encoder, &output_view);
        }

        self.render_context
            .queue()
            .submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())
    }
}
