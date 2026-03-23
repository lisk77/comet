use comet_assets::AtlasRef;

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

pub enum Renderer2DCommand {
    Clear,
    InitAtlas,
    InitAtlasFromPaths(Vec<String>),
    ResolveAtlasRef(&'static str),
    Size,
    ScaleFactor,
    PrecomputedTextBounds {
        text: String,
        font: comet_assets::Asset<comet_assets::Font>,
        font_size: f32,
    },
    SubmitFrame(CameraPacket2D, Vec<Draw2D>, Vec<Text2D>),
}
