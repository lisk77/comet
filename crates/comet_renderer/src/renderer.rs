use comet_colors::Color;
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub trait RendererHandle {
    type Command: Send + 'static;
    type Event: Send + 'static;

    fn new(sender: flume::Sender<Self::Command>, receiver: flume::Receiver<Self::Event>) -> Self; 
    fn poll_event(&self) -> Option<Self::Event>; 
}

pub trait Renderer: Sized + Send + Sync {
    type Handle: RendererHandle;

    fn new(
        window: Arc<Window>,
        clear_color: Option<impl Color>,
        event_sender: flume::Sender<<Self::Handle as RendererHandle>::Event>,
    ) -> Self;
    fn apply_command(&mut self, command: <Self::Handle as RendererHandle>::Command);
    fn window(&self) -> &Window;
    fn size(&self) -> PhysicalSize<u32>;
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);
    fn scale_factor(&self) -> f64;
    fn set_scale_factor(&mut self, scale_factor: f64);
    fn update(&mut self) -> f32;
    fn render(&mut self) -> Result<(), wgpu::SurfaceError>;
}
