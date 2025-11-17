use crate::{
    camera::CameraManager,
    render_context::RenderContext,
    render_pass::{universal_clear_execute, universal_load_execute, RenderPass},
    renderer::Renderer,
};
use comet_colors::Color;
use comet_ecs::{Component, Render, Render2D, Transform2D};
use comet_log::*;
use comet_math::{m4, v2};
use comet_resources::{
    font::Font, graphic_resource_manager::GraphicResourceManager, texture_atlas::*, Texture, Vertex,
};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

const BASE_2D_SHADER_SRC: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.color = model.color;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sample_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    return sample_color * in.color;
}
"#;

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

        self.init_atlas_by_paths(paths);
    }

    pub fn init_atlas_by_paths(&mut self, paths: Vec<String>) {
        self.resource_manager.create_texture_atlas(paths);

        let texture_bind_group_layout =
            Arc::new(self.render_context.device().create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("Texture Bind Group Layout"),
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
                },
            ));

        let texture_sampler =
            self.render_context
                .device()
                .create_sampler(&wgpu::SamplerDescriptor {
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Nearest,
                    min_filter: wgpu::FilterMode::Nearest,
                    mipmap_filter: wgpu::FilterMode::Nearest,
                    lod_min_clamp: 0.0,
                    lod_max_clamp: 100.0,
                    compare: None,
                    anisotropy_clamp: 1,
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
            Box::new(universal_clear_execute),
            BASE_2D_SHADER_SRC,
            None,
            &Texture::from_image(
                self.render_context.device(),
                self.render_context.queue(),
                self.resource_manager.texture_atlas().atlas(),
                Some("Universal"),
                false,
            )
            .unwrap(),
            texture_bind_group_layout.clone(),
            texture_sampler,
            Vec::new(),
            &[camera_bind_group_layout],
        );

        let atlas_texture = Texture::from_image(
            self.render_context.device(),
            self.render_context.queue(),
            self.resource_manager.texture_atlas().atlas(),
            Some("Universal Updated"),
            false,
        )
        .unwrap();

        let texture_sampler =
            self.render_context
                .device()
                .create_sampler(&wgpu::SamplerDescriptor {
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Nearest,
                    min_filter: wgpu::FilterMode::Nearest,
                    mipmap_filter: wgpu::FilterMode::Nearest,
                    lod_min_clamp: 0.0,
                    lod_max_clamp: 100.0,
                    compare: None,
                    anisotropy_clamp: 1,
                    border_color: None,
                    ..Default::default()
                });

        let new_bind_group = Arc::new(self.render_context.device().create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&atlas_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture_sampler),
                    },
                ],
                label: Some("Universal Texture Bind Group (Updated)"),
            },
        ));

        self.render_context.resources_mut().replace_bind_group(
            "Universal".to_string(),
            0,
            new_bind_group,
        );
    }

    pub fn load_font(&mut self, path: &str, size: f32) {
        info!("Loading font from {}", path);

        let font = Font::new(path, size);
        self.resource_manager.fonts_mut().push(font);

        let fonts = self.resource_manager.fonts();
        let merged_atlas = TextureAtlas::from_fonts(fonts);
        self.resource_manager.set_font_atlas(merged_atlas.clone());

        let font_texture = Texture::from_image(
            self.render_context.device(),
            self.render_context.queue(),
            merged_atlas.atlas(),
            Some("FontAtlas"),
            false,
        )
        .expect("Failed to create GPU texture for font atlas");

        let texture_bind_group_layout =
            Arc::new(self.render_context.device().create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("Font Texture Bind Group Layout"),
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
                },
            ));

        let texture_sampler =
            self.render_context
                .device()
                .create_sampler(&wgpu::SamplerDescriptor {
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Nearest,
                    min_filter: wgpu::FilterMode::Nearest,
                    mipmap_filter: wgpu::FilterMode::Nearest,
                    ..Default::default()
                });

        let font_bind_group = Arc::new(self.render_context.device().create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&font_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture_sampler),
                    },
                ],
                label: Some("Font Bind Group"),
            },
        ));

        let camera_bind_group_layout =
            Arc::new(self.render_context.device().create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("Font Camera Bind Group Layout"),
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
            "Font".to_string(),
            Box::new(universal_load_execute),
            BASE_2D_SHADER_SRC,
            None,
            &font_texture,
            texture_bind_group_layout.clone(),
            texture_sampler,
            vec![],
            &[camera_bind_group_layout],
        );

        let camera_group_clone = {
            self.render_context
                .resources()
                .get_bind_groups("Universal")
                .and_then(|groups| groups.get(1))
                .cloned()
        };

        let resources = self.render_context.resources_mut();

        if let Some(groups) = resources.get_bind_groups("Font") {
            if groups.is_empty() {
                resources.insert_bind_group("Font".into(), font_bind_group.clone());
            } else {
                resources.replace_bind_group("Font".into(), 0, font_bind_group.clone());
            }
        } else {
            resources.insert_bind_group("Font".into(), font_bind_group.clone());
        }

        if let Some(camera_group) = camera_group_clone {
            let has_camera = resources
                .get_bind_groups("Font")
                .map(|v| v.len() > 1)
                .unwrap_or(false);

            if has_camera {
                resources.replace_bind_group("Font".into(), 1, camera_group);
            } else {
                resources.insert_bind_group("Font".into(), camera_group);
            }
        }

        info!("Font {} successfully loaded into renderer", path);
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

        if let Err(e) = self
            .resource_manager
            .load_shader(self.render_context.device(), shader_stage, shader_path)
            .or_else(|_| {
                self.resource_manager.load_shader_from_string(
                    self.render_context.device(),
                    format!("{} Shader", label.clone()).as_str(),
                    shader_path,
                )
            })
        {
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

            let shader_module = self
                .resource_manager
                .get_shader(shader_path)
                .unwrap_or_else(|| {
                    self.resource_manager
                        .get_shader(format!("{} Shader", label.clone()).as_str())
                        .unwrap()
                });
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

        if let Some(camera_layout) = extra_bind_group_layouts.get(0) {
            let device = self.render_context.device();

            let identity: [[f32; 4]; 4] = m4::IDENTITY.into();
            let cam_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} Default Camera Buffer", label)),
                contents: bytemuck::cast_slice(&[identity]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            let default_camera_bg =
                Arc::new(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("{} Default Camera Bind Group", label)),
                    layout: camera_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: cam_buffer.as_entire_binding(),
                    }],
                }));

            let resources = self.render_context.resources_mut();
            resources.insert_buffer(label.clone(), Arc::new(cam_buffer));
            resources.insert_bind_group(label.clone(), default_camera_bg);
        } else {
            warn!(
                    "Render pass '{}' created without camera layout — skipping default camera bind group",
                    label
                );
        }

        self.render_passes
            .push(RenderPass::new(label.clone(), execute));

        self.render_context
            .new_batch(label.clone(), Vec::new(), Vec::new());
        info!("Created render pass {}!", label)
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

    fn get_texture_region(&self, texture_path: &str) -> Option<&TextureRegion> {
        if !self
            .resource_manager
            .texture_atlas()
            .textures()
            .contains_key(texture_path)
        {
            #[cfg(comet_debug)]
            error!("Texture {} not found in atlas", texture_path);
        }
        self.resource_manager
            .texture_atlas()
            .textures()
            .get(texture_path)
    }

    fn get_glyph_region(&self, glyph: char, font: &str) -> &TextureRegion {
        let key = format!("{}::{}", font, glyph);

        match self.resource_manager.font_atlas().textures().get(&key) {
            Some(region) => region,
            None => {
                #[cfg(comet_debug)]
                warn!(
                    "Missing glyph for character '{}' in font '{}', using fallback.",
                    glyph, font
                );
                let fallback_key = format!("{}:: ", font);
                self.resource_manager
                    .font_atlas()
                    .textures()
                    .get(&fallback_key)
                    .unwrap_or_else(|| {
                        fatal!(
                            "No fallback glyph available (space also missing) for font '{}'",
                            font
                        )
                    })
            }
        }
    }

    pub fn precompute_text_bounds(&self, text: &str, font: &str, size: f32) -> v2 {
        let mut bounds = v2::ZERO;

        let _ =
            self.add_text_to_buffers(text, font, size, v2::ZERO, wgpu::Color::WHITE, &mut bounds);

        bounds
    }

    pub fn add_text_to_buffers(
        &self,
        text: &str,
        font: &str,
        size: f32,
        position: comet_math::v2,
        color: wgpu::Color,
        bounds: &mut comet_math::v2,
    ) -> (Vec<Vertex>, Vec<u16>) {
        let vert_color = [
            color.r as f32,
            color.g as f32,
            color.b as f32,
            color.a as f32,
        ];

        let config = self.render_context.config();

        let screen_position = comet_math::v2::new(
            position.x() / config.width as f32,
            position.y() / config.height as f32,
        );

        let font_data = self
            .resource_manager
            .fonts()
            .iter()
            .find(|f| f.name() == font)
            .unwrap_or_else(|| panic!("Font '{}' not found in resource manager", font));

        let scale_factor = size / font_data.size();
        let line_height = (font_data.line_height() / config.height as f32) * scale_factor;

        let lines = text
            .split('\n')
            .map(|s| {
                s.chars()
                    .map(|c| if c == '\t' { ' ' } else { c })
                    .collect::<String>()
            })
            .collect::<Vec<String>>();

        let mut max_line_width_px = 0.0;
        let mut total_height_px = 0.0;

        for line in &lines {
            let mut line_width_px = 0.0;
            for c in line.chars() {
                if let Some(region) = font_data.get_glyph(c) {
                    line_width_px += region.advance();
                }
            }
            if line_width_px > max_line_width_px {
                max_line_width_px = line_width_px;
            }
            total_height_px += font_data.line_height();
        }

        bounds.set_x((max_line_width_px / config.width as f32) * scale_factor);
        bounds.set_y((total_height_px / config.height as f32) * scale_factor);

        let mut x_offset = 0.0;
        let mut y_offset = 0.0;
        let mut vertex_data = Vec::new();
        let mut index_data = Vec::new();

        for line in lines {
            for c in line.chars() {
                let region = self.get_glyph_region(c, font);

                let (dim_x, dim_y) = region.dimensions();
                let w = (dim_x as f32 / config.width as f32) * scale_factor;
                let h = (dim_y as f32 / config.height as f32) * scale_factor;

                let offset_x_px = (region.offset_x() / config.width as f32) * scale_factor;
                let offset_y_px = (region.offset_y() / config.height as f32) * scale_factor;

                let glyph_left = screen_position.x() + x_offset + offset_x_px;
                let glyph_top = screen_position.y() - offset_y_px - y_offset;
                let glyph_right = glyph_left + w;
                let glyph_bottom = glyph_top - h;

                let vertices = vec![
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
                let indices = vec![
                    buffer_size,
                    buffer_size + 1,
                    buffer_size + 3,
                    buffer_size + 1,
                    buffer_size + 2,
                    buffer_size + 3,
                ];

                x_offset += (region.advance() / config.width as f32) * scale_factor;

                vertex_data.extend(vertices);
                index_data.extend(indices);
            }

            y_offset += line_height;
            x_offset = 0.0;
        }

        (vertex_data, index_data)
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

        let texts = scene.get_entities_with(vec![
            comet_ecs::Transform2D::type_id(),
            comet_ecs::Text::type_id(),
        ]);

        self.setup_camera(scene, cameras);

        let mut vertex_buffer: Vec<Vertex> = Vec::new();
        let mut index_buffer: Vec<u16> = Vec::new();

        for entity in entities {
            let renderer_component = scene.get_component::<Render2D>(entity).unwrap();
            let transform_component = scene.get_component::<Transform2D>(entity).unwrap();

            if renderer_component.is_visible() {
                let world_position = transform_component.position().clone();
                let rotation_angle = transform_component.rotation().to_radians();

                let region =
                    match self.get_texture_region(renderer_component.get_texture()) {
                        Some(r) => r,
                        None => continue,
                    };

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

                let rotated_world_corners = [
                    (
                        world_corners[0].0 * cos_angle - world_corners[0].1 * sin_angle
                            + world_position.x(),
                        world_corners[0].0 * sin_angle + world_corners[0].1 * cos_angle
                            + world_position.y(),
                    ),
                    (
                        world_corners[1].0 * cos_angle - world_corners[1].1 * sin_angle
                            + world_position.x(),
                        world_corners[1].0 * sin_angle + world_corners[1].1 * cos_angle
                            + world_position.y(),
                    ),
                    (
                        world_corners[2].0 * cos_angle - world_corners[2].1 * sin_angle
                            + world_position.x(),
                        world_corners[2].0 * sin_angle + world_corners[2].1 * cos_angle
                            + world_position.y(),
                    ),
                    (
                        world_corners[3].0 * cos_angle - world_corners[3].1 * sin_angle
                            + world_position.x(),
                        world_corners[3].0 * sin_angle + world_corners[3].1 * cos_angle
                            + world_position.y(),
                    ),
                ];

                let inv_width = 1.0 / self.render_context.config().width as f32;
                let inv_height = 1.0 / self.render_context.config().height as f32;

                let snapped_screen_corners = [
                    (
                        rotated_world_corners[0].0.round() * inv_width,
                        rotated_world_corners[0].1.round() * inv_height,
                    ),
                    (
                        rotated_world_corners[1].0.round() * inv_width,
                        rotated_world_corners[1].1.round() * inv_height,
                    ),
                    (
                        rotated_world_corners[2].0.round() * inv_width,
                        rotated_world_corners[2].1.round() * inv_height,
                    ),
                    (
                        rotated_world_corners[3].0.round() * inv_width,
                        rotated_world_corners[3].1.round() * inv_height,
                    ),
                ];

                vertex_buffer.extend_from_slice(&[
                    Vertex::new(
                        [snapped_screen_corners[0].0, snapped_screen_corners[0].1, 0.0],
                        [region.u0(), region.v0()],
                        [1.0, 1.0, 1.0, 1.0],
                    ),
                    Vertex::new(
                        [snapped_screen_corners[1].0, snapped_screen_corners[1].1, 0.0],
                        [region.u0(), region.v1()],
                        [1.0, 1.0, 1.0, 1.0],
                    ),
                    Vertex::new(
                        [snapped_screen_corners[2].0, snapped_screen_corners[2].1, 0.0],
                        [region.u1(), region.v1()],
                        [1.0, 1.0, 1.0, 1.0],
                    ),
                    Vertex::new(
                        [snapped_screen_corners[3].0, snapped_screen_corners[3].1, 0.0],
                        [region.u1(), region.v0()],
                        [1.0, 1.0, 1.0, 1.0],
                    ),
                ]);

                index_buffer.extend_from_slice(&[
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

        for text_entity in texts {
            let position = {
                let transform = scene
                    .get_component::<comet_ecs::Transform2D>(text_entity)
                    .unwrap();
                comet_math::v2::new(transform.position().x(), transform.position().y())
            };

            if let Some(text_component) = scene.get_component_mut::<comet_ecs::Text>(text_entity) {
                if !text_component.is_visible() {
                    continue;
                }

                let font = text_component.font();
                let size = text_component.font_size();
                let color = text_component.color().to_wgpu();
                let content = text_component.content();

                let mut bounds = comet_math::v2::ZERO;

                let (vertices, indices) = self.add_text_to_buffers(
                    content,
                    font,
                    size,
                    position,
                    color,
                    &mut bounds,
                );

                text_component.set_bounds(bounds);

                self.render_context
                    .update_batch_buffers("Font".to_string(), vertices, indices);
            }
        }
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

        let layout = match self
            .render_context
            .resources()
            .get_bind_group_layout("Universal")
            .and_then(|layouts| layouts.get(1))
        {
            Some(l) => l.clone(),
            None => {
                error!(
                    "Camera bind group layout missing for 'Universal' pass. Call init_atlas first."
                );
                return;
            }
        };

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
            None => resources.insert_buffer("Universal".into(), buffer.clone()),
            Some(_) => resources.replace_buffer("Universal".into(), 0, buffer.clone()),
        }

        if let Some(groups) = resources.get_bind_groups("Universal") {
            if groups.len() < 2 {
                resources.insert_bind_group("Universal".into(), bind_group.clone());
            } else {
                resources.replace_bind_group("Universal".into(), 1, bind_group.clone());
            }
        } else {
            resources.insert_bind_group("Universal".into(), bind_group.clone());
        }

        if let Some(groups) = resources.get_bind_groups("Font") {
            if groups.len() < 2 {
                resources.insert_bind_group("Font".into(), bind_group.clone());
            } else {
                resources.replace_bind_group("Font".into(), 1, bind_group.clone());
            }
        }

        if resources.get_bind_group_layout("Font").is_none() {
            #[cfg(comet_debug)]
            debug!("Font pass not initialized yet; skipping Font camera bind group setup.");
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
