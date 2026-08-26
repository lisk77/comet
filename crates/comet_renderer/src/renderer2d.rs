use crate::gizmo_registry::GizmoRegistry;
use crate::{
    camera::{resolve_camera_viewport, CameraUniform, RenderCamera, ResolvedViewport},
    draw_batch::{DrawCommand, GeometryDescriptor, IndexStreamDescriptor, VertexStreamDescriptor},
    gpu_texture::GpuTexture,
    render_commands::{
        CameraPacket2D, Draw2D, FramePacket2D, Renderer2DCommand, ScreenText2D, Text2D,
    },
    render_events::Renderer2DEvent,
    render_graph::{
        nodes::{PassNode, PostProcessNode},
        RenderGraph,
    },
    render_pass::{LoadOp, PassOutput},
    render_state::RenderState,
    SpriteInstance, Vertex,
};
use comet_app::{App, Module};
use comet_assets::{texture_atlas::*, AssetPath, AtlasRef, ImageRef};
use comet_colors::Color;
use comet_ecs::Component;
use comet_ecs::EcsModuleExt;
use comet_gizmos::{Gizmo, GizmoBuffer, GizmoShape};
use comet_log::*;
use comet_macros::module;
use comet_math::{v2, v3};
use comet_window::renderer::{Renderer, RendererHandle};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct FontKey {
    index: u32,
    generation: u32,
    size_bits: u32,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum GlyphRepresentation {
    Bitmap,
    Mtsdf,
    Pixel,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct GlyphKey {
    font: FontKey,
    character: char,
    representation: GlyphRepresentation,
}

#[derive(Clone, Copy)]
struct ResolvedGlyph {
    region: TextureRegion,
    representation: GlyphRepresentation,
    distance_range: f32,
}
use winit::{dpi::PhysicalSize, window::Window};

type FrameMailbox2D = Arc<Mutex<Option<FramePacket2D>>>;

#[cfg(debug_assertions)]
static DEBUG_FONT_ATLAS_GENERATION: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

pub struct Renderer2D {
    render_state: RenderState,
    asset_provider: comet_assets::AssetProvider,
    graph: RenderGraph,
    last_frame_time: std::time::Instant,
    delta_time: f32,
    event_sender: flume::Sender<Renderer2DEvent>,
    font_cache: std::collections::HashMap<FontKey, f32>,
    glyph_cache: std::collections::HashMap<GlyphKey, TextureRegion>,
    sprite_instances: Vec<SpriteInstance>,
    world_text_vertices: Vec<Vertex>,
    world_text_indices: Vec<u16>,
    screen_text_vertices: Vec<Vertex>,
    screen_text_indices: Vec<u16>,
    gizmo_vertices: Vec<Vertex>,
    gizmo_indices: Vec<u16>,
    #[cfg(debug_assertions)]
    debug_font_atlas: image::RgbaImage,
    #[cfg(debug_assertions)]
    debug_font_atlas_dirty: bool,
}

pub struct RenderHandle2D {
    command_sender: flume::Sender<Renderer2DCommand>,
    event_receiver: flume::Receiver<Renderer2DEvent>,
    frame_mailbox: FrameMailbox2D,
    last_size: Option<PhysicalSize<u32>>,
    pending_atlas_rebuild: bool,
    pending_frame_times: Vec<f32>,
    gizmo_buffer: GizmoBuffer,
    gizmo_registry: GizmoRegistry,
}

#[module]
impl RenderHandle2D {
    fn resolve_atlas_ref(
        &mut self,
        path: AssetPath,
    ) -> Option<(AtlasRef, Option<comet_assets::Asset<comet_assets::Image>>)> {
        let _ = self
            .command_sender
            .send(Renderer2DCommand::ResolveAtlasRef(path));
        self.recv_matching_event(Duration::from_millis(5000), |event| {
            matches!(event, Renderer2DEvent::AtlasRef(..))
        })
        .and_then(|event| match event {
            Renderer2DEvent::AtlasRef(Some(atlas_ref), image_handle) => {
                Some((atlas_ref, image_handle))
            }
            _ => None,
        })
    }

    fn ensure_handle_in_atlas(
        &mut self,
        handle: comet_assets::Asset<comet_assets::Image>,
    ) -> Option<AtlasRef> {
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

    pub fn precompute_text_bounds(
        &mut self,
        text: &str,
        font: comet_assets::Asset<comet_assets::Font>,
        font_size: impl Into<comet_math::ScreenUnit>,
    ) -> v2 {
        let _ = self
            .command_sender
            .send(Renderer2DCommand::PrecomputedTextBounds {
                text: text.to_string(),
                font,
                font_size: font_size.into(),
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
                Renderer2DEvent::FrameTime(frame_time) => self.pending_frame_times.push(frame_time),
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
                        Renderer2DEvent::FrameTime(frame_time) => {
                            self.pending_frame_times.push(*frame_time)
                        }
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
        let _ = self
            .command_sender
            .send(Renderer2DCommand::AddRenderPass(desc));
        self.recv_matching_event(Duration::from_millis(5000), |e| {
            matches!(e, Renderer2DEvent::PassAdded(_))
        })
        .and_then(|e| match e {
            Renderer2DEvent::PassAdded(handle) => Some(handle),
            _ => None,
        })
    }

    pub fn remove_render_pass(&mut self, output: PassOutput) {
        let _ = self
            .command_sender
            .send(Renderer2DCommand::RemoveRenderPass(output.0));
        let _ = self.recv_matching_event(Duration::from_millis(5000), |e| {
            matches!(e, Renderer2DEvent::PassRemoved)
        });
    }

    pub fn set_pass_output(
        &mut self,
        label: &str,
        output: Option<PassOutput>,
    ) -> Option<PassOutput> {
        let _ = self
            .command_sender
            .send(Renderer2DCommand::SetPassOutput(label.to_string(), output));
        self.recv_matching_event(Duration::from_millis(5000), |e| {
            matches!(e, Renderer2DEvent::PassOutputSet(_))
        })
        .and_then(|e| match e {
            Renderer2DEvent::PassOutputSet(handle) => handle,
            _ => None,
        })
    }

    pub fn set_pass_render_target(&mut self, label: &str, render_target: Option<&PassOutput>) {
        let _ = self
            .command_sender
            .send(Renderer2DCommand::SetPassRenderTarget(
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
            for (_, render) in
                scene.query_mut::<(&comet_ecs::Transform, &mut comet_ecs::Sprite), ()>()
            {
                if let ImageRef::ResolvedHandle(h, _) = render.texture() {
                    render.set_image_ref(ImageRef::Handle(h));
                }
            }
        }

        let mut selected_camera: Option<(
            [f32; 2],
            f32,
            comet_ecs::Camera,
            comet_ecs::Projection,
            comet_ecs::Screen,
        )> = None;
        for (transform, camera, projection, screen) in scene.query::<(
            &comet_ecs::Transform,
            &comet_ecs::Camera,
            &comet_ecs::Projection,
            &comet_ecs::Screen,
        ), comet_ecs::With<comet_ecs::Camera2d>>(
        ) {
            if !camera.is_enabled() {
                continue;
            }
            let should_replace = selected_camera
                .as_ref()
                .is_none_or(|(_, _, current, _, _)| camera.priority() > current.priority());
            if should_replace {
                selected_camera = Some((
                    [transform.position().x(), transform.position().y()],
                    transform.rotation().as_degrees().z(),
                    *camera,
                    *projection,
                    screen.clone(),
                ));
            }
        }
        let Some((camera_pos, camera_rot, camera, projection, screen)) = selected_camera else {
            return;
        };

        let mut draws = Vec::new();
        let mut referenced_handles = Vec::new();
        for (transform, render) in
            scene.query_mut::<(&comet_ecs::Transform, &mut comet_ecs::Sprite), ()>()
        {
            if !render.is_visible() {
                continue;
            }

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
                rotation_deg: transform.rotation().as_degrees().z(),
                scale: [transform.scale().x(), transform.scale().y()],
                texture: atlas_ref,
                draw_index: render.draw_index(),
                visible: true,
            });
        }

        let mut texts = Vec::new();
        for (transform, text, layout) in scene.query::<(
            &comet_ecs::Transform,
            &comet_ecs::Text,
            Option<&comet_ecs::TextLayout>,
        ), comet_ecs::Without<comet_ecs::ScreenPosition>>(
        ) {
            if !text.is_visible() {
                continue;
            }
            let color = text.color().to_wgpu();
            let anchor = layout.map_or(comet_ecs::Anchor::TopLeft, |layout| layout.anchor());
            let justification = layout.map_or(comet_ecs::TextJustification::Left, |layout| {
                layout.justification()
            });
            texts.push(Text2D {
                position: [transform.position().x(), transform.position().y()],
                anchor,
                justification,
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

        let virtual_resolution = screen.virtual_resolution();
        let mut screen_texts = Vec::new();
        for (position, text, layout) in scene.query::<(
            &comet_ecs::ScreenPosition,
            &comet_ecs::Text,
            Option<&comet_ecs::TextLayout>,
        ), comet_ecs::Without<comet_ecs::Transform>>()
        {
            if !text.is_visible() {
                continue;
            }
            let color = text.color().to_wgpu();
            let text_anchor = layout.map_or(comet_ecs::Anchor::TopLeft, |layout| layout.anchor());
            let justification = layout.map_or(comet_ecs::TextJustification::Left, |layout| {
                layout.justification()
            });
            screen_texts.push(ScreenText2D {
                anchor: position.anchor(),
                offset: [position.offset().x(), position.offset().y()],
                text_anchor,
                justification,
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
            priority: camera.priority(),
            projection,
            virtual_resolution,
            resolution_scaling: screen.resolution_scaling(),
            magnification: projection.magnification(),
            viewport: screen.viewport(),
        };

        self.gizmo_registry.flush(scene, &mut self.gizmo_buffer);
        let gizmo_shapes = std::mem::take(&mut self.gizmo_buffer.shapes);

        let replaced_frame = self.frame_mailbox.lock().unwrap().replace(FramePacket2D {
            camera: camera_packet,
            draws,
            texts,
            screen_texts,
            referenced_handles,
            gizmo_shapes,
        });
        drop(replaced_frame);
    }
}

impl RenderHandle2D {
    fn with_frame_mailbox(
        command_sender: flume::Sender<Renderer2DCommand>,
        event_receiver: flume::Receiver<Renderer2DEvent>,
        frame_mailbox: FrameMailbox2D,
    ) -> Self {
        Self {
            command_sender,
            event_receiver,
            frame_mailbox,
            last_size: None,
            pending_atlas_rebuild: false,
            pending_frame_times: Vec::new(),
            gizmo_buffer: GizmoBuffer::new(),
            gizmo_registry: GizmoRegistry::new(),
        }
    }
}

impl RendererHandle for RenderHandle2D {
    type Command = Renderer2DCommand;
    type Event = Renderer2DEvent;

    fn new(sender: flume::Sender<Self::Command>, receiver: flume::Receiver<Self::Event>) -> Self {
        Self::with_frame_mailbox(sender, receiver, Arc::new(Mutex::new(None)))
    }

    fn poll_event(&self) -> Option<Renderer2DEvent> {
        self.event_receiver.try_recv().ok()
    }
}

impl comet_app::Module for RenderHandle2D {
    fn dependencies(app: &mut comet_app::App)
    where
        Self: Sized,
    {
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
            for frame_time in renderer.pending_frame_times.drain(..) {
                app.record_render_frame_time(frame_time);
            }
            app.reinsert_module(renderer);
        });
    }
}

const BITMAP_TEXT_THRESHOLD: f32 = 18.0;
const FONT_ATLAS_SIZE: u32 = 1024;

const FONT_SHADER: &str = r#"
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
    @location(2) field: f32,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.color = model.color;
    out.field = model.position.z;
    out.clip_position = camera.view_proj * vec4<f32>(model.position.xy, 0.0, 1.0);
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

fn median(a: f32, b: f32, c: f32) -> f32 {
    return max(min(a, b), min(max(a, b), c));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var sample_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    if in.field < -0.5 {
        let dimensions = textureDimensions(t_diffuse);
        let texel = vec2<i32>(in.tex_coords * vec2<f32>(dimensions));
        sample_color = textureLoad(t_diffuse, texel, 0);
    }
    var coverage = sample_color.a;
    if in.field > 0.0 {
        let distance = median(sample_color.r, sample_color.g, sample_color.b);
        let dimensions = vec2<f32>(textureDimensions(t_diffuse));
        let unit_range = vec2<f32>(in.field) / dimensions;
        let screen_texel_size = vec2<f32>(1.0) / max(fwidth(in.tex_coords), vec2<f32>(0.000001));
        let screen_range = max(0.5 * dot(unit_range, screen_texel_size), 1.0);
        coverage = clamp(screen_range * (distance - 0.5) + 0.5, 0.0, 1.0);
    }
    return vec4<f32>(in.color.rgb, coverage * in.color.a);
}
"#;

const SPRITE_SHADER: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) vertex_color: vec4<f32>,
    @location(3) position_size: vec4<f32>,
    @location(4) rotation: vec2<f32>,
    @location(5) uv_min: vec2<f32>,
    @location(6) uv_max: vec2<f32>,
    @location(7) instance_color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let local = model.position.xy * model.position_size.zw;
    let rotated = vec2<f32>(
        local.x * model.rotation.x - local.y * model.rotation.y,
        local.x * model.rotation.y + local.y * model.rotation.x,
    );
    let world_position = model.position_size.xy + rotated;
    out.clip_position = camera.view_proj * vec4<f32>(world_position, model.position.z, 1.0);
    out.tex_coords = mix(model.uv_min, model.uv_max, model.tex_coords);
    out.color = model.vertex_color * model.instance_color;
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords) * in.color;
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
            self.render_state
                .resources_mut()
                .insert_asset_atlas_handle("atlas".to_string(), handle);
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

        let quad_vertices = [
            Vertex::new([-1.0, 1.0, 0.0], [0.0, 0.0], [1.0; 4]),
            Vertex::new([-1.0, -1.0, 0.0], [0.0, 1.0], [1.0; 4]),
            Vertex::new([1.0, -1.0, 0.0], [1.0, 1.0], [1.0; 4]),
            Vertex::new([1.0, 1.0, 0.0], [1.0, 0.0], [1.0; 4]),
        ];
        let quad_indices = [0u16, 1, 3, 1, 2, 3];
        let sprite_geometry = GeometryDescriptor::new(
            vec![
                VertexStreamDescriptor::dynamic("Quad Vertex Buffer", Vertex::desc())
                    .with_initial_data(&quad_vertices),
                VertexStreamDescriptor::dynamic("Instance Buffer", SpriteInstance::desc())
                    .with_initial_capacity_elements::<SpriteInstance>(1024),
            ],
            Some(
                IndexStreamDescriptor::dynamic("Quad Index Buffer", wgpu::IndexFormat::Uint16)
                    .with_initial_data_u16(&quad_indices),
            ),
        );

        self.graph.add_node(
            PassNode::with_geometry(
                "Universal",
                SPRITE_SHADER,
                wgpu::PrimitiveTopology::TriangleList,
                Some(gpu_texture_arc),
                vec![],
                LoadOp::Background,
                sprite_geometry,
            ),
            self.render_state.device(),
            self.render_state.queue(),
            format,
            width,
            height,
        );

        self.graph.add_node(
            PassNode::new(
                "Gizmo",
                GIZMO_SHADER,
                wgpu::PrimitiveTopology::LineList,
                None,
                vec!["Universal", "Font"],
                LoadOp::Load,
            ),
            self.render_state.device(),
            self.render_state.queue(),
            format,
            width,
            height,
        );
    }

    fn ensure_font_atlas(&mut self) -> bool {
        if self
            .render_state
            .resources()
            .get_asset_atlas_handle("font_atlas")
            .is_some()
        {
            return true;
        }

        let mut atlas = comet_assets::TextureAtlas::with_capacity(FONT_ATLAS_SIZE);
        atlas.clear_atlas_image();
        let Some(atlas_handle) = self.asset_provider.add(atlas) else {
            error!("Failed to allocate font atlas asset");
            return false;
        };
        let font_texture = Arc::new(GpuTexture::create_2d_texture(
            self.render_state.device(),
            FONT_ATLAS_SIZE,
            FONT_ATLAS_SIZE,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            wgpu::FilterMode::Linear,
            Some("FontAtlas"),
        ));
        self.render_state
            .resources_mut()
            .insert_asset_atlas_handle("font_atlas".to_string(), atlas_handle);
        self.render_state
            .resources_mut()
            .insert_gpu_texture("font_atlas".to_string(), font_texture.clone());

        let format = self.render_state.config().format;
        let width = self.render_state.config().width;
        let height = self.render_state.config().height;
        self.graph.add_node(
            PassNode::new(
                "Font",
                FONT_SHADER,
                wgpu::PrimitiveTopology::TriangleList,
                Some(font_texture.clone()),
                vec!["Universal"],
                LoadOp::Load,
            ),
            self.render_state.device(),
            self.render_state.queue(),
            format,
            width,
            height,
        );
        self.graph.add_node(
            PassNode::new(
                "ScreenFont",
                FONT_SHADER,
                wgpu::PrimitiveTopology::TriangleList,
                Some(font_texture),
                vec!["Gizmo"],
                LoadOp::Load,
            ),
            self.render_state.device(),
            self.render_state.queue(),
            format,
            width,
            height,
        );
        true
    }

    fn font_variant(
        &self,
        font: comet_assets::Asset<comet_assets::Font>,
        requested_size: comet_math::Px,
    ) -> (FontKey, GlyphRepresentation, f32, f32) {
        let requested = requested_size.pixels().max(1.0);
        let settings = self
            .asset_provider
            .with(font, |font| font.settings())
            .unwrap_or_default();
        let (generation_size, representation) = match settings.rasterization() {
            comet_assets::FontRasterization::Auto if requested > BITMAP_TEXT_THRESHOLD => (
                settings.mtsdf_generation_size().pixels(),
                GlyphRepresentation::Mtsdf,
            ),
            comet_assets::FontRasterization::Mtsdf => (
                settings.mtsdf_generation_size().pixels(),
                GlyphRepresentation::Mtsdf,
            ),
            comet_assets::FontRasterization::Pixel => {
                (requested.round().max(1.0), GlyphRepresentation::Pixel)
            }
            _ => (requested.round().max(1.0), GlyphRepresentation::Bitmap),
        };
        let distance_range = if representation == GlyphRepresentation::Mtsdf {
            settings.mtsdf_range() as f32
        } else {
            0.0
        };
        (
            FontKey {
                index: font.index(),
                generation: font.generation(),
                size_bits: generation_size.to_bits(),
            },
            representation,
            requested / generation_size,
            distance_range,
        )
    }

    fn ensure_font_variant(
        &mut self,
        font: comet_assets::Asset<comet_assets::Font>,
        font_key: FontKey,
        representation: GlyphRepresentation,
    ) -> bool {
        if self.font_cache.contains_key(&font_key) {
            return true;
        }
        if !self.ensure_font_atlas() {
            return false;
        }

        let size = comet_math::px(f32::from_bits(font_key.size_bits));
        let Some(font_data) = self.asset_provider.with(font, |font| font.clone()) else {
            error!("Font handle {:?} is unavailable", font);
            return false;
        };
        let rasterized = match representation {
            GlyphRepresentation::Bitmap | GlyphRepresentation::Pixel => font_data.rasterize(size),
            GlyphRepresentation::Mtsdf => {
                font_data.rasterize_mtsdf(size, font_data.settings().mtsdf_range())
            }
        };
        let Some((mut glyphs, line_height)) = rasterized else {
            return false;
        };
        glyphs.sort_by_key(|glyph| {
            std::cmp::Reverse((
                glyph.render.width().max(glyph.render.height()),
                glyph.render.width() * glyph.render.height(),
            ))
        });
        let atlas_handle = self
            .render_state
            .resources()
            .get_asset_atlas_handle("font_atlas")
            .unwrap();

        for glyph in glyphs {
            let Some(character) = glyph.name.chars().next() else {
                continue;
            };
            let key = GlyphKey {
                font: font_key,
                character,
                representation,
            };
            let name = format!(
                "{}:{}:{}:{}:{}",
                font_key.index,
                font_key.generation,
                font_key.size_bits,
                match representation {
                    GlyphRepresentation::Bitmap => 0,
                    GlyphRepresentation::Mtsdf => 1,
                    GlyphRepresentation::Pixel => 2,
                },
                character as u32,
            );
            let width = glyph.render.width();
            let height = glyph.render.height();
            let insertion = self
                .asset_provider
                .with_mut(atlas_handle, |atlas| {
                    atlas.insert_named(
                        name,
                        width,
                        height,
                        2,
                        glyph.advance,
                        glyph.offset_x,
                        glyph.offset_y,
                    )
                })
                .flatten();
            let Some((blit_position, region)) = insertion else {
                error!("Font atlas is full while inserting a font variant");
                return false;
            };
            if let Some((x, y)) = blit_position {
                if let Some(texture) = self.render_state.resources().get_gpu_texture("font_atlas") {
                    texture.write_region(
                        self.render_state.queue(),
                        x,
                        y,
                        glyph.render.as_bytes(),
                        width,
                        height,
                    );
                }
                #[cfg(debug_assertions)]
                if let Some(glyph_image) = glyph.render.as_rgba8() {
                    for (glyph_x, glyph_y, pixel) in glyph_image.enumerate_pixels() {
                        let debug_pixel = match representation {
                            GlyphRepresentation::Mtsdf => {
                                image::Rgba([pixel[0], pixel[1], pixel[2], 255])
                            }
                            GlyphRepresentation::Bitmap | GlyphRepresentation::Pixel => {
                                image::Rgba([pixel[3], pixel[3], pixel[3], 255])
                            }
                        };
                        self.debug_font_atlas
                            .put_pixel(x + glyph_x, y + glyph_y, debug_pixel);
                    }
                    self.debug_font_atlas_dirty = true;
                }
            }
            self.glyph_cache.insert(key, region);
        }
        self.font_cache.insert(font_key, line_height);
        self.save_debug_font_atlas();
        true
    }

    fn ensure_image_in_atlas(
        &mut self,
        handle: comet_assets::Asset<comet_assets::Image>,
    ) -> Option<AtlasRef> {
        let atlas_handle = self
            .render_state
            .resources()
            .get_asset_atlas_handle("atlas")?;

        if let Some(region) = self
            .asset_provider
            .with(atlas_handle, |atlas| atlas.region_for_handle(handle))
            .flatten()
        {
            return Some(AtlasRef::new(region, atlas_handle));
        }

        let (w, h) = self
            .asset_provider
            .with(handle, |img| (img.width(), img.height()))?;

        let alloc = self
            .asset_provider
            .with_mut(atlas_handle, |atlas| {
                atlas.insert_image_handle(handle, w, h, 1)
            })
            .flatten();

        let (blit_x, blit_y, region) = match alloc {
            Some(r) => r,
            None => {
                self.rebuild_atlas(atlas_handle);
                match self
                    .asset_provider
                    .with_mut(atlas_handle, |atlas| {
                        atlas.insert_image_handle(handle, w, h, 1)
                    })
                    .flatten()
                {
                    Some(r) => r,
                    None => {
                        error!("Failed to insert into atlas even after rebuild");
                        return None;
                    }
                }
            }
        };

        let gpu_texture = self
            .render_state
            .resources()
            .get_gpu_texture("atlas")?
            .clone();
        self.asset_provider.with(handle, |img| {
            gpu_texture.write_region(self.render_state.queue(), blit_x, blit_y, img.data(), w, h);
        });
        self.asset_provider
            .with_mut(handle, |img| img.evict_pixels());

        Some(AtlasRef::new(region, atlas_handle))
    }

    fn rebuild_atlas(&mut self, atlas_handle: comet_assets::Asset<comet_assets::TextureAtlas>) {
        let handles = self
            .asset_provider
            .with(atlas_handle, |atlas| atlas.handle_keys())
            .unwrap_or_default();
        let (old_w, old_h) = self
            .asset_provider
            .with(atlas_handle, |atlas| (atlas.width(), atlas.height()))
            .unwrap_or((512, 512));

        let new_size = (old_w * 2).max(old_h * 2).min(8192);
        info!(
            "Atlas full: rebuilding {}x{} → {}x{}",
            old_w, old_h, new_size, new_size
        );

        self.asset_provider.with_mut(atlas_handle, |atlas| {
            atlas.reset_for_rebuild(new_size, new_size);
        });

        let new_gpu = GpuTexture::create_2d_texture(
            self.render_state.device(),
            new_size,
            new_size,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            wgpu::FilterMode::Nearest,
            Some("Atlas"),
        );

        for h in handles {
            let dims = self
                .asset_provider
                .with(h, |img| (img.width(), img.height()));
            let Some((w, h_px)) = dims else {
                continue;
            };

            let result = self
                .asset_provider
                .with_mut(atlas_handle, |atlas| {
                    atlas.insert_image_handle(h, w, h_px, 1)
                })
                .flatten();
            let Some((blit_x, blit_y, _)) = result else {
                error!("Failed to re-pack handle during atlas rebuild");
                continue;
            };

            let uploaded = self
                .asset_provider
                .with(h, |img| {
                    if !img.is_evicted() {
                        new_gpu.write_region(
                            self.render_state.queue(),
                            blit_x,
                            blit_y,
                            img.data(),
                            w,
                            h_px,
                        );
                        true
                    } else {
                        false
                    }
                })
                .unwrap_or(false);

            if !uploaded {
                let path = self.asset_provider.path_for::<comet_assets::Image>(h);
                if let Some(path) = path {
                    let fs_path = comet_assets::resolve_asset_path(&path);
                    if let Ok(bytes) = std::fs::read(&fs_path) {
                        if let Ok(img) = comet_assets::Image::from_bytes(&bytes, false) {
                            new_gpu.write_region(
                                self.render_state.queue(),
                                blit_x,
                                blit_y,
                                img.data(),
                                w,
                                h_px,
                            );
                        }
                    }
                }
            }
        }

        let new_gpu_arc = Arc::new(new_gpu);
        self.render_state
            .resources_mut()
            .insert_gpu_texture("atlas".to_string(), new_gpu_arc.clone());

        let device = self.render_state.device();
        if let Some(node) = self.graph.pass_mut("Universal") {
            node.set_texture(new_gpu_arc, device);
        }

        let _ = self.event_sender.send(Renderer2DEvent::AtlasRebuilt);
    }

    fn add_pass(&mut self, desc: crate::render_commands::PassDescriptor) -> PassOutput {
        let load = if desc.render_target.is_some() {
            if let LoadOp::Color(_) | LoadOp::Background = desc.load {
                warn!(
                    "pass '{}': render_target with non-Load op, forcing Load",
                    desc.label
                );
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
        if let Some(node) = self.graph.post_process_mut(label) {
            node.set_render_target(render_target);
            self.graph.mark_dirty();
        } else {
            error!("set_pass_render_target: no PostProcessNode '{}'", label);
        }
    }

    fn set_pass_output(&mut self, label: &str, output: Option<PassOutput>) -> Option<PassOutput> {
        if let Some(node) = self.graph.post_process_mut(label) {
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

    fn get_glyph(
        &self,
        character: char,
        font: FontKey,
        representation: GlyphRepresentation,
        distance_range: f32,
    ) -> ResolvedGlyph {
        let region = self
            .glyph_cache
            .get(&GlyphKey {
                font,
                character,
                representation,
            })
            .or_else(|| {
                self.glyph_cache.get(&GlyphKey {
                    font,
                    character: ' ',
                    representation,
                })
            })
            .copied()
            .unwrap_or_else(|| fatal!("No glyph or fallback for '{}' in font atlas", character));
        ResolvedGlyph {
            region,
            representation,
            distance_range,
        }
    }

    pub fn precompute_text_bounds(
        &mut self,
        text: &str,
        font: comet_assets::Asset<comet_assets::Font>,
        size: comet_math::ScreenUnit,
    ) -> v2 {
        let size = size.resolve(self.scale_factor() as f32);
        let mut bounds = v2::ZERO;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        self.add_text_to_buffers(
            text,
            font,
            size,
            1.0,
            v2::ZERO,
            wgpu::Color::WHITE,
            comet_ecs::Anchor::TopLeft,
            comet_ecs::TextJustification::Left,
            &mut bounds,
            &mut vertices,
            &mut indices,
        );
        bounds
    }

    pub fn add_text_to_buffers(
        &mut self,
        text: &str,
        font: comet_assets::Asset<comet_assets::Font>,
        raster_size: comet_math::Px,
        geometry_scale: f32,
        position: comet_math::v2,
        color: wgpu::Color,
        anchor: comet_ecs::Anchor,
        justification: comet_ecs::TextJustification,
        bounds: &mut comet_math::v2,
        vertex_data: &mut Vec<Vertex>,
        index_data: &mut Vec<u16>,
    ) {
        let (cache_key, representation, variant_scale, distance_range) =
            self.font_variant(font, raster_size);
        self.ensure_font_variant(font, cache_key, representation);
        let generation_size = f32::from_bits(cache_key.size_bits);
        let line_height_px = self
            .font_cache
            .get(&cache_key)
            .copied()
            .unwrap_or(generation_size);
        let glyph_scale = geometry_scale * variant_scale;

        let vert_color = [
            color.r as f32,
            color.g as f32,
            color.b as f32,
            color.a as f32,
        ];

        let line_height = line_height_px * glyph_scale;
        let screen_position = position;

        let lines: Vec<Vec<ResolvedGlyph>> = text
            .split('\n')
            .map(|line| {
                line.chars()
                    .map(|character| {
                        let character = if character == '\t' { ' ' } else { character };
                        self.get_glyph(character, cache_key, representation, distance_range)
                    })
                    .collect()
            })
            .collect();

        let line_widths: Vec<f32> = lines
            .iter()
            .map(|line| line.iter().map(|glyph| glyph.region.advance()).sum::<f32>() * glyph_scale)
            .collect();
        let max_line_width = line_widths.iter().copied().fold(0.0, f32::max);
        let block_height = lines.len() as f32 * line_height;
        bounds.set_x(max_line_width);
        bounds.set_y(block_height);

        let (anchor_x, anchor_y) = match anchor {
            comet_ecs::Anchor::TopLeft => (0.0, 0.0),
            comet_ecs::Anchor::TopCenter => (0.5, 0.0),
            comet_ecs::Anchor::TopRight => (1.0, 0.0),
            comet_ecs::Anchor::CenterLeft => (0.0, 0.5),
            comet_ecs::Anchor::Center => (0.5, 0.5),
            comet_ecs::Anchor::CenterRight => (1.0, 0.5),
            comet_ecs::Anchor::BottomLeft => (0.0, 1.0),
            comet_ecs::Anchor::BottomCenter => (0.5, 1.0),
            comet_ecs::Anchor::BottomRight => (1.0, 1.0),
        };
        let block_origin = v2::new(
            screen_position.x() - max_line_width * anchor_x,
            screen_position.y() + block_height * anchor_y,
        );

        let mut y_offset = 0.0f32;

        for (line, line_width) in lines.into_iter().zip(line_widths) {
            let mut x_offset = match justification {
                comet_ecs::TextJustification::Left => 0.0,
                comet_ecs::TextJustification::Center => (max_line_width - line_width) * 0.5,
                comet_ecs::TextJustification::Right => max_line_width - line_width,
            };
            for glyph in line {
                let region = glyph.region;
                let (dim_x, dim_y) = region.dimensions();
                let w = dim_x as f32 * glyph_scale;
                let h = dim_y as f32 * glyph_scale;
                let offset_x = region.offset_x() * glyph_scale;
                let offset_y = region.offset_y() * glyph_scale;
                let field = match glyph.representation {
                    GlyphRepresentation::Bitmap => 0.0,
                    GlyphRepresentation::Mtsdf => glyph.distance_range,
                    GlyphRepresentation::Pixel => -1.0,
                };

                let glyph_left = block_origin.x() + x_offset + offset_x;
                let glyph_top = block_origin.y() - offset_y - y_offset;
                let (glyph_left, glyph_top) = if glyph.representation == GlyphRepresentation::Pixel
                {
                    (glyph_left.round(), glyph_top.round())
                } else {
                    (glyph_left, glyph_top)
                };
                let glyph_right = glyph_left + w;
                let glyph_bottom = glyph_top - h;

                let buffer_size = vertex_data.len() as u16;
                vertex_data.extend_from_slice(&[
                    Vertex::new(
                        [glyph_left, glyph_top, field],
                        [region.u0(), region.v0()],
                        vert_color,
                    ),
                    Vertex::new(
                        [glyph_left, glyph_bottom, field],
                        [region.u0(), region.v1()],
                        vert_color,
                    ),
                    Vertex::new(
                        [glyph_right, glyph_bottom, field],
                        [region.u1(), region.v1()],
                        vert_color,
                    ),
                    Vertex::new(
                        [glyph_right, glyph_top, field],
                        [region.u1(), region.v0()],
                        vert_color,
                    ),
                ]);
                index_data.extend_from_slice(&[
                    buffer_size,
                    buffer_size + 1,
                    buffer_size + 3,
                    buffer_size + 1,
                    buffer_size + 2,
                    buffer_size + 3,
                ]);

                x_offset += region.advance() * glyph_scale;
            }

            y_offset += line_height;
        }
    }

    fn resolve_text_size(
        &self,
        size: comet_ecs::TextSize,
        view: crate::camera::ResolvedCameraViewport,
    ) -> (comet_math::Px, f32) {
        let world_units_per_pixel =
            view.visible_world_size.y() / view.viewport.height.max(1) as f32;
        match size {
            comet_ecs::TextSize::Screen(size) => {
                let raster_size = size.resolve(self.scale_factor() as f32);
                (raster_size, world_units_per_pixel)
            }
            comet_ecs::TextSize::World(world_size) => {
                let raster_size = comet_math::px(world_size / world_units_per_pixel);
                (raster_size, world_units_per_pixel)
            }
        }
    }

    pub fn submit_frame(
        &mut self,
        camera: CameraPacket2D,
        mut draws: Vec<Draw2D>,
        texts: Vec<Text2D>,
        screen_texts: Vec<ScreenText2D>,
        referenced_handles: Vec<comet_assets::Asset<comet_assets::Image>>,
        gizmo_shapes: Vec<GizmoShape>,
    ) {
        if let Some(atlas_handle) = self
            .render_state
            .resources()
            .get_asset_atlas_handle("atlas")
        {
            let any_evicted = self
                .asset_provider
                .with_mut(atlas_handle, |atlas| {
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
                })
                .unwrap_or(false);
            if any_evicted {
                let _ = self.event_sender.send(Renderer2DEvent::AtlasRebuilt);
            }
        }
        let (world_view, screen_view) = self.resolve_camera_views(camera);
        draws.sort_by_key(|draw| draw.draw_index);

        let mut sprite_instances = std::mem::take(&mut self.sprite_instances);
        sprite_instances.clear();
        sprite_instances.reserve(draws.len());
        for draw in draws {
            if !draw.visible {
                continue;
            }

            let region = self.get_texture_region(draw.texture);
            let (width, height) = region.dimensions();
            sprite_instances.push(SpriteInstance::new(
                draw.position,
                [
                    width as f32 * 0.5 * draw.scale[0],
                    height as f32 * 0.5 * draw.scale[1],
                ],
                draw.rotation_deg.to_radians(),
                [region.u0(), region.v0(), region.u1(), region.v1()],
                [1.0; 4],
            ));
        }

        let device = self.render_state.device();
        let queue = self.render_state.queue();

        if let Some(node) = self.graph.pass_mut("Universal") {
            let instance_count = sprite_instances.len() as u32;
            let update_result = node
                .write_vertex_stream(1, &sprite_instances, device, queue)
                .and_then(|()| {
                    node.set_draw_command(DrawCommand::Indexed {
                        indices: 0..6,
                        base_vertex: 0,
                        instances: 0..instance_count,
                    })
                });
            if let Err(error) = update_result {
                error!("Failed to update sprite draw batch: {}", error);
            }
        }
        self.sprite_instances = sprite_instances;

        let mut font_vertex_buffer = std::mem::take(&mut self.world_text_vertices);
        let mut font_index_buffer = std::mem::take(&mut self.world_text_indices);
        font_vertex_buffer.clear();
        font_index_buffer.clear();

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

            let (raster_size, geometry_scale) = self.resolve_text_size(text.size, world_view);
            let mut bounds = v2::ZERO;
            self.add_text_to_buffers(
                &text.content,
                text.font,
                raster_size,
                geometry_scale,
                position,
                color,
                text.anchor,
                text.justification,
                &mut bounds,
                &mut font_vertex_buffer,
                &mut font_index_buffer,
            );
        }

        {
            let device = self.render_state.device();
            let queue = self.render_state.queue();
            if let Some(node) = self.graph.pass_mut("Font") {
                if let Err(error) =
                    node.set_geometry(&font_vertex_buffer, &font_index_buffer, device, queue)
                {
                    error!("Failed to update font draw batch: {}", error);
                }
            }
        }
        self.world_text_vertices = font_vertex_buffer;
        self.world_text_indices = font_index_buffer;

        let mut screen_font_vertex_buffer = std::mem::take(&mut self.screen_text_vertices);
        let mut screen_font_index_buffer = std::mem::take(&mut self.screen_text_indices);
        screen_font_vertex_buffer.clear();
        screen_font_index_buffer.clear();
        let screen_size = screen_view.visible_world_size;
        for text in screen_texts {
            if !text.visible {
                continue;
            }

            let half_width = screen_size.x() * 0.5;
            let half_height = screen_size.y() * 0.5;
            let anchor = match text.anchor {
                comet_ecs::Anchor::TopLeft => v2::new(-half_width, half_height),
                comet_ecs::Anchor::TopCenter => v2::new(0.0, half_height),
                comet_ecs::Anchor::TopRight => v2::new(half_width, half_height),
                comet_ecs::Anchor::CenterLeft => v2::new(-half_width, 0.0),
                comet_ecs::Anchor::Center => v2::ZERO,
                comet_ecs::Anchor::CenterRight => v2::new(half_width, 0.0),
                comet_ecs::Anchor::BottomLeft => v2::new(-half_width, -half_height),
                comet_ecs::Anchor::BottomCenter => v2::new(0.0, -half_height),
                comet_ecs::Anchor::BottomRight => v2::new(half_width, -half_height),
            };
            let position = anchor + v2::new(text.offset[0], -text.offset[1]);
            let color = wgpu::Color {
                r: text.color[0] as f64,
                g: text.color[1] as f64,
                b: text.color[2] as f64,
                a: text.color[3] as f64,
            };
            let (raster_size, geometry_scale) = self.resolve_text_size(text.size, screen_view);
            let mut bounds = v2::ZERO;
            self.add_text_to_buffers(
                &text.content,
                text.font,
                raster_size,
                geometry_scale,
                position,
                color,
                text.text_anchor,
                text.justification,
                &mut bounds,
                &mut screen_font_vertex_buffer,
                &mut screen_font_index_buffer,
            );
        }

        let screen_camera = RenderCamera::new(screen_size, v3::ZERO);
        let mut screen_uniform = CameraUniform::new();
        screen_uniform.update_view_proj(&screen_camera);
        let device = self.render_state.device();
        let queue = self.render_state.queue();
        if let Some(node) = self.graph.pass_mut("ScreenFont") {
            if let Err(error) = node.set_geometry(
                &screen_font_vertex_buffer,
                &screen_font_index_buffer,
                device,
                queue,
            ) {
                error!("Failed to update screen font draw batch: {}", error);
            }
            node.set_camera(&screen_uniform, queue);
            node.set_viewport(Some(screen_view.viewport));
        }
        self.screen_text_vertices = screen_font_vertex_buffer;
        self.screen_text_indices = screen_font_index_buffer;
        self.save_debug_font_atlas();

        // Text processing lazily creates the Font pass, so apply camera uniforms afterward.
        self.apply_camera_view(camera, world_view);

        let mut gizmo_verts = std::mem::take(&mut self.gizmo_vertices);
        let mut gizmo_indices = std::mem::take(&mut self.gizmo_indices);
        gizmo_verts.clear();
        gizmo_indices.clear();

        for shape in gizmo_shapes {
            match shape {
                GizmoShape::Line { start, end, color } => {
                    let c = [color.red(), color.green(), color.blue(), color.alpha()];
                    let base = gizmo_verts.len() as u16;
                    gizmo_verts.push(Vertex::new(
                        [start.x(), start.y(), start.z()],
                        [0.0, 0.0],
                        c,
                    ));
                    gizmo_verts.push(Vertex::new([end.x(), end.y(), end.z()], [0.0, 0.0], c));
                    gizmo_indices.extend_from_slice(&[base, base + 1]);
                }
                GizmoShape::Rect {
                    position,
                    size,
                    color,
                } => {
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
                        base,
                        base + 1,
                        base + 1,
                        base + 2,
                        base + 2,
                        base + 3,
                        base + 3,
                        base,
                    ]);
                }
                GizmoShape::Circle {
                    position,
                    radius,
                    color,
                } => {
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
                GizmoShape::NGon {
                    position,
                    radius,
                    vertices,
                    color,
                } => {
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

        if let Some(node) = self.graph.pass_mut("Gizmo") {
            if let Err(error) = node.set_geometry(&gizmo_verts, &gizmo_indices, device, queue) {
                error!("Failed to update gizmo draw batch: {}", error);
            }
        }
        self.gizmo_vertices = gizmo_verts;
        self.gizmo_indices = gizmo_indices;
    }

    #[cfg(debug_assertions)]
    fn save_debug_font_atlas(&mut self) {
        if !self.debug_font_atlas_dirty {
            return;
        }
        self.debug_font_atlas_dirty = false;
        let image = self.debug_font_atlas.clone();
        let generation =
            DEBUG_FONT_ATLAS_GENERATION.fetch_add(1, std::sync::atomic::Ordering::AcqRel) + 1;
        std::thread::spawn(move || {
            let temporary_path = format!("font_atlas.{generation}.tmp.png");
            if let Err(error) = image.save(&temporary_path) {
                error!("Failed to save debug font atlas: {}", error);
                return;
            }
            if DEBUG_FONT_ATLAS_GENERATION.load(std::sync::atomic::Ordering::Acquire) == generation
            {
                if let Err(error) = std::fs::rename(&temporary_path, "font_atlas.png") {
                    error!("Failed to publish debug font atlas: {}", error);
                }
            } else if let Err(error) = std::fs::remove_file(&temporary_path) {
                error!("Failed to remove stale debug font atlas: {}", error);
            }
        });
    }

    #[cfg(not(debug_assertions))]
    fn save_debug_font_atlas(&mut self) {}

    fn resolve_camera_views(
        &self,
        camera: CameraPacket2D,
    ) -> (
        crate::camera::ResolvedCameraViewport,
        crate::camera::ResolvedCameraViewport,
    ) {
        let output_bounds = if let Some(viewport) = camera.viewport {
            let x = (viewport.x().pixels().floor() as u32)
                .min(self.render_state.config().width.saturating_sub(1));
            let y = (viewport.y().pixels().floor() as u32)
                .min(self.render_state.config().height.saturating_sub(1));
            let width = viewport.width().pixels().round().max(1.0) as u32;
            let height = viewport.height().pixels().round().max(1.0) as u32;
            ResolvedViewport {
                x,
                y,
                width: width.min(self.render_state.config().width - x),
                height: height.min(self.render_state.config().height - y),
            }
        } else {
            ResolvedViewport {
                x: 0,
                y: 0,
                width: self.render_state.config().width,
                height: self.render_state.config().height,
            }
        };
        let scale_factor = self.scale_factor() as f32;
        let virtual_resolution = camera
            .virtual_resolution
            .map(|size| size.resolve(scale_factor))
            .unwrap_or_else(|| {
                v2::new(
                    output_bounds.width as f32 / scale_factor,
                    output_bounds.height as f32 / scale_factor,
                )
            });
        let resolved = resolve_camera_viewport(
            virtual_resolution,
            camera.resolution_scaling,
            camera.magnification,
            output_bounds,
        );
        let screen_view = resolve_camera_viewport(
            virtual_resolution,
            camera.resolution_scaling,
            1.0,
            output_bounds,
        );

        (resolved, screen_view)
    }

    fn apply_camera_view(
        &mut self,
        camera: CameraPacket2D,
        resolved: crate::camera::ResolvedCameraViewport,
    ) {
        let view_proj: [[f32; 4]; 4] = match camera.projection {
            comet_ecs::Projection::Custom(matrix) => matrix.into(),
            _ => RenderCamera::new(
                resolved.visible_world_size,
                v3::new(camera.position[0], camera.position[1], 0.0),
            )
            .build_view_projection_matrix()
            .into(),
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.set_view_proj(view_proj);

        let queue = self.render_state.queue();

        if let Some(node) = self.graph.pass_mut("Universal") {
            node.set_camera(&camera_uniform, queue);
            node.set_viewport(Some(resolved.viewport));
        }
        if let Some(node) = self.graph.pass_mut("Font") {
            node.set_camera(&camera_uniform, queue);
            node.set_viewport(Some(resolved.viewport));
        }
        if let Some(node) = self.graph.pass_mut("Gizmo") {
            node.set_camera(&camera_uniform, queue);
            node.set_viewport(Some(resolved.viewport));
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
            glyph_cache: std::collections::HashMap::new(),
            sprite_instances: Vec::new(),
            world_text_vertices: Vec::new(),
            world_text_indices: Vec::new(),
            screen_text_vertices: Vec::new(),
            screen_text_indices: Vec::new(),
            gizmo_vertices: Vec::new(),
            gizmo_indices: Vec::new(),
            #[cfg(debug_assertions)]
            debug_font_atlas: image::RgbaImage::from_pixel(
                FONT_ATLAS_SIZE,
                FONT_ATLAS_SIZE,
                image::Rgba([0, 0, 0, 255]),
            ),
            #[cfg(debug_assertions)]
            debug_font_atlas_dirty: false,
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
                let atlas_ref = self
                    .render_state
                    .resources()
                    .get_asset_atlas_handle("atlas")
                    .and_then(|handle| {
                        self.asset_provider
                            .with(handle, |atlas| {
                                atlas
                                    .textures()
                                    .get(path.as_str())
                                    .copied()
                                    .map(|region| AtlasRef::new(region, handle))
                            })
                            .flatten()
                    });

                let mut dynamic_image_handle: Option<comet_assets::Asset<comet_assets::Image>> =
                    None;
                let atlas_ref = atlas_ref.or_else(|| {
                    let fs_path = comet_assets::resolve_asset_path(path.as_str());
                    let bytes = std::fs::read(&fs_path).ok()?;
                    let image = comet_assets::Image::from_bytes(&bytes, false).ok()?;
                    let image_handle = self.asset_provider.add(image)?;
                    self.asset_provider
                        .track_for_reload::<comet_assets::Image>(image_handle, path.clone());
                    let result = self.ensure_image_in_atlas(image_handle);
                    if result.is_some() {
                        dynamic_image_handle = Some(image_handle);
                    }
                    result
                });

                let _ = self
                    .event_sender
                    .send(Renderer2DEvent::AtlasRef(atlas_ref, dynamic_image_handle));
            }
            Renderer2DCommand::EnsureHandleInAtlas(handle) => {
                let atlas_ref = self.ensure_image_in_atlas(handle);
                let _ = self
                    .event_sender
                    .send(Renderer2DEvent::AtlasRef(atlas_ref, None));
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

            Renderer2DCommand::AddRenderPass(desc) => {
                let pass_output = self.add_pass(desc);
                let _ = self
                    .event_sender
                    .send(Renderer2DEvent::PassAdded(pass_output));
            }
            Renderer2DCommand::RemoveRenderPass(label) => {
                self.remove_pass(&label);
                let _ = self.event_sender.send(Renderer2DEvent::PassRemoved);
            }
            Renderer2DCommand::SetPassOutput(label, output) => {
                let handle = self.set_pass_output(&label, output);
                let _ = self
                    .event_sender
                    .send(Renderer2DEvent::PassOutputSet(handle));
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
        let _ = self
            .event_sender
            .send(Renderer2DEvent::FrameTime(self.delta_time));
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
    frame_mailbox: FrameMailbox2D,
}

impl comet_window::ErasedRenderer for ErasedRenderer2D {
    fn init_assets(&mut self, app: &comet_app::App) {
        self.renderer.init_assets(app);
    }
    fn drain_commands(&mut self) {
        while let Ok(cmd) = self.cmd_rx.try_recv() {
            self.renderer.apply_command(cmd);
        }

        let frame = self.frame_mailbox.lock().unwrap().take();
        if let Some(frame) = frame {
            self.renderer.submit_frame(
                frame.camera,
                frame.draws,
                frame.texts,
                frame.screen_texts,
                frame.referenced_handles,
                frame.gizmo_shapes,
            );
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
    fn dependencies(app: &mut App)
    where
        Self: Sized,
    {
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
                let frame_mailbox = Arc::new(Mutex::new(None));

                let renderer = Renderer2D::new(window, clear_color, evt_tx);
                let handle =
                    RenderHandle2D::with_frame_mailbox(cmd_tx, evt_rx, frame_mailbox.clone());

                let erased: Box<dyn comet_window::ErasedRenderer> = Box::new(ErasedRenderer2D {
                    renderer,
                    cmd_rx,
                    frame_mailbox,
                });
                let add_handle: Box<dyn FnOnce(&mut comet_app::App) + Send> =
                    Box::new(move |app| {
                        app.add_module(handle);
                    });

                (erased, add_handle)
            }));
    }
}
