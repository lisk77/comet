use winit::dpi::PhysicalSize;

pub enum Renderer2DEvent {
    Size (PhysicalSize<u32>),
    PrecomputedTextBounds { width: f32, height: f32 },
    FrameTime(f32)
}