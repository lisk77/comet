use comet_colors::Color;
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub trait Renderer: Sized + Send + Sync {
    fn new(window: Arc<Window>, clear_color: Option<impl Color>) -> Self;
    fn size(&self) -> PhysicalSize<u32>;
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);
    fn scale_factor(&self) -> f64;
    fn set_scale_factor(&mut self, scale_factor: f64);
    fn update(&mut self) -> f32;
    fn render(&mut self) -> Result<(), wgpu::SurfaceError>;
}

