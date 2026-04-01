use comet_assets::AtlasRef;
use winit::dpi::PhysicalSize;
use crate::render_pass::PassOutput;

pub enum Renderer2DEvent {
    AtlasRef(Option<AtlasRef>, Option<comet_assets::Asset<comet_assets::Image>>),
    AtlasRebuilt,
    Size(PhysicalSize<u32>),
    ScaleFactor(f64),
    PrecomputedTextBounds { width: f32, height: f32 },
    FrameTime(f32),
    PassAdded(Option<PassOutput>),
}
