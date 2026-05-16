use crate::{
    camera::{CameraUniform, RenderCamera},
    gpu_texture::GpuTexture,
    render_commands::{CameraPacket2D, Draw2D, Renderer2DCommand, Text2D},
    render_state::RenderState,
    render_events::Renderer2DEvent,
    render_pass::{LoadOp, PassOutput},
    render_graph::{RenderGraph, nodes::{PassNode, PostProcessNode}},
    Vertex,
};
use comet_colors::Color;
use comet_gizmos::{Gizmo, GizmoBuffer, GizmoShape};
use crate::gizmo_registry::GizmoRegistry;
use comet_ecs::Component;
use comet_app::{App, Module};
use comet_ecs::EcsModuleExt;
use comet_macros::module;
use comet_window::renderer::{Renderer, RendererHandle};
use comet_log::*;
use comet_math::{v2, v3};
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
use winit::{dpi::PhysicalSize, window::Window};

pub struct Renderer2D {
    render_state: RenderState,
    asset_provider: comet_assets::AssetProvider,
    graph: RenderGraph,
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
    gizmo_buffer: GizmoBuffer,
    gizmo_registry: GizmoRegistry,
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
        render_target: Option<&PassOutput>,
        output_format: Option<wgpu::TextureFormat>,
        shader_src: String,
        load: LoadOp,
    ) -> Option<PassOutput> {
        let desc = crate::render_commands::PassDescriptor {
            label,
            inputs: inputs.iter().map(|p| p.0.clone()).collect(),
            output,
            render_target: render_target.map(|p| p.0.clone()),
            output_format,
            shader_src,
            load,
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

    pub fn set_pass_output(&mut self, label: &str, output: Option<PassOutput>) -> Option<PassOutput> {
        let _ = self.command_sender.send(Renderer2DCommand::SetPassOutput(label.to_string(), output));
        self.recv_matching_event(Duration::from_millis(5000), |e| {
            matches!(e, Renderer2DEvent::PassOutputSet(_))
        })
        .and_then(|e| match e {
            Renderer2DEvent::PassOutputSet(handle) => handle,
            _ => None,
        })
    }

    pub fn set_pass_render_target(&mut self, label: &str, render_target: Option<&PassOutput>) {
        let _ = self.command_sender.send(Renderer2DCommand::SetPassRenderTarget(
            label.to_string(),
            render_target.map(|p| p.0.clone()),
        ));
        let _ = self.recv_matching_event(Duration::from_millis(5000), |e| {
            matches!(e, Renderer2DEvent::PassRenderTargetSet)
        });
    }

    pub fn show_gizmo<C: Component + Gizmo + 'static>(&mut self, entity: comet_ecs::Entity) {
        self.gizmo_registry.show::<C>(entity);
    }

    pub fn hide_gizmo<C: Component + Gizmo + 'static>(&mut self, entity: comet_ecs::Entity) {
        self.gizmo_registry.hide::<C>(entity);
    }

    pub fn show_all_gizmos<C: Component + Gizmo + 'static>(&mut self) {
        self.gizmo_registry.show_all::<C>();
    }

    pub fn hide_all_gizmos<C: Component + Gizmo + 'static>(&mut self) {
        self.gizmo_registry.hide_all::<C>();
    }

}

impl RenderHandle2D {
    pub fn render_scene_2d(&mut self, scene: &mut comet_ecs::Scene) {
        self.poll_events();
        if self.pending_atlas_rebuild {
            self.pending_atlas_rebuild = false;
            for (_, render) in scene
                .query_mut::<(&comet_ecs::Transform, &mut comet_ecs::Sprite), ()>()
                .iter()
            {
                if let ImageRef::ResolvedHandle(h, _) = render.texture() {
                    render.set_image_ref(ImageRef::Handle(h));
                }
            }
        }

        let mut selected_camera: Option<([f32; 2], f32, f32, u8, comet_ecs::Projection)> = None;
        for (transform, camera) in scene
            .query::<(&comet_ecs::Transform, &comet_ecs::Camera), ()>()
            .iter()
        {
            let should_replace = selected_camera
                .as_ref()
                .is_none_or(|(_, _, _, current_priority, _)| camera.priority() < *current_priority);
            if should_replace {
                selected_camera = Some((
                    [transform.position().x(), transform.position().y()],
                    transform.rotation().z().to_degrees(),
                    camera.zoom(),
                    camera.priority(),
                    camera.projection().clone(),
                ));
            }
        }
        let Some((camera_pos, camera_rot, camera_zoom, camera_priority, camera_projection)) =
            selected_camera
        else {
            return;
        };

        let mut draws = Vec::new();
        let mut referenced_handles = Vec::new();
        for (transform, render) in scene
            .query_mut::<(&comet_ecs::Transform, &mut comet_ecs::Sprite), ()>()
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
                rotation_deg: transform.rotation().z().to_degrees(),
                scale: [transform.scale().x(), transform.scale().y()],
                texture: atlas_ref,
                draw_index: render.draw_index(),
                visible: render.is_visible(),
            });
        }
        draws.sort_by_key(|draw| draw.draw_index);

        let mut texts = Vec::new();
        for (transform, text) in scene
            .query::<(&comet_ecs::Transform, &comet_ecs::Text), ()>()
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
            priority: camera_priority,
            projection: camera_projection,
        };

        self.gizmo_registry.flush(scene, &mut self.gizmo_buffer);
        let gizmo_shapes = std::mem::take(&mut self.gizmo_buffer.shapes);

        let _ =
            self.command_sender
                .send(Renderer2DCommand::SubmitFrame(camera_packet, draws, texts, referenced_handles, gizmo_shapes));
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
            gizmo_buffer: GizmoBuffer::new(),
            gizmo_registry: GizmoRegistry::new(),
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

const SPRITE_SHADER: &str = r#"
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

const GIZMO_SHADER: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

impl Renderer2D {
    fn setup_atlas_pipeline(&mut self, mut atlas: comet_assets::TextureAtlas) {
        let gpu_texture = match GpuTexture::from_dynamic_image(
            self.render_state.device(),
            self.render_state.queue(),
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
            self.render_state.resources_mut().insert_asset_atlas_handle("atlas".to_string(), handle);
        } else {
            error!("Failed to add texture atlas to asset provider");
            return;
        }

        let gpu_texture_arc = Arc::new(gpu_texture);
        self.render_state
            .resources_mut()
            .insert_gpu_texture("atlas".to_string(), gpu_texture_arc.clone());

        let format = self.render_state.config().format;
        let width = self.render_state.config().width;
        let height = self.render_state.config().height;

        self.graph.add_node(
            PassNode::new("Universal", SPRITE_SHADER, wgpu::PrimitiveTopology::TriangleList, Some(gpu_texture_arc), vec![], LoadOp::Background),
            self.render_state.device(),
            self.render_state.queue(),
            format,
            width,
            height,
        );

        self.graph.add_node(
            PassNode::new("Gizmo", GIZMO_SHADER, wgpu::PrimitiveTopology::LineList, None, vec!["Universal", "Font"], LoadOp::Load),
            self.render_state.device(),
            self.render_state.queue(),
            format,
            width,
            height,
        );
    }

    fn ensure_font_initialized(&mut self, handle: comet_assets::Asset<comet_assets::Font>, size: f32) {
        let key = FontKey { index: handle.index(), generation: handle.generation(), size_bits: size.to_bits() };
        if self.font_cache.contains_key(&key) {
            return;
        }

        let font_data = match self.asset_provider.with(handle, |f| f.clone()) {
            Some(f) => f,
            None => {
                error!("Font handle {:?} not read: skipping rasterization", handle);
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
            self.render_state.device(),
            self.render_state.queue(),
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

        if let Some(old_handle) = self.render_state.resources().get_asset_atlas_handle("font_atlas") {
            self.asset_provider.unload(old_handle);
        }
        if let Some(atlas_handle) = self.asset_provider.add(atlas) {
            self.render_state.resources_mut().insert_asset_atlas_handle("font_atlas".to_string(), atlas_handle);
        }
        self.render_state.resources_mut().insert_gpu_texture("font_atlas".to_string(), font_texture_arc.clone());

        if self.graph.has_node("Font") {
            let device = self.render_state.device();
            self.graph
                .get_node_mut::<PassNode>("Font")
                .unwrap()
                .set_texture(font_texture_arc, device);
        } else {
            let format = self.render_state.config().format;
            let width = self.render_state.config().width;
            let height = self.render_state.config().height;
            self.graph.add_node(
                PassNode::new("Font", SPRITE_SHADER, wgpu::PrimitiveTopology::TriangleList, Some(font_texture_arc), vec!["Universal"], LoadOp::Load),
                self.render_state.device(),
                self.render_state.queue(),
                format,
                width,
                height,
            );
        }
    }

    fn ensure_image_in_atlas(&mut self, handle: comet_assets::Asset<comet_assets::Image>) -> Option<AtlasRef> {
        let atlas_handle = self.render_state.resources().get_asset_atlas_handle("atlas")?;

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

        let gpu_texture = self.render_state.resources().get_gpu_texture("atlas")?.clone();
        self.asset_provider.with(handle, |img| {
            gpu_texture.write_region(self.render_state.queue(), blit_x, blit_y, img.data(), w, h);
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
        info!("Atlas full: rebuilding {}x{} → {}x{}", old_w, old_h, new_size, new_size);

        self.asset_provider.with_mut(atlas_handle, |atlas| {
            atlas.reset_for_rebuild(new_size, new_size);
        });

        let new_gpu = GpuTexture::create_2d_texture(
            self.render_state.device(),
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
                    new_gpu.write_region(self.render_state.queue(), blit_x, blit_y, img.data(), w, h_px);
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
                            new_gpu.write_region(self.render_state.queue(), blit_x, blit_y, img.data(), w, h_px);
                        }
                    }
                }
            }
        }

        let new_gpu_arc = Arc::new(new_gpu);
        self.render_state.resources_mut().insert_gpu_texture("atlas".to_string(), new_gpu_arc.clone());

        let device = self.render_state.device();
        if let Some(node) = self.graph.get_node_mut::<PassNode>("Universal") {
            node.set_texture(new_gpu_arc, device);
        }

        let _ = self.event_sender.send(Renderer2DEvent::AtlasRebuilt);
    }

    fn add_pass(&mut self, desc: crate::render_commands::PassDescriptor) -> PassOutput {
        let load = if desc.render_target.is_some() {
            if let LoadOp::Color(_) | LoadOp::Background = desc.load {
                warn!("pass '{}': render_target with non-Load op, forcing Load", desc.label);
            }
            LoadOp::Load
        } else {
            desc.load
        };

        let pass_output = PassOutput(desc.output.clone().unwrap_or_else(|| desc.label.clone()));

        let format = self.render_state.config().format;
        let width = self.render_state.config().width;
        let height = self.render_state.config().height;

        self.graph.add_node(
            PostProcessNode::new(
                desc.label,
                desc.inputs,
                desc.output,
                desc.render_target,
                desc.output_format,
                load,
                desc.shader_src,
            ),
            self.render_state.device(),
            self.render_state.queue(),
            format,
            width,
            height,
        );

        #[cfg(feature = "comet_debug")]
        info!("Created pass {}!", pass_output.name());
        pass_output
    }

    fn remove_pass(&mut self, label: &str) {
        self.graph.remove_node(label);
    }

    fn set_pass_render_target(&mut self, label: &str, render_target: Option<String>) {
        if let Some(node) = self.graph.get_node_mut::<PostProcessNode>(label) {
            node.set_render_target(render_target);
            self.graph.mark_dirty();
        } else {
            error!("set_pass_render_target: no PostProcessNode '{}'", label);
        }
    }

    fn set_pass_output(&mut self, label: &str, output: Option<PassOutput>) -> Option<PassOutput> {
        if let Some(node) = self.graph.get_node_mut::<PostProcessNode>(label) {
            let result = output.clone();
            node.set_output(output.map(|p| p.0));
            self.graph.mark_dirty();
            result
        } else {
            error!("set_pass_output: no PostProcessNode '{}'", label);
            None
        }
    }

    fn get_texture_region(&self, texture: AtlasRef) -> TextureRegion {
        texture.region()
    }

    fn get_glyph_region(&self, glyph: char, font: comet_assets::Asset<comet_assets::Font>, size: f32) -> TextureRegion {
        let key = format!("{}@{}::{}", font.index(), size.to_bits(), glyph);
        let fallback_key = format!("{}@{}:: ", font.index(), size.to_bits());

        if let Some(handle) = self.render_state.resources().get_asset_atlas_handle("font_atlas") {
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

        let config = self.render_state.config();
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
        gizmo_shapes: Vec<GizmoShape>,
    ) {
        if let Some(atlas_handle) = self.render_state.resources().get_asset_atlas_handle("atlas") {
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

            vertex_buffer.extend_from_slice(&[
                Vertex::new(
                    [rotated_world_corners[0].0, rotated_world_corners[0].1, 0.0],
                    [region.u0(), region.v0()],
                    [1.0, 1.0, 1.0, 1.0],
                ),
                Vertex::new(
                    [rotated_world_corners[1].0, rotated_world_corners[1].1, 0.0],
                    [region.u0(), region.v1()],
                    [1.0, 1.0, 1.0, 1.0],
                ),
                Vertex::new(
                    [rotated_world_corners[2].0, rotated_world_corners[2].1, 0.0],
                    [region.u1(), region.v1()],
                    [1.0, 1.0, 1.0, 1.0],
                ),
                Vertex::new(
                    [rotated_world_corners[3].0, rotated_world_corners[3].1, 0.0],
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

        let device = self.render_state.device();
        let queue = self.render_state.queue();

        if let Some(node) = self.graph.get_node_mut::<PassNode>("Universal") {
            node.set_geometry(vertex_buffer, index_buffer, device, queue);
        }

        let mut font_vertex_buffer: Vec<Vertex> = Vec::new();
        let mut font_index_buffer: Vec<u16> = Vec::new();

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
            let (mut vertices, indices) = self.add_text_to_buffers(
                &text.content,
                text.font,
                text.size,
                position,
                color,
                &mut bounds,
            );

            let offset = font_vertex_buffer.len() as u16;
            font_vertex_buffer.append(&mut vertices);
            font_index_buffer.extend(indices.iter().map(|i| i + offset));
        }

        let device = self.render_state.device();
        let queue = self.render_state.queue();

        if let Some(node) = self.graph.get_node_mut::<PassNode>("Font") {
            node.set_geometry(font_vertex_buffer, font_index_buffer, device, queue);
        }

        let mut gizmo_verts: Vec<Vertex> = Vec::new();
        let mut gizmo_indices: Vec<u16> = Vec::new();

        for shape in gizmo_shapes {
            match shape {
                GizmoShape::Line { start, end, color } => {
                    let c = [color.red(), color.green(), color.blue(), color.alpha()];
                    let base = gizmo_verts.len() as u16;
                    gizmo_verts.push(Vertex::new([start.x(), start.y(), start.z()], [0.0, 0.0], c));
                    gizmo_verts.push(Vertex::new([end.x(), end.y(), end.z()], [0.0, 0.0], c));
                    gizmo_indices.extend_from_slice(&[base, base + 1]);
                }
                GizmoShape::Rect { position, size, color } => {
                    let c = [color.red(), color.green(), color.blue(), color.alpha()];
                    let hx = size.x() * 0.5;
                    let hy = size.y() * 0.5;
                    let base = gizmo_verts.len() as u16;
                    let corners = [
                        [position.x() - hx, position.y() + hy, position.z()],
                        [position.x() + hx, position.y() + hy, position.z()],
                        [position.x() + hx, position.y() - hy, position.z()],
                        [position.x() - hx, position.y() - hy, position.z()],
                    ];
                    for corner in &corners {
                        gizmo_verts.push(Vertex::new(*corner, [0.0, 0.0], c));
                    }
                    gizmo_indices.extend_from_slice(&[
                        base, base + 1,
                        base + 1, base + 2,
                        base + 2, base + 3,
                        base + 3, base,
                    ]);
                }
                GizmoShape::Circle { position, radius, color } => {
                    let c = [color.red(), color.green(), color.blue(), color.alpha()];
                    let segments = 32u32;
                    let base = gizmo_verts.len() as u16;
                    for i in 0..segments {
                        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
                        let x = position.x() + radius * angle.cos();
                        let y = position.y() + radius * angle.sin();
                        gizmo_verts.push(Vertex::new([x, y, position.z()], [0.0, 0.0], c));
                        let next = (i + 1) % segments;
                        gizmo_indices.extend_from_slice(&[base + i as u16, base + next as u16]);
                    }
                }
                GizmoShape::NGon { position, radius, vertices, color } => {
                    let c = [color.red(), color.green(), color.blue(), color.alpha()];
                    let n = vertices.max(3);
                    let base = gizmo_verts.len() as u16;
                    for i in 0..n {
                        let angle = (i as f32 / n as f32) * std::f32::consts::TAU;
                        let x = position.x() + radius * angle.cos();
                        let y = position.y() + radius * angle.sin();
                        gizmo_verts.push(Vertex::new([x, y, position.z()], [0.0, 0.0], c));
                        let next = (i + 1) % n;
                        gizmo_indices.extend_from_slice(&[base + i as u16, base + next as u16]);
                    }
                }
            }
        }

        let device = self.render_state.device();
        let queue = self.render_state.queue();

        if let Some(node) = self.graph.get_node_mut::<PassNode>("Gizmo") {
            node.set_geometry(gizmo_verts, gizmo_indices, device, queue);
        }
    }

    fn setup_camera_from_packet(&mut self, camera: CameraPacket2D) {
        use comet_ecs::Projection;

        let width = self.render_state.config().width as f32;
        let height = self.render_state.config().height as f32;

        let view_proj: [[f32; 4]; 4] = match camera.projection {
            Projection::Custom { matrix } => matrix.into(),
            _ => {
                let render_camera = RenderCamera::new(
                    camera.zoom,
                    v2::new(width, height),
                    v3::new(camera.position[0], camera.position[1], 0.0),
                );
                render_camera.build_view_projection_matrix().into()
            }
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.set_view_proj(view_proj);

        let queue = self.render_state.queue();

        if let Some(node) = self.graph.get_node_mut::<PassNode>("Universal") {
            node.set_camera(&camera_uniform, queue);
        }
        if let Some(node) = self.graph.get_node_mut::<PassNode>("Font") {
            node.set_camera(&camera_uniform, queue);
        }
        if let Some(node) = self.graph.get_node_mut::<PassNode>("Gizmo") {
            node.set_camera(&camera_uniform, queue);
        }
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
            render_state: RenderState::new(window, clear_color),
            asset_provider,
            graph: RenderGraph::new(),
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
        self.setup_atlas_pipeline(comet_assets::TextureAtlas::with_capacity(512));
    }

    fn apply_command(&mut self, command: <Self::Handle as RendererHandle>::Command) {
        match command {
            Renderer2DCommand::Clear => {}
            Renderer2DCommand::ResolveAtlasRef(path) => {
                let atlas_ref = self.render_state
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
            Renderer2DCommand::SubmitFrame(camera, draws, texts, referenced_handles, gizmo_shapes) => {
                self.submit_frame(camera, draws, texts, referenced_handles, gizmo_shapes)
            }
            Renderer2DCommand::AddRenderPass(desc) => {
                let pass_output = self.add_pass(desc);
                let _ = self.event_sender.send(Renderer2DEvent::PassAdded(pass_output));
            }
            Renderer2DCommand::RemoveRenderPass(label) => {
                self.remove_pass(&label);
                let _ = self.event_sender.send(Renderer2DEvent::PassRemoved);
            }
            Renderer2DCommand::SetPassOutput(label, output) => {
                let handle = self.set_pass_output(&label, output);
                let _ = self.event_sender.send(Renderer2DEvent::PassOutputSet(handle));
            }
            Renderer2DCommand::SetPassRenderTarget(label, render_target) => {
                self.set_pass_render_target(&label, render_target);
                let _ = self.event_sender.send(Renderer2DEvent::PassRenderTargetSet);
            }
        }
    }

    fn window(&self) -> &Window {
        self.render_state.window()
    }

    fn size(&self) -> PhysicalSize<u32> {
        self.render_state.size()
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.render_state.set_size(new_size);
            self.render_state.config_mut().width = new_size.width;
            self.render_state.config_mut().height = new_size.height;
            self.render_state.configure_surface();
            self.graph.on_resize(
                self.render_state.device(),
                self.render_state.queue(),
                new_size.width,
                new_size.height,
            );
        }
    }

    fn scale_factor(&self) -> f64 {
        self.render_state.scale_factor()
    }

    fn set_scale_factor(&mut self, scale_factor: f64) {
        self.render_state.set_scale_factor(scale_factor);
    }

    fn update(&mut self) -> f32 {
        let now = std::time::Instant::now();
        self.delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        self.delta_time
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.render_state.surface().get_current_texture()?;
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let clear_color = self.render_state.clear_color();
        let format = self.render_state.config().format;
        let width = self.render_state.config().width;
        let height = self.render_state.config().height;

        self.graph.execute(
            self.render_state.device(),
            self.render_state.queue(),
            &output_view,
            clear_color,
            format,
            width,
            height,
        );

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

