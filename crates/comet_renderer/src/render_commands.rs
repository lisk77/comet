use comet_assets::AtlasRef;
use comet_gizmos::GizmoShape;
use crate::render_pass::LoadOp;

#[derive(Clone, Copy, Debug)]
pub struct CameraPacket2D {
    pub position: [f32; 2],
    pub rotation_deg: f32,
    pub zoom: f32,
    pub dimensions: [f32; 2],
    pub priority: u8,
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
    pub content: String,
    pub font: comet_assets::Asset<comet_assets::Font>,
    pub size: f32,
    pub color: [f32; 4],
    pub visible: bool,
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
    ResolveAtlasRef(&'static str),
    EnsureHandleInAtlas(comet_assets::Asset<comet_assets::Image>),
    Size,
    ScaleFactor,
    PrecomputedTextBounds {
        text: String,
        font: comet_assets::Asset<comet_assets::Font>,
        font_size: f32,
    },
    SubmitFrame(CameraPacket2D, Vec<Draw2D>, Vec<Text2D>, Vec<comet_assets::Asset<comet_assets::Image>>, Vec<GizmoShape>),
    AddRenderPass(PassDescriptor),
    RemoveRenderPass(String),
    SetPassOutput(String, Option<crate::render_pass::PassOutput>),
    SetPassRenderTarget(String, Option<String>),
}
