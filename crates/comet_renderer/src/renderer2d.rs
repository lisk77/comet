use crate::{
    camera::RenderCamera,
    gpu_texture::GpuTexture,
    render_commands::{CameraPacket2D, Draw2D, Renderer2DCommand, Text2D},
    render_context::RenderContext,
    render_events::Renderer2DEvent,
    render_pass::{universal_clear_execute, universal_load_execute, PassCache, PassOutput, RenderPass},
    Vertex,
};
use comet_colors::Color;
use comet_ecs::Render;
use comet_app::{App, Module};
use comet_ecs::EcsModuleExt;
use comet_macros::module;
use comet_window::renderer::{Renderer, RendererHandle};
use comet_log::*;
use comet_math::{m4, v2, v3};
use comet_assets::{
    AtlasRef, ImageRef,
    texture_atlas::*,
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Hash, PartialEq, Eq)]
struct FontKey {
    index: u32,
    generation: u32,
    size_bits: u32,
}
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

pub struct Renderer2D {
    render_context: RenderContext,
    asset_provider: comet_assets::AssetProvider,
    render_passes: Vec<RenderPass>,
    execution_order: Vec<String>,
    graph_dirty: bool,
    last_frame_time: std::time::Instant,
    delta_time: f32,
    event_sender: flume::Sender<Renderer2DEvent>,
    font_cache: std::collections::HashMap<FontKey, f32>,
    accumulated_font_glyphs: Vec<comet_assets::GlyphData>,
}

pub struct RenderHandle2D {
    command_sender: flume::Sender<Renderer2DCommand>,
    event_receiver: flume::Receiver<Renderer2DEvent>,
    last_size: Option<PhysicalSize<u32>>,
    pending_atlas_rebuild: bool,
}

#[module]
impl RenderHandle2D {
    fn resolve_atlas_ref(&mut self, path: &'static str) -> Option<(AtlasRef, Option<comet_assets::Asset<comet_assets::Image>>)> {
        let _ = self
            .command_sender
            .send(Renderer2DCommand::ResolveAtlasRef(path));
        self.recv_matching_event(Duration::from_millis(5000), |event| {
            matches!(event, Renderer2DEvent::AtlasRef(..))
        })
        .and_then(|event| match event {
            Renderer2DEvent::AtlasRef(Some(atlas_ref), image_handle) => Some((atlas_ref, image_handle)),
            _ => None,
        })
    }

    fn ensure_handle_in_atlas(&mut self, handle: comet_assets::Asset<comet_assets::Image>) -> Option<AtlasRef> {
        let _ = self
            .command_sender
            .send(Renderer2DCommand::EnsureHandleInAtlas(handle));
        self.recv_matching_event(Duration::from_millis(5000), |event| {
            matches!(event, Renderer2DEvent::AtlasRef(..))
        })
        .and_then(|event| match event {
            Renderer2DEvent::AtlasRef(atlas_ref, _) => atlas_ref,
            _ => None,
        })
    }

    pub fn size(&mut self) -> PhysicalSize<u32> {
        let _ = self.command_sender.send(Renderer2DCommand::Size);
        self.recv_matching_event(Duration::from_millis(5000), |event| {
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
        self.recv_matching_event(Duration::from_millis(5000), |event| {
            matches!(event, Renderer2DEvent::ScaleFactor(_))
        })
        .and_then(|e| match e {
            Renderer2DEvent::ScaleFactor(factor) => Some(factor),
            _ => None,
        })
        .unwrap_or(1.0)
    }

    pub fn precompute_text_bounds(&mut self, text: &str, font: comet_assets::Asset<comet_assets::Font>, font_size: f32) -> v2 {
        let _ = self
            .command_sender
            .send(Renderer2DCommand::PrecomputedTextBounds {
                text: text.to_string(),
                font,
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
            match event {
                Renderer2DEvent::Size(size) => self.last_size = Some(size),
                Renderer2DEvent::AtlasRebuilt => self.pending_atlas_rebuild = true,
                _ => {}
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
                    match &event {
                        Renderer2DEvent::Size(size) => self.last_size = Some(*size),
                        Renderer2DEvent::AtlasRebuilt => self.pending_atlas_rebuild = true,
                        _ => {}
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

    pub fn add_render_pass(
        &mut self,
        label: String,
        inputs: Vec<&PassOutput>,
        output: Option<String>,
        output_format: Option<wgpu::TextureFormat>,
        shader_src: String,
        clear: Option<wgpu::Color>,
    ) -> Option<PassOutput> {
        let desc = crate::render_commands::PassDescriptor {
            label,
            inputs: inputs.iter().map(|p| p.0.clone()).collect(),
            output,
            output_format,
            shader_src,
            clear,
        };
        let _ = self.command_sender.send(Renderer2DCommand::AddRenderPass(desc));
        self.recv_matching_event(Duration::from_millis(5000), |e| {
            matches!(e, Renderer2DEvent::PassAdded(_))
        })
        .and_then(|e| match e {
            Renderer2DEvent::PassAdded(handle) => Some(handle),
            _ => None,
        })
    }

    pub fn remove_render_pass(&mut self, output: PassOutput) {
        let _ = self.command_sender.send(Renderer2DCommand::RemoveRenderPass(output.0));
        let _ = self.recv_matching_event(Duration::from_millis(5000), |e| {
            matches!(e, Renderer2DEvent::PassRemoved)
        });
    }

    pub fn set_pass_output(&mut self, label: &str, output: Option<String>) -> Option<PassOutput> {
        let _ = self.command_sender.send(Renderer2DCommand::SetPassOutput(label.to_string(), output.clone()));
        let _ = self.recv_matching_event(Duration::from_millis(5000), |e| {
            matches!(e, Renderer2DEvent::PassOutputSet)
        });
        output.map(PassOutput)
    }

}

impl RenderHandle2D {
    pub fn render_scene_2d(&mut self, scene: &mut comet_ecs::Scene) {
        self.poll_events();
        if self.pending_atlas_rebuild {
            self.pending_atlas_rebuild = false;
            for (_, render) in scene
                .query_mut::<(&comet_ecs::Transform2D, &mut comet_ecs::Render2D), ()>()
                .iter()
            {
                if let ImageRef::ResolvedHandle(h, _) = render.texture() {
                    render.set_image_ref(ImageRef::Handle(h));
                }
            }
        }

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
        let mut referenced_handles = Vec::new();
        for (transform, render) in scene
            .query_mut::<(&comet_ecs::Transform2D, &mut comet_ecs::Render2D), ()>()
            .iter()
        {
            let atlas_ref = match render.texture() {
                ImageRef::Atlas(atlas_ref) => atlas_ref,
                ImageRef::Unresolved(path) => {
                    let Some((atlas_ref, image_handle)) = self.resolve_atlas_ref(path) else {
                        continue;
                    };
                    if let Some(handle) = image_handle {
                        render.set_image_ref(ImageRef::ResolvedHandle(handle, atlas_ref));
                        referenced_handles.push(handle);
                    } else {
                        render.set_image_ref(ImageRef::Atlas(atlas_ref));
                    }
                    atlas_ref
                }
                ImageRef::Handle(handle) => {
                    let Some(atlas_ref) = self.ensure_handle_in_atlas(handle) else {
                        continue;
                    };
                    render.set_image_ref(ImageRef::ResolvedHandle(handle, atlas_ref));
                    referenced_handles.push(handle);
                    atlas_ref
                }
                ImageRef::ResolvedHandle(handle, atlas_ref) => {
                    referenced_handles.push(handle);
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
                .send(Renderer2DCommand::SubmitFrame(camera_packet, draws, texts, referenced_handles));
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
            pending_atlas_rebuild: false,
        }
    }

    fn poll_event(&self) -> Option<Renderer2DEvent> {
        self.event_receiver.try_recv().ok()
    }
}

impl comet_app::Module for RenderHandle2D {
    fn dependencies(app: &mut comet_app::App) where Self: Sized {
        if !app.has_module::<comet_assets::AssetModule>() {
            app.add_module(comet_assets::AssetModule::new());
        }
        if !app.has_module::<comet_ecs::EcsModule>() {
            app.add_module(comet_ecs::EcsModule::new());
        }
    }
    fn build(&mut self, app: &mut comet_app::App) {
        app.add_post_tick_hook(|app| {
            let mut renderer = app.take_module::<RenderHandle2D>().unwrap();
            renderer.render_scene_2d(app.scene_mut());
            app.reinsert_module(renderer);
        });
    }
}

impl Renderer2D {
    fn setup_atlas_pipeline(&mut self, mut atlas: comet_assets::TextureAtlas) {
        let gpu_texture = match GpuTexture::from_dynamic_image(
            self.render_context.device(),
            self.render_context.queue(),
            atlas.atlas(),
            Some("Atlas"),
            false,
        ) {
            Ok(tex) => tex,
            Err(e) => {
                error!("Failed to convert atlas to GPU texture: {}", e);
                return;
            }
        };
        atlas.clear_atlas_image();

        if let Some(handle) = self.asset_provider.add(atlas) {
            self.render_context.resources_mut().insert_asset_atlas_handle("atlas".to_string(), handle);
        } else {
            error!("Failed to add texture atlas to asset provider");
            return;
        }

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
            vec![],
            None,
            None,
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

        let camera_group = self.render_context.resources()
            .get_bind_groups("Universal")
            .and_then(|groups| groups.get(1))
            .cloned();
        if let Some(cg) = camera_group {
            let resources = self.render_context.resources_mut();
            if resources.get_bind_groups("Font").is_some() {
                let font_groups = resources.get_bind_groups("Font").map(|v| v.len()).unwrap_or(0);
                if font_groups > 1 {
                    resources.replace_bind_group("Font".into(), 1, cg);
                } else {
                    resources.insert_bind_group("Font".into(), cg);
                }

                if let Some(pos) = self.render_passes.iter().position(|p| p.label == "Font") {
                    let font_pass = self.render_passes.remove(pos);
                    self.render_passes.push(font_pass);
                }
            }
        }
    }

    fn ensure_font_initialized(&mut self, handle: comet_assets::Asset<comet_assets::Font>, size: f32) {
        let key = FontKey { index: handle.index(), generation: handle.generation(), size_bits: size.to_bits() };
        if self.font_cache.contains_key(&key) {
            return;
        }

        let font_data = match self.asset_provider.with(handle, |f| f.clone()) {
            Some(f) => f,
            None => {
                error!("Font handle {:?} not ready — skipping rasterization", handle);
                return;
            }
        };

        let (mut glyphs, line_height) = match font_data.rasterize(size) {
            Some(r) => r,
            None => {
                error!("Failed to rasterize font '{}'", font_data.name());
                return;
            }
        };

        let prefix = format!("{}@{}::", handle.index(), size.to_bits());
        for g in &mut glyphs {
            g.name = format!("{}{}", prefix, g.name);
        }
        self.accumulated_font_glyphs.extend(glyphs);
        self.font_cache.insert(key, line_height);

        let mut atlas = comet_assets::TextureAtlas::from_glyphs(&self.accumulated_font_glyphs);

        let font_texture = match GpuTexture::from_dynamic_image(
            self.render_context.device(),
            self.render_context.queue(),
            atlas.atlas(),
            Some("FontAtlas"),
            false,
        ) {
            Ok(tex) => tex,
            Err(e) => {
                error!("Failed to create GPU texture for font atlas: {}", e);
                return;
            }
        };
        atlas.clear_atlas_image();
        let font_texture_arc = Arc::new(font_texture);

        if let Some(old_handle) = self.render_context.resources().get_asset_atlas_handle("font_atlas") {
            self.asset_provider.unload(old_handle);
        }
        if let Some(atlas_handle) = self.asset_provider.add(atlas) {
            self.render_context.resources_mut().insert_asset_atlas_handle("font_atlas".to_string(), atlas_handle);
        }
        self.render_context.resources_mut().insert_gpu_texture("font_atlas".to_string(), font_texture_arc.clone());

        let font_pass_exists = self.render_context.resources().get_bind_group_layout("Font").is_some();

        if font_pass_exists {
            let texture_bind_group_layout = self.render_context.resources()
                .get_bind_group_layout("Font")
                .and_then(|v| v.first())
                .cloned()
                .expect("Font bind group layout missing");

            let sampler = self.render_context.resources().get_sampler("Font")
                .expect("Font sampler missing");
            let new_bind_group = Arc::new(self.render_context.device().create_bind_group(
                &wgpu::BindGroupDescriptor {
                    layout: &texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&font_texture_arc.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(sampler),
                        },
                    ],
                    label: Some("Font Texture Bind Group (Updated)"),
                },
            ));
            self.render_context.resources_mut().replace_bind_group("Font".into(), 0, new_bind_group);
        } else {
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

            let texture_sampler = self.render_context.device().create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });

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
                vec![],
                None,
                None,
                Box::new(universal_load_execute),
                BASE_2D_SHADER_SRC,
                None,
                &(*font_texture_arc),
                texture_bind_group_layout,
                texture_sampler,
                vec![],
                &[camera_bind_group_layout],
            );

            let camera_group = self.render_context.resources()
                .get_bind_groups("Universal")
                .and_then(|groups| groups.get(1))
                .cloned();
            if let Some(cg) = camera_group {
                let resources = self.render_context.resources_mut();
                if resources.get_bind_groups("Font").map(|v| v.len() > 1).unwrap_or(false) {
                    resources.replace_bind_group("Font".into(), 1, cg);
                } else {
                    resources.insert_bind_group("Font".into(), cg);
                }
            }
        }
    }

    fn ensure_image_in_atlas(&mut self, handle: comet_assets::Asset<comet_assets::Image>) -> Option<AtlasRef> {
        if self.render_context.resources().get_asset_atlas_handle("atlas").is_none() {
            self.setup_atlas_pipeline(comet_assets::TextureAtlas::with_capacity(512));
        }
        let atlas_handle = self.render_context.resources().get_asset_atlas_handle("atlas")?;

        if let Some(region) = self.asset_provider.with(atlas_handle, |atlas| atlas.region_for_handle(handle)).flatten() {
            return Some(AtlasRef::new(region, atlas_handle));
        }

        let (w, h) = self.asset_provider.with(handle, |img| (img.width(), img.height()))?;

        let alloc = self.asset_provider.with_mut(atlas_handle, |atlas| {
            atlas.insert_image_handle(handle, w, h, 1)
        }).flatten();

        let (blit_x, blit_y, region) = match alloc {
            Some(r) => r,
            None => {
                self.rebuild_atlas(atlas_handle);
                match self.asset_provider.with_mut(atlas_handle, |atlas| {
                    atlas.insert_image_handle(handle, w, h, 1)
                }).flatten() {
                    Some(r) => r,
                    None => {
                        error!("Failed to insert into atlas even after rebuild");
                        return None;
                    }
                }
            }
        };

        let gpu_texture = self.render_context.resources().get_gpu_texture("atlas")?.clone();
        self.asset_provider.with(handle, |img| {
            gpu_texture.write_region(self.render_context.queue(), blit_x, blit_y, img.data(), w, h);
        });
        self.asset_provider.with_mut(handle, |img| img.evict_pixels());

        Some(AtlasRef::new(region, atlas_handle))
    }

    fn rebuild_atlas(&mut self, atlas_handle: comet_assets::Asset<comet_assets::TextureAtlas>) {
        let handles = self.asset_provider
            .with(atlas_handle, |atlas| atlas.handle_keys())
            .unwrap_or_default();
        let (old_w, old_h) = self.asset_provider
            .with(atlas_handle, |atlas| (atlas.width(), atlas.height()))
            .unwrap_or((512, 512));

        let new_size = (old_w * 2).max(old_h * 2).min(8192);
        info!("Atlas full — rebuilding {}x{} → {}x{}", old_w, old_h, new_size, new_size);

        self.asset_provider.with_mut(atlas_handle, |atlas| {
            atlas.reset_for_rebuild(new_size, new_size);
        });

        let new_gpu = GpuTexture::create_2d_texture(
            self.render_context.device(),
            new_size, new_size,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            wgpu::FilterMode::Nearest,
            Some("Atlas"),
        );

        for h in handles {
            let dims = self.asset_provider.with(h, |img| (img.width(), img.height()));
            let Some((w, h_px)) = dims else { continue; };

            let result = self.asset_provider.with_mut(atlas_handle, |atlas| {
                atlas.insert_image_handle(h, w, h_px, 1)
            }).flatten();
            let Some((blit_x, blit_y, _)) = result else {
                error!("Failed to re-pack handle during atlas rebuild");
                continue;
            };

            let uploaded = self.asset_provider.with(h, |img| {
                if !img.is_evicted() {
                    new_gpu.write_region(self.render_context.queue(), blit_x, blit_y, img.data(), w, h_px);
                    true
                } else {
                    false
                }
            }).unwrap_or(false);

            if !uploaded {
                let path = self.asset_provider.path_for::<comet_assets::Image>(h);
                if let Some(path) = path {
                    let fs_path = comet_assets::resolve_asset_path(&path);
                    if let Ok(bytes) = std::fs::read(&fs_path) {
                        if let Ok(img) = comet_assets::Image::from_bytes(&bytes, false) {
                            new_gpu.write_region(self.render_context.queue(), blit_x, blit_y, img.data(), w, h_px);
                        }
                    }
                }
            }
        }

        let new_gpu_arc = Arc::new(new_gpu);
        self.render_context.resources_mut().insert_gpu_texture("atlas".to_string(), new_gpu_arc.clone());

        let new_bind_group = Arc::new({
            let layout = self.render_context.resources()
                .get_bind_group_layout("Universal")
                .and_then(|v| v.first())
                .cloned()
                .expect("Universal bind group layout missing during atlas rebuild");
            let sampler = self.render_context.resources()
                .get_sampler("Universal")
                .expect("Universal sampler missing during atlas rebuild");
            self.render_context.device().create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&new_gpu_arc.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
                label: Some("Universal Texture Bind Group (Rebuilt)"),
            })
        });
        self.render_context.resources_mut().replace_bind_group("Universal".to_string(), 0, new_bind_group);

        let _ = self.event_sender.send(Renderer2DEvent::AtlasRebuilt);
    }

    pub fn new_render_pass(
        &mut self,
        label: String,
        inputs: Vec<&PassOutput>,
        output: Option<String>,
        output_format: Option<wgpu::TextureFormat>,
        execute: Box<
            dyn Fn(String, &mut RenderContext, &mut wgpu::CommandEncoder, &wgpu::TextureView, &[&wgpu::BindGroup])
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
    ) -> Option<PassOutput> {
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
                    buffers: &[Vertex::desc()],
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

        let input_names: Vec<String> = inputs.iter().map(|p| p.0.clone()).collect();
        let pass_output = output.as_ref().map(|name| PassOutput(name.clone()));

        self.render_passes
            .push(RenderPass::new(label.clone(), input_names, output, output_format, None, execute));

        self.render_context
            .new_batch(label.clone(), Vec::new(), Vec::new());
        info!("Created render pass {}!", label);

        self.graph_dirty = true;
        pass_output
    }

    fn add_pass(&mut self, desc: crate::render_commands::PassDescriptor) -> PassOutput {
        let shader_module = self.render_context.device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("{} Shader", desc.label)),
            source: wgpu::ShaderSource::Wgsl(desc.shader_src.into()),
        });

        let input_count = desc.inputs.len();

        let bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>> = (0..input_count).map(|i| {
            Arc::new(self.render_context.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(&format!("{} Input {} Layout", desc.label, i)),
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
            }))
        }).collect();

        let sampler = Arc::new(self.render_context.device().create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        }));

        let layout_refs: Vec<&wgpu::BindGroupLayout> = bind_group_layouts.iter().map(|l| l.as_ref()).collect();
        let pipeline_layout = self.render_context.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{} Pipeline Layout", desc.label)),
            bind_group_layouts: &layout_refs,
            push_constant_ranges: &[],
        });

        let output_format = desc.output_format.unwrap_or(self.render_context.config().format);
        let pipeline = Arc::new(self.render_context.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("{} Pipeline", desc.label)),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: output_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        }));

        let load_op = match desc.clear {
            Some(color) => wgpu::LoadOp::Clear(color),
            None => wgpu::LoadOp::Load,
        };

        let execute: Box<dyn Fn(String, &mut RenderContext, &mut wgpu::CommandEncoder, &wgpu::TextureView, &[&wgpu::BindGroup]) + Send + Sync> = {
            let pipeline = pipeline.clone();
            Box::new(move |label, ctx, encoder, view, input_bind_groups| {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some(&format!("{} Render Pass", label)),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations { load: load_op, store: wgpu::StoreOp::Store },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                rpass.set_pipeline(&pipeline);
                for (i, bg) in input_bind_groups.iter().enumerate() {
                    rpass.set_bind_group(i as u32, bg, &[]);
                }
                rpass.draw(0..3, 0..1);
            })
        };

        let pass_output = PassOutput(desc.output.clone().unwrap_or_else(|| desc.label.clone()));
        let input_names = desc.inputs;
        let cache = PassCache::new(bind_group_layouts, sampler);

        self.render_passes.push(RenderPass::new(
            desc.label.clone(),
            input_names,
            desc.output,
            desc.output_format,
            Some(cache),
            execute,
        ));

        self.graph_dirty = true;
        info!("Created post-process pass {}!", desc.label);
        pass_output
    }

    fn remove_render_pass(&mut self, label: &str) {
        if let Some(pos) = self.render_passes.iter().position(|p| p.label == label) {
            let pass = self.render_passes.remove(pos);
            if let Some(ref name) = pass.output {
                self.render_context.resources_mut().remove_gpu_texture(name);
            }
        }
        self.graph_dirty = true;
    }

    fn set_pass_output(&mut self, label: &str, output: Option<String>) {
        let Some(pass) = self.render_passes.iter_mut().find(|p| p.label == label) else {
            error!("set_pass_output: no pass '{}'", label);
            return;
        };
        if let Some(ref old) = pass.output {
            self.render_context.resources_mut().remove_gpu_texture(old);
        }
        pass.output = output;
        if let Some(ref mut cache) = pass.cache {
            cache.invalidate();
        }
        self.graph_dirty = true;
    }

    fn build_graph(&mut self) {
        use std::collections::{HashMap, VecDeque};

        let n = self.render_passes.len();
        let output_map: HashMap<&str, usize> = self.render_passes.iter().enumerate()
            .filter_map(|(i, p)| p.output.as_deref().map(|name| (name, i)))
            .collect();

        let mut in_degree = vec![0usize; n];
        let mut adj: Vec<Vec<usize>> = vec![vec![]; n];

        for (i, pass) in self.render_passes.iter().enumerate() {
            for input in &pass.inputs {
                if let Some(&producer) = output_map.get(input.as_str()) {
                    adj[producer].push(i);
                    in_degree[i] += 1;
                } else {
                    error!("Render pass '{}' declares input '{}' but no pass produces it", pass.label, input);
                }
            }
        }

        let mut queue: VecDeque<usize> = in_degree.iter().enumerate()
            .filter(|(_, &d)| d == 0)
            .map(|(i, _)| i)
            .collect();

        let mut order = Vec::with_capacity(n);
        while let Some(i) = queue.pop_front() {
            order.push(i);
            for &dep in &adj[i] {
                in_degree[dep] -= 1;
                if in_degree[dep] == 0 {
                    queue.push_back(dep);
                }
            }
        }

        if order.len() != n {
            fatal!("Render graph contains a cycle");
        }

        self.execution_order = order.into_iter().map(|i| self.render_passes[i].label.clone()).collect();
        self.graph_dirty = false;
        info!("Render graph built: {:?}", self.execution_order);
    }

    fn get_texture_region(&self, texture: AtlasRef) -> TextureRegion {
        texture.region()
    }

    fn get_glyph_region(&self, glyph: char, font: comet_assets::Asset<comet_assets::Font>, size: f32) -> TextureRegion {
        let key = format!("{}@{}::{}", font.index(), size.to_bits(), glyph);
        let fallback_key = format!("{}@{}:: ", font.index(), size.to_bits());

        if let Some(handle) = self.render_context.resources().get_asset_atlas_handle("font_atlas") {
            self.asset_provider.with(handle, |atlas| {
                atlas.textures().get(&key).copied()
                    .or_else(|| atlas.textures().get(&fallback_key).copied())
                    .unwrap_or_else(|| fatal!("No glyph or fallback for '{}' in font atlas", glyph))
            }).unwrap_or_else(|| {
                fatal!("Failed to access font atlas from asset provider");
            })
        } else {
            fatal!("Font atlas not initialized yet");
        }
    }

    pub fn precompute_text_bounds(&mut self, text: &str, font: comet_assets::Asset<comet_assets::Font>, size: f32) -> v2 {
        let mut bounds = v2::ZERO;
        let _ = self.add_text_to_buffers(text, font, size, v2::ZERO, wgpu::Color::WHITE, &mut bounds);
        bounds
    }

    pub fn add_text_to_buffers(
        &mut self,
        text: &str,
        font: comet_assets::Asset<comet_assets::Font>,
        size: f32,
        position: comet_math::v2,
        color: wgpu::Color,
        bounds: &mut comet_math::v2,
    ) -> (Vec<Vertex>, Vec<u16>) {
        self.ensure_font_initialized(font, size);

        let cache_key = FontKey { index: font.index(), generation: font.generation(), size_bits: size.to_bits() };
        let line_height_px = self.font_cache.get(&cache_key).copied().unwrap_or(size);

        let vert_color = [
            color.r as f32,
            color.g as f32,
            color.b as f32,
            color.a as f32,
        ];

        let config = self.render_context.config();
        let line_height = line_height_px / config.height as f32;

        let screen_position = comet_math::v2::new(
            position.x() / config.width as f32,
            position.y() / config.height as f32,
        );

        let lines: Vec<String> = text
            .split('\n')
            .map(|s| s.chars().map(|c| if c == '\t' { ' ' } else { c }).collect())
            .collect();

        let mut max_line_width = 0.0f32;
        for line in &lines {
            let line_width: f32 = line.chars()
                .map(|c| self.get_glyph_region(c, font, size).advance())
                .sum();
            if line_width > max_line_width {
                max_line_width = line_width;
            }
        }
        bounds.set_x(max_line_width);
        bounds.set_y(lines.len() as f32 * line_height_px);

        let mut x_offset = 0.0f32;
        let mut y_offset = 0.0f32;
        let mut vertex_data = Vec::new();
        let mut index_data = Vec::new();

        for line in lines {
            for c in line.chars() {
                let region = self.get_glyph_region(c, font, size);

                let (dim_x, dim_y) = region.dimensions();
                let w = dim_x as f32 / config.width as f32;
                let h = dim_y as f32 / config.height as f32;
                let offset_x = region.offset_x() / config.width as f32;
                let offset_y = region.offset_y() / config.height as f32;

                let glyph_left = screen_position.x() + x_offset + offset_x;
                let glyph_top = screen_position.y() - offset_y - y_offset;
                let glyph_right = glyph_left + w;
                let glyph_bottom = glyph_top - h;

                let buffer_size = vertex_data.len() as u16;
                vertex_data.extend_from_slice(&[
                    Vertex::new([glyph_left,  glyph_top,    0.0], [region.u0(), region.v0()], vert_color),
                    Vertex::new([glyph_left,  glyph_bottom, 0.0], [region.u0(), region.v1()], vert_color),
                    Vertex::new([glyph_right, glyph_bottom, 0.0], [region.u1(), region.v1()], vert_color),
                    Vertex::new([glyph_right, glyph_top,    0.0], [region.u1(), region.v0()], vert_color),
                ]);
                index_data.extend_from_slice(&[
                    buffer_size, buffer_size + 1, buffer_size + 3,
                    buffer_size + 1, buffer_size + 2, buffer_size + 3,
                ]);

                x_offset += region.advance() / config.width as f32;
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
        referenced_handles: Vec<comet_assets::Asset<comet_assets::Image>>,
    ) {
        if self.render_context.resources().get_asset_atlas_handle("atlas").is_none() {
            self.setup_atlas_pipeline(comet_assets::TextureAtlas::with_capacity(512));
        }

        if let Some(atlas_handle) = self.render_context.resources().get_asset_atlas_handle("atlas") {
            let any_evicted = self.asset_provider.with_mut(atlas_handle, |atlas| {
                let mut evicted = false;
                for handle in &referenced_handles {
                    if atlas.region_for_handle(*handle).is_some() {
                        atlas.mark_used(*handle);
                    } else {
                        evicted = true;
                    }
                }
                atlas.evict_stale(120);
                evicted
            }).unwrap_or(false);
            if any_evicted {
                let _ = self.event_sender.send(Renderer2DEvent::AtlasRebuilt);
            }
        }
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

        let buffer = match self.render_context.resources().get_buffer("Universal")
            .and_then(|v| v.first())
            .cloned()
        {
            Some(b) => b,
            None => {
                error!("Camera buffer missing for 'Universal' pass.");
                return;
            }
        };

        self.render_context.queue().write_buffer(
            &buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );
    }
}

impl Renderer for Renderer2D {
    type Handle = RenderHandle2D;

    fn new(
        window: Arc<Window>,
        clear_color: Option<impl Color>,
        event_sender: flume::Sender<Renderer2DEvent>,
    ) -> Self {
        let asset_provider = comet_assets::AssetProvider::new(comet_assets::AssetManager::new());
        Self {
            render_context: RenderContext::new(window, clear_color),
            asset_provider,
            render_passes: Vec::new(),
            execution_order: Vec::new(),
            graph_dirty: true,
            last_frame_time: std::time::Instant::now(),
            delta_time: 0.0,
            event_sender,
            font_cache: std::collections::HashMap::new(),
            accumulated_font_glyphs: Vec::new(),
        }
    }

    fn init_assets(&mut self, app: &::comet_app::App) {
        if app.has_context::<comet_assets::AssetProvider>() {
            self.asset_provider = app.context::<comet_assets::AssetProvider>().clone();
        }
    }

    fn apply_command(&mut self, command: <Self::Handle as RendererHandle>::Command) {
        match command {
            Renderer2DCommand::Clear => {}
            Renderer2DCommand::ResolveAtlasRef(path) => {
                let atlas_ref = self.render_context
                    .resources()
                    .get_asset_atlas_handle("atlas")
                    .and_then(|handle| {
                        self.asset_provider.with(handle, |atlas| {
                            atlas.textures()
                                .get(path)
                                .copied()
                                .map(|region| AtlasRef::new(region, handle))
                        })
                        .flatten()
                    });

                let mut dynamic_image_handle: Option<comet_assets::Asset<comet_assets::Image>> = None;
                let atlas_ref = atlas_ref.or_else(|| {
                    let fs_path = comet_assets::resolve_asset_path(path);
                    let bytes = std::fs::read(&fs_path).ok()?;
                    let image = comet_assets::Image::from_bytes(&bytes, false).ok()?;
                    let image_handle = self.asset_provider.add(image)?;
                    self.asset_provider.track_for_reload::<comet_assets::Image>(image_handle, path);
                    let result = self.ensure_image_in_atlas(image_handle);
                    if result.is_some() {
                        dynamic_image_handle = Some(image_handle);
                    }
                    result
                });

                let _ = self.event_sender.send(Renderer2DEvent::AtlasRef(atlas_ref, dynamic_image_handle));
            }
            Renderer2DCommand::EnsureHandleInAtlas(handle) => {
                let atlas_ref = self.ensure_image_in_atlas(handle);
                let _ = self.event_sender.send(Renderer2DEvent::AtlasRef(atlas_ref, None));
            }
            Renderer2DCommand::Size => {
                let _ = self.event_sender.send(Renderer2DEvent::Size(self.size()));
            }
            Renderer2DCommand::ScaleFactor => {
                let _ = self
                    .event_sender
                    .send(Renderer2DEvent::ScaleFactor(self.scale_factor()));
            }
            Renderer2DCommand::PrecomputedTextBounds {
                text,
                font,
                font_size,
            } => {
                let bounds = self.precompute_text_bounds(&text, font, font_size);
                let _ = self
                    .event_sender
                    .send(Renderer2DEvent::PrecomputedTextBounds {
                        width: bounds.x(),
                        height: bounds.y(),
                    });
            }
            Renderer2DCommand::SubmitFrame(camera, draws, texts, referenced_handles) => {
                self.submit_frame(camera, draws, texts, referenced_handles)
            }
            Renderer2DCommand::AddRenderPass(desc) => {
                let pass_output = self.add_pass(desc);
                let _ = self.event_sender.send(Renderer2DEvent::PassAdded(pass_output));
            }
            Renderer2DCommand::RemoveRenderPass(label) => {
                self.remove_render_pass(&label);
                let _ = self.event_sender.send(Renderer2DEvent::PassRemoved);
            }
            Renderer2DCommand::SetPassOutput(label, output) => {
                self.set_pass_output(&label, output);
                let _ = self.event_sender.send(Renderer2DEvent::PassOutputSet);
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

            for pass in &mut self.render_passes {
                if let Some(ref name) = pass.output {
                    self.render_context.resources_mut().remove_gpu_texture(name);
                }
                if let Some(ref mut cache) = pass.cache {
                    cache.invalidate();
                }
            }
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
        if self.graph_dirty {
            self.build_graph();
        }

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

        let labels: Vec<String> = self.execution_order.clone();
        for label in labels {
            let pass_idx = match self.render_passes.iter().position(|p| p.label == label) {
                Some(i) => i,
                None => continue,
            };

            if let Some(ref name) = self.render_passes[pass_idx].output.clone() {
                if self.render_context.resources().get_gpu_texture(name).is_none() {
                    let w = self.render_context.config().width;
                    let h = self.render_context.config().height;
                    let format = self.render_passes[pass_idx].output_format
                        .unwrap_or(self.render_context.config().format);
                    self.render_context.create_intermediate_texture(name.clone(), w, h, format);
                    if let Some(ref mut cache) = self.render_passes[pass_idx].cache {
                        cache.invalidate();
                    }
                }
            }

            let intermediate = self.render_passes[pass_idx].output.as_ref()
                .and_then(|name| self.render_context.resources().get_gpu_texture(name).cloned());

            let view: &wgpu::TextureView = match &intermediate {
                Some(tex) => &tex.view,
                None => &output_view,
            };

            if self.render_passes[pass_idx].cache.as_ref().is_some_and(|c| c.bind_groups.is_none()) {
                let inputs: Vec<Arc<GpuTexture>> = self.render_passes[pass_idx].inputs.iter()
                    .filter_map(|name| self.render_context.resources().get_gpu_texture(name).cloned())
                    .collect();
                if let Some(ref cache) = self.render_passes[pass_idx].cache {
                    let groups: Vec<Arc<wgpu::BindGroup>> = inputs.iter().zip(cache.layouts.iter()).map(|(tex, layout)| {
                        Arc::new(self.render_context.device().create_bind_group(&wgpu::BindGroupDescriptor {
                            layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(&tex.view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(&cache.sampler),
                                },
                            ],
                            label: Some(&format!("{} Bind Group", label)),
                        }))
                    }).collect();
                    self.render_passes[pass_idx].cache.as_mut().unwrap().bind_groups = Some(groups);
                }
            }

            let owned_groups: Vec<Arc<wgpu::BindGroup>> = self.render_passes[pass_idx].cache
                .as_ref()
                .and_then(|c| c.bind_groups.as_ref())
                .map(|v| v.clone())
                .unwrap_or_default();
            let group_refs: Vec<&wgpu::BindGroup> = owned_groups.iter().map(|g| g.as_ref()).collect();

            (self.render_passes[pass_idx].execute)(label.clone(), &mut self.render_context, &mut encoder, view, &group_refs);
        }

        self.render_context
            .queue()
            .submit(std::iter::once(encoder.finish()));

        self.render_context.device().poll(wgpu::Maintain::Poll);

        output.present();

        Ok(())
    }
}

struct ErasedRenderer2D {
    renderer: Renderer2D,
    cmd_rx: flume::Receiver<Renderer2DCommand>,
}

impl comet_window::ErasedRenderer for ErasedRenderer2D {
    fn init_assets(&mut self, app: &comet_app::App) {
        self.renderer.init_assets(app);
    }
    fn drain_commands(&mut self) {
        while let Ok(cmd) = self.cmd_rx.try_recv() {
            self.renderer.apply_command(cmd);
        }
    }
    fn window(&self) -> &winit::window::Window {
        self.renderer.window()
    }
    fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.renderer.size()
    }
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(new_size);
    }
    fn scale_factor(&self) -> f64 {
        self.renderer.scale_factor()
    }
    fn set_scale_factor(&mut self, scale_factor: f64) {
        self.renderer.set_scale_factor(scale_factor);
    }
    fn update(&mut self) -> f32 {
        self.renderer.update()
    }
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.renderer.render()
    }
}

pub struct Renderer2DModule;

impl Renderer2DModule {
    pub fn new() -> Self {
        Self
    }
}

impl Module for Renderer2DModule {
    fn dependencies(app: &mut App) where Self: Sized {
        if !app.has_module::<comet_assets::AssetModule>() {
            app.add_module(comet_assets::AssetModule::new());
        }
    }
    fn build(&mut self, app: &mut App) {
        if !app.has_module::<comet_window::winit_module::WinitModule>() {
            return;
        }
        app.get_module_mut::<comet_window::winit_module::WinitModule>()
            .set_renderer_factory(Box::new(|window, clear_color| {
                let (cmd_tx, cmd_rx) = flume::unbounded::<Renderer2DCommand>();
                let (evt_tx, evt_rx) = flume::unbounded::<Renderer2DEvent>();

                let renderer = Renderer2D::new(window, clear_color, evt_tx);
                let handle = RenderHandle2D::new(cmd_tx, evt_rx);

                let erased: Box<dyn comet_window::ErasedRenderer> =
                    Box::new(ErasedRenderer2D { renderer, cmd_rx });
                let add_handle: Box<dyn FnOnce(&mut comet_app::App) + Send> =
                    Box::new(move |app| { app.add_module(handle); });

                (erased, add_handle)
            }));
    }
}

