use comet_assets::AtlasRef;
use winit::dpi::PhysicalSize;

pub enum Renderer2DEvent {
    AtlasRef(Option<AtlasRef>),
    Size(PhysicalSize<u32>),
    ScaleFactor(f64),
    PrecomputedTextBounds { width: f32, height: f32 },
    FrameTime(f32),
}
