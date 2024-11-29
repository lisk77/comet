use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::window::Window;
use comet_colors::LinearRgba;

pub trait Renderer: Sized {
	async fn new(window: Arc<Window>, clear_color: Option<LinearRgba>) -> Self;
	fn size(&self) -> PhysicalSize<u32>;
	fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);
	fn update(&mut self) -> f32;
	fn render(&mut self) -> Result<(), wgpu::SurfaceError>;
}