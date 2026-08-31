use crate::gizmo_registry::GizmoRegistry;
use crate::{
    camera::{resolve_camera_viewport, CameraUniform, RenderCamera, ResolvedViewport},
    gpu_mesh::{GpuMesh, MeshVertexAttribute},
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
use comet_log::{error, fatal, info, warn};
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
struct FontVariantKey {
    font: FontKey,
    representation: GlyphRepresentation,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct GlyphKey {
    font: FontKey,
    character: char,
    representation: GlyphRepresentation,
}
use winit::{dpi::PhysicalSize, window::Window};

type FrameMailbox2D = Arc<Mutex<Option<FramePacket2D>>>;

pub struct Renderer2D {
    render_state: RenderState,
    #[cfg(feature = "diagnostics")]
    diagnostics: Option<diagnostics::Renderer2DDiagnosticsPublisher>,
    #[cfg(feature = "diagnostics")]
    frame_diagnostics: diagnostics::Renderer2DDiagnostics,
    #[cfg(feature = "diagnostics")]
    latest_snapshot_produced_at: Option<Instant>,
    #[cfg(feature = "diagnostics")]
    latest_snapshot_sequence: Option<u64>,
    #[cfg(feature = "diagnostics")]
    last_rendered_snapshot_sequence: Option<u64>,
    asset_provider: comet_assets::AssetProvider,
    graph: RenderGraph,
    last_frame_time: std::time::Instant,
    delta_time: f32,
    event_sender: flume::Sender<Renderer2DEvent>,
    font_cache: std::collections::HashMap<FontVariantKey, f32>,
    glyph_cache: std::collections::HashMap<GlyphKey, TextureRegion>,
    font_job_sender: flume::Sender<text::FontVariantJob>,
    font_result_receiver: flume::Receiver<text::FontVariantResult>,
    pending_font_variants: std::collections::HashSet<FontVariantKey>,
    failed_font_variants: std::collections::HashSet<FontVariantKey>,
    sprite_instances: Vec<SpriteInstance>,
    sprite_instance_staging: Vec<SpriteInstance>,
    sprite_mesh_runs: Vec<(comet_ecs::Mesh, std::ops::Range<u32>)>,
    sprite_gpu_draws: Vec<(Arc<GpuMesh>, std::ops::Range<u32>)>,
    world_text_vertices: Vec<Vertex>,
    world_text_indices: Vec<u16>,
    world_text_staging_vertices: Vec<Vertex>,
    world_text_staging_indices: Vec<u16>,
    screen_text_vertices: Vec<Vertex>,
    screen_text_indices: Vec<u16>,
    screen_text_staging_vertices: Vec<Vertex>,
    screen_text_staging_indices: Vec<u16>,
    gizmo_vertices: Vec<Vertex>,
    gizmo_indices: Vec<u16>,
    gizmo_staging_vertices: Vec<Vertex>,
    gizmo_staging_indices: Vec<u16>,
}

#[cfg(feature = "diagnostics")]
mod diagnostics;
mod frame;
mod handle;
mod pipeline;
mod runtime;
mod shaders;
mod text;

pub use handle::{RenderHandle2D, RenderHandle2DExt};
pub use runtime::Renderer2DModule;
