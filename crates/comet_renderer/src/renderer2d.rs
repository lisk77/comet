use crate::{
    camera::RenderCamera,
    gpu_texture::GpuTexture,
    render_commands::{CameraPacket2D, Draw2D, Renderer2DCommand, Text2D},
    render_context::RenderContext,
    render_events::Renderer2DEvent,
    render_pass::{universal_clear_execute, universal_load_execute, RenderPass},
    renderer::{Renderer, RendererHandle},
};
use comet_colors::Color;
use comet_ecs::Render;
use comet_log::*;
use comet_math::{m4, v2, v3};
use comet_assets::{
    asset_root, AtlasRef, ImageRef,
    texture_atlas::*, Vertex,
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
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
    asset_provider: Arc<comet_assets::AssetProvider>,
    render_passes: Vec<RenderPass>,
    last_frame_time: std::time::Instant,
    delta_time: f32,
    event_sender: flume::Sender<Renderer2DEvent>,
}

pub struct RenderHandle2D {
    command_sender: flume::Sender<Renderer2DCommand>,
    event_receiver: flume::Receiver<Renderer2DEvent>,
    last_size: Option<PhysicalSize<u32>>,
}

impl RenderHandle2D {
    pub fn init_atlas(&mut self) {
        let _ = self.command_sender.send(Renderer2DCommand::InitAtlas);
    }

    pub fn init_atlas_by_paths(&mut self, paths: Vec<String>) {
        let _ = self
            .command_sender
            .send(Renderer2DCommand::InitAtlasFromPaths(paths));
    }

    pub fn load_font(&mut self, path: &str, size: f32) {
        let _ = self
            .command_sender
            .send(Renderer2DCommand::LoadFont(path.to_string(), size));
    }

    fn resolve_atlas_ref(&mut self, path: &'static str) -> Option<AtlasRef> {
        let _ = self
            .command_sender
            .send(Renderer2DCommand::ResolveAtlasRef(path));
        self.recv_matching_event(Duration::from_millis(25), |event| {
            matches!(event, Renderer2DEvent::AtlasRef(_))
        })
        .and_then(|event| match event {
            Renderer2DEvent::AtlasRef(atlas_ref) => atlas_ref,
            _ => None,
        })
    }

    pub fn size(&mut self) -> PhysicalSize<u32> {
        let _ = self.command_sender.send(Renderer2DCommand::Size);
        self.recv_matching_event(Duration::from_millis(25), |event| {
            matches!(event, Renderer2DEvent::Size(_))
        })
        .and_then(|e| match e {
            Renderer2DEvent::Size(size) => Some(size),
            _ => None,
        })
        .map(|size| {
            self.last_size = Some(size);
            size
        })
        .unwrap_or_else(|| self.last_size.unwrap_or(PhysicalSize::new(0, 0)))
    }

    pub fn scale_factor(&mut self) -> f64 {
        let _ = self.command_sender.send(Renderer2DCommand::ScaleFactor);
        self.recv_matching_event(Duration::from_millis(25), |event| {
            matches!(event, Renderer2DEvent::ScaleFactor(_))
        })
        .and_then(|e| match e {
            Renderer2DEvent::ScaleFactor(factor) => Some(factor),
            _ => None,
        })
        .unwrap_or(1.0)
    }

    pub fn precompute_text_bounds(&mut self, text: &str, font_path: &str, font_size: f32) -> v2 {
        let _ = self
            .command_sender
            .send(Renderer2DCommand::PrecomputedTextBounds {
                text: text.to_string(),
                font_path: font_path.to_string(),
                font_size,
            });
        self.recv_matching_event(Duration::from_secs(5), |event| {
            matches!(event, Renderer2DEvent::PrecomputedTextBounds { .. })
        })
        .and_then(|e| match e {
            Renderer2DEvent::PrecomputedTextBounds { width, height } => {
                Some(v2::new(width, height))
            }
            _ => None,
        })
        .unwrap_or(v2::ZERO)
    }

    pub fn poll_events(&mut self) {
        while let Ok(event) = self.event_receiver.try_recv() {
            if let Renderer2DEvent::Size(size) = event {
                self.last_size = Some(size);
            }
        }
    }

    fn recv_matching_event<F>(&mut self, timeout: Duration, predicate: F) -> Option<Renderer2DEvent>
    where
        F: Fn(&Renderer2DEvent) -> bool,
    {
        let deadline = Instant::now() + timeout;

        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return None;
            }

            match self.event_receiver.recv_timeout(remaining) {
                Ok(event) => {
                    if let Renderer2DEvent::Size(size) = event {
                        self.last_size = Some(size);
                    }
                    if predicate(&event) {
                        return Some(event);
                    }
                }
                Err(flume::RecvTimeoutError::Timeout) => return None,
                Err(flume::RecvTimeoutError::Disconnected) => return None,
            }
        }
    }

    pub fn render_scene_2d(&mut self, scene: &mut comet_ecs::Scene) {
        let mut selected_camera: Option<([f32; 2], f32, f32, [f32; 2], u8)> = None;
        for (transform, camera) in scene
            .query::<(&comet_ecs::Transform2D, &comet_ecs::Camera2D), ()>()
            .iter()
        {
            let should_replace = selected_camera
                .as_ref()
                .is_none_or(|(_, _, _, _, current_priority)| camera.priority() < *current_priority);
            if should_replace {
                selected_camera = Some((
                    [transform.position().x(), transform.position().y()],
                    transform.rotation().to_degrees(),
                    camera.zoom(),
                    [camera.dimensions().x(), camera.dimensions().y()],
                    camera.priority(),
                ));
            }
        }
        let Some((camera_pos, camera_rot, camera_zoom, camera_dims, camera_priority)) =
            selected_camera
        else {
            return;
        };

        let mut draws = Vec::new();
        for (transform, render) in scene
            .query_mut::<(&comet_ecs::Transform2D, &mut comet_ecs::Render2D), ()>()
            .iter()
        {
            let atlas_ref = match render.texture() {
                ImageRef::Atlas(atlas_ref) => atlas_ref,
                ImageRef::Unresolved(path) => {
                    let Some(atlas_ref) = self.resolve_atlas_ref(path) else {
                        continue;
                    };
                    render.set_texture(ImageRef::Atlas(atlas_ref));
                    atlas_ref
                }
            };

            draws.push(Draw2D {
                position: [transform.position().x(), transform.position().y()],
                rotation_deg: transform.rotation().to_degrees(),
                scale: [1.0, 1.0],
                texture: atlas_ref,
                draw_index: render.draw_index(),
                visible: render.is_visible(),
            });
        }
        draws.sort_by_key(|draw| draw.draw_index);

        let mut texts = Vec::new();
        for (transform, text) in scene
            .query::<(&comet_ecs::Transform2D, &comet_ecs::Text), ()>()
            .iter()
        {
            if !text.is_visible() {
                continue;
            }
            let color = text.color().to_wgpu();
            texts.push(Text2D {
                position: [transform.position().x(), transform.position().y()],
                content: text.content().to_string(),
                font: text.font(),
                size: text.font_size(),
                color: [
                    color.r as f32,
                    color.g as f32,
                    color.b as f32,
                    color.a as f32,
                ],
                visible: true,
            });
        }

        let camera_packet = CameraPacket2D {
            position: camera_pos,
            rotation_deg: camera_rot,
            zoom: camera_zoom,
            dimensions: camera_dims,
            priority: camera_priority,
        };

        let _ =
            self.command_sender
                .send(Renderer2DCommand::SubmitFrame(camera_packet, draws, texts));
    }
}

impl RendererHandle for RenderHandle2D {
    type Command = Renderer2DCommand;
    type Event = Renderer2DEvent;

    fn new(sender: flume::Sender<Self::Command>, receiver: flume::Receiver<Self::Event>) -> Self {
        Self {
            command_sender: sender,
            event_receiver: receiver,
            last_size: None,
        }
    }

    fn poll_event(&self) -> Option<Renderer2DEvent> {
        self.event_receiver.try_recv().ok()
    }
}

impl<'a> Renderer2D<'a> {
    pub fn init_atlas(&mut self) {
        let texture_path = "res://textures/".to_string();
        let mut paths: Vec<String> = Vec::new();

        for path in std::fs::read_dir(asset_root().join("textures")).unwrap() {
            paths.push(texture_path.clone() + path.unwrap().file_name().to_str().unwrap());
        }

        self.init_atlas_by_paths(paths);
    }

    pub fn init_atlas_by_paths(&mut self, paths: Vec<String>) {
        // Load TextureAtlas from paths
        let texture_atlas = match comet_assets::TextureAtlas::from_texture_paths(paths.clone()) {
            atlas => {
                info!("Loaded texture atlas from paths: {} images", paths.len());
                atlas
            }
        };

        // Store atlas metadata in resources for texture region lookups
        if let Some(handle) = self.asset_provider.add_texture_atlas(texture_atlas.clone()) {
            self.render_context.resources_mut().insert_asset_atlas_handle("atlas".to_string(), handle);
        } else {
            error!("Failed to add texture atlas to asset provider");
            return;
        }

        // Convert to GPU texture
        let gpu_texture = match GpuTexture::from_dynamic_image(
            self.render_context.device(),
            self.render_context.queue(),
            texture_atlas.atlas(),
            Some("Atlas"),
            false,
        ) {
            Ok(tex) => tex,
            Err(e) => {
                error!("Failed to convert atlas to GPU texture: {}", e);
                return;
            }
        };

        // Wrap in Arc and cache GPU texture  
        let gpu_texture_arc = Arc::new(gpu_texture);
        self.render_context
            .resources_mut()
            .insert_gpu_texture("atlas".to_string(), gpu_texture_arc.clone());

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
            &(*gpu_texture_arc),
            texture_bind_group_layout.clone(),
            texture_sampler,
            Vec::new(),
            &[camera_bind_group_layout],
        );

        let new_bind_group = Arc::new({
            let device = self.render_context.device();
            let sampler = self.render_context.resources().get_sampler("Universal")
                .expect("Universal sampler missing after new_render_pass");
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&gpu_texture_arc.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
                label: Some("Universal Texture Bind Group (Updated)"),
            })
        });

        self.render_context.resources_mut().replace_bind_group(
            "Universal".to_string(),
            0,
            new_bind_group,
        );
    }

    pub fn load_font(&mut self, path: &str, size: f32) {
        info!("Loading font from {}", path);

        // Load font and store it in the asset provider for metric lookups
        let font = comet_assets::Font::new(path, size);
        let font_handle = match self.asset_provider.add_font(font) {
            Some(h) => h,
            None => {
                error!("Failed to add font '{}' to asset provider", path);
                return;
            }
        };
        self.render_context.resources_mut().insert_asset_font_handle(path.to_string(), font_handle);

        // Build the merged font atlas using a reference to the stored font
        let font_atlas = self.asset_provider.with_font(font_handle, |f| {
            comet_assets::TextureAtlas::from_fonts(std::slice::from_ref(f))
        }).unwrap_or_else(|| {
            error!("Font not accessible after adding to asset provider");
            comet_assets::TextureAtlas::empty()
        });

        // Store atlas handle in asset provider
        if let Some(handle) = self.asset_provider.add_texture_atlas(font_atlas.clone()) {
            self.render_context.resources_mut().insert_asset_atlas_handle("font_atlas".to_string(), handle);
        } else {
            error!("Failed to add font atlas to asset provider");
            return;
        }

        let font_texture = match GpuTexture::from_dynamic_image(
            self.render_context.device(),
            self.render_context.queue(),
            font_atlas.atlas(),
            Some("FontAtlas"),
            false,
        ) {
            Ok(tex) => tex,
            Err(e) => {
                error!("Failed to create GPU texture for font atlas: {}", e);
                return;
            }
        };

        // Wrap in Arc and cache GPU texture
        let font_texture_arc = Arc::new(font_texture);
        self.render_context
            .resources_mut()
            .insert_gpu_texture("font_atlas".to_string(), font_texture_arc.clone());

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
                        resource: wgpu::BindingResource::TextureView(&font_texture_arc.view),
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
            &(*font_texture_arc),
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

        info!("Font {} successfully loaded into renderer (cached)", path);
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
        _shader_stage: Option<wgpu::naga::ShaderStage>,
        texture: &GpuTexture,
        texture_bind_group_layout: Arc<wgpu::BindGroupLayout>,
        texture_sampler: wgpu::Sampler,
        bind_groups: Vec<Arc<wgpu::BindGroup>>,
        extra_bind_group_layouts: &[Arc<wgpu::BindGroupLayout>],
    ) {
        info!("Creating render pass {}", label);

        let shader_module = self.render_context.device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("{} Shader", label)),
            source: wgpu::ShaderSource::Wgsl(shader_path.into()),
        });

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

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&format!("{} Render Pipeline", label)),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: "vs_main",
                    buffers: &[comet_assets::Vertex::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
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

    fn get_texture_region(&self, texture: AtlasRef) -> TextureRegion {
        texture.region()
    }

    fn get_glyph_region(&self, glyph: char, font: &str) -> TextureRegion {
        let key = format!("{}::{}", font, glyph);

        // Query font atlas from asset provider using stored handle
        if let Some(handle) = self.render_context.resources().get_asset_atlas_handle("font_atlas") {
            self.asset_provider.with_texture_atlas(handle, |atlas| {
                match atlas.textures().get(&key) {
                    Some(region) => *region,
                    None => {
                        #[cfg(feature = "comet_debug")]
                        warn!(
                            "Missing glyph for character '{}' in font '{}', using fallback.",
                            glyph, font
                        );
                        let fallback_key = format!("{}:: ", font);
                        atlas.textures().get(&fallback_key).copied().unwrap_or_else(|| {
                            fatal!(
                                "No fallback glyph available (space also missing) for font '{}'",
                                font
                            )
                        })
                    }
                }
            }).unwrap_or_else(|| {
                fatal!("Failed to access font atlas from asset provider");
            })
        } else {
            fatal!("Font atlas not loaded yet - call load_font() first");
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

        let font_handle = self.render_context.resources().get_asset_font_handle(font)
            .unwrap_or_else(|| panic!("Font '{}' not found", font));
        let font_data = self.asset_provider.with_font(font_handle, |f| f.clone())
            .unwrap_or_else(|| panic!("Font '{}' not accessible via asset provider", font));

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

        bounds.set_x(max_line_width_px * scale_factor);
        bounds.set_y(total_height_px * scale_factor);

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

    pub fn submit_frame(
        &mut self,
        camera: CameraPacket2D,
        mut draws: Vec<Draw2D>,
        texts: Vec<Text2D>,
    ) {
        self.setup_camera_from_packet(camera);

        draws.sort_by_key(|draw| draw.draw_index);

        let mut vertex_buffer: Vec<Vertex> = Vec::new();
        let mut index_buffer: Vec<u16> = Vec::new();

        for draw in draws {
            if !draw.visible {
                continue;
            }

            let region = self.get_texture_region(draw.texture);

            let (dim_x, dim_y) = region.dimensions();
            let half_width = dim_x as f32 * 0.5 * draw.scale[0];
            let half_height = dim_y as f32 * 0.5 * draw.scale[1];

            let buffer_size = vertex_buffer.len() as u16;

            let world_corners = [
                (-half_width, half_height),
                (-half_width, -half_height),
                (half_width, -half_height),
                (half_width, half_height),
            ];

            let rotation_angle = draw.rotation_deg.to_radians();
            let cos_angle = rotation_angle.cos();
            let sin_angle = rotation_angle.sin();

            let rotated_world_corners = [
                (
                    world_corners[0].0 * cos_angle - world_corners[0].1 * sin_angle
                        + draw.position[0],
                    world_corners[0].0 * sin_angle
                        + world_corners[0].1 * cos_angle
                        + draw.position[1],
                ),
                (
                    world_corners[1].0 * cos_angle - world_corners[1].1 * sin_angle
                        + draw.position[0],
                    world_corners[1].0 * sin_angle
                        + world_corners[1].1 * cos_angle
                        + draw.position[1],
                ),
                (
                    world_corners[2].0 * cos_angle - world_corners[2].1 * sin_angle
                        + draw.position[0],
                    world_corners[2].0 * sin_angle
                        + world_corners[2].1 * cos_angle
                        + draw.position[1],
                ),
                (
                    world_corners[3].0 * cos_angle - world_corners[3].1 * sin_angle
                        + draw.position[0],
                    world_corners[3].0 * sin_angle
                        + world_corners[3].1 * cos_angle
                        + draw.position[1],
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
                    [
                        snapped_screen_corners[0].0,
                        snapped_screen_corners[0].1,
                        0.0,
                    ],
                    [region.u0(), region.v0()],
                    [1.0, 1.0, 1.0, 1.0],
                ),
                Vertex::new(
                    [
                        snapped_screen_corners[1].0,
                        snapped_screen_corners[1].1,
                        0.0,
                    ],
                    [region.u0(), region.v1()],
                    [1.0, 1.0, 1.0, 1.0],
                ),
                Vertex::new(
                    [
                        snapped_screen_corners[2].0,
                        snapped_screen_corners[2].1,
                        0.0,
                    ],
                    [region.u1(), region.v1()],
                    [1.0, 1.0, 1.0, 1.0],
                ),
                Vertex::new(
                    [
                        snapped_screen_corners[3].0,
                        snapped_screen_corners[3].1,
                        0.0,
                    ],
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

        self.render_context.update_batch_buffers(
            "Universal".to_string(),
            vertex_buffer,
            index_buffer,
        );

        for text in texts {
            if !text.visible {
                continue;
            }

            let position = v2::new(text.position[0], text.position[1]);
            let color = wgpu::Color {
                r: text.color[0] as f64,
                g: text.color[1] as f64,
                b: text.color[2] as f64,
                a: text.color[3] as f64,
            };

            let mut bounds = v2::ZERO;
            let (vertices, indices) = self.add_text_to_buffers(
                &text.content,
                text.font,
                text.size,
                position,
                color,
                &mut bounds,
            );

            self.render_context
                .update_batch_buffers("Font".to_string(), vertices, indices);
        }
    }

    fn setup_camera_from_packet(&mut self, camera: CameraPacket2D) {
        let render_camera = RenderCamera::new(
            camera.zoom,
            v2::new(camera.dimensions[0], camera.dimensions[1]),
            v3::new(camera.position[0], camera.position[1], 0.0),
        );

        let mut camera_uniform = crate::camera::CameraUniform::new();
        camera_uniform.update_view_proj(&render_camera);

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
            #[cfg(feature = "comet_debug")]
            debug!("Font pass not initialized yet; skipping Font camera bind group setup.");
        }
    }
}

impl<'a> Renderer for Renderer2D<'a> {
    type Handle = RenderHandle2D;

    fn new(
        window: Arc<Window>,
        clear_color: Option<impl Color>,
        event_sender: flume::Sender<Renderer2DEvent>,
        asset_provider: Arc<comet_assets::AssetProvider>,
    ) -> Self {
        Self {
            render_context: RenderContext::new(window, clear_color),
            asset_provider,
            render_passes: Vec::new(),
            last_frame_time: std::time::Instant::now(),
            delta_time: 0.0,
            event_sender,
        }
    }

    fn apply_command(&mut self, command: <Self::Handle as RendererHandle>::Command) {
        match command {
            Renderer2DCommand::Clear => {}
            Renderer2DCommand::InitAtlas => self.init_atlas(),
            Renderer2DCommand::InitAtlasFromPaths(paths) => self.init_atlas_by_paths(paths),
            Renderer2DCommand::ResolveAtlasRef(path) => {
                let atlas_ref = self.render_context
                    .resources()
                    .get_asset_atlas_handle("atlas")
                    .and_then(|handle| {
                        self.asset_provider.with_texture_atlas(handle, |atlas| {
                            atlas.textures()
                                .get(path)
                                .copied()
                                .map(|region| AtlasRef::new(region, handle))
                        })
                        .flatten()
                    });
                let _ = self.event_sender.send(Renderer2DEvent::AtlasRef(atlas_ref));
            }
            Renderer2DCommand::Size => {
                let _ = self.event_sender.send(Renderer2DEvent::Size(self.size()));
            }
            Renderer2DCommand::ScaleFactor => {
                let _ = self
                    .event_sender
                    .send(Renderer2DEvent::ScaleFactor(self.scale_factor()));
            }
            Renderer2DCommand::LoadFont(font_path, font_size) => {
                self.load_font(font_path.as_str(), font_size)
            }
            Renderer2DCommand::PrecomputedTextBounds {
                text,
                font_path,
                font_size,
            } => {
                let bounds = self.precompute_text_bounds(&text, font_path.as_str(), font_size);
                let _ = self
                    .event_sender
                    .send(Renderer2DEvent::PrecomputedTextBounds {
                        width: bounds.x(),
                        height: bounds.y(),
                    });
            }
            Renderer2DCommand::SubmitFrame(camera, draws, texts) => {
                self.submit_frame(camera, draws, texts)
            }
        }
    }

    fn window(&self) -> &Window {
        self.render_context.window()
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
