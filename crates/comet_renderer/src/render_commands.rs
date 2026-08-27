use crate::render_pass::LoadOp;
use comet_assets::AtlasRef;
use comet_ecs::Projection;
use comet_gizmos::GizmoShape;

#[derive(Clone, Copy, Debug)]
pub struct CameraPacket2D {
    pub position: [f32; 2],
    pub rotation_deg: f32,
    pub priority: i32,
    pub projection: Projection,
    pub virtual_resolution: Option<comet_math::ScreenSize>,
    pub resolution_scaling: comet_ecs::ResolutionScaling,
    pub magnification: f32,
    pub viewport: Option<comet_ecs::CameraViewport>,
}

#[derive(Clone, Copy, Debug)]
pub struct Draw2D {
    pub position: [f32; 2],
    pub rotation_deg: f32,
    pub scale: [f32; 2],
    pub texture: AtlasRef,
    pub draw_index: u32,
    pub visible: bool,
}

#[derive(Clone, Debug)]
pub struct Text2D {
    pub position: [f32; 2],
    pub anchor: comet_ecs::Anchor,
    pub justification: comet_ecs::TextJustification,
    pub content: String,
    pub font: comet_assets::Asset<comet_assets::Font>,
    pub size: comet_ecs::TextSize,
    pub color: [f32; 4],
    pub visible: bool,
}

#[derive(Clone, Debug)]
pub struct ScreenText2D {
    pub anchor: comet_ecs::Anchor,
    pub offset: [f32; 2],
    pub text_anchor: comet_ecs::Anchor,
    pub justification: comet_ecs::TextJustification,
    pub content: String,
    pub font: comet_assets::Asset<comet_assets::Font>,
    pub size: comet_ecs::TextSize,
    pub color: [f32; 4],
    pub visible: bool,
}

pub(crate) struct FramePacket2D {
    #[cfg(feature = "diagnostics")]
    pub(crate) sequence: u64,
    #[cfg(feature = "diagnostics")]
    pub(crate) produced_at: std::time::Instant,
    #[cfg(feature = "diagnostics")]
    pub(crate) replaced_frames: u64,
    pub(crate) camera: CameraPacket2D,
    pub(crate) draws: Vec<Draw2D>,
    pub(crate) texts: Vec<Text2D>,
    pub(crate) screen_texts: Vec<ScreenText2D>,
    pub(crate) referenced_handles: Vec<comet_assets::Asset<comet_assets::Image>>,
    pub(crate) gizmo_shapes: Vec<GizmoShape>,
}

pub struct PassDescriptor {
    pub label: String,
    pub inputs: Vec<String>,
    pub output: Option<String>,
    pub render_target: Option<String>,
    pub output_format: Option<wgpu::TextureFormat>,
    pub shader_src: String,
    pub load: LoadOp,
}

pub enum Renderer2DCommand {
    Clear,
    ResolveAtlasRef(comet_assets::AssetPath),
    EnsureHandleInAtlas(comet_assets::Asset<comet_assets::Image>),
    Size,
    ScaleFactor,
    PrecomputedTextBounds {
        text: String,
        font: comet_assets::Asset<comet_assets::Font>,
        font_size: comet_math::ScreenUnit,
    },

    AddRenderPass(PassDescriptor),
    RemoveRenderPass(String),
    SetPassOutput(String, Option<crate::render_pass::PassOutput>),
    SetPassRenderTarget(String, Option<String>),
}
