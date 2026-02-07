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
    pub texture: &'static str,
    pub draw_index: u32,
    pub visible: bool,
}

pub enum Renderer2DCommand {
    Clear,
    InitAtlas,
    InitAtlasFromPaths(Vec<String>),
    SubmitFrame {
        camera: CameraPacket2D,
        draws: Vec<Draw2D>,
    },
}
