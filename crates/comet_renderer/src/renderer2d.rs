use crate::renderer::Renderer;
use crate::{camera::CameraManager, render_context::RenderContext, render_pass::RenderPass};
use comet_colors::Color;
use comet_resources::graphic_resource_manager::GraphicResourceManager;
use std::sync::Arc;
use winit::{dpi::PhysicalSize, window::Window};

pub struct Renderer2D<'a> {
    render_context: RenderContext<'a>,
    resource_manager: GraphicResourceManager,
    camera_manager: CameraManager,
    render_passes: Vec<RenderPass>,
    last_frame_time: std::time::Instant,
    delta_time: f32,
}

impl<'a> Renderer for Renderer2D<'a> {
    fn new(window: Arc<Window>, clear_color: Option<impl Color>) -> Self {
        Self {
            render_context: RenderContext::new(window, clear_color),
            resource_manager: GraphicResourceManager::new(),
            camera_manager: CameraManager::new(),
            render_passes: Vec::new(),
            last_frame_time: std::time::Instant::now(),
            delta_time: 0.0,
        }
    }

    fn size(&self) -> PhysicalSize<u32> {
        self.render_context.size()
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.render_context.set_size(new_size);
            self.render_context.config_mut().width = new_size.width;
            self.render_context.config_mut().height = new_size.height;
            self.render_context.configure_surface();
        }
    }

    fn scale_factor(&self) -> f64 {
        self.render_context.scale_factor()
    }

    fn set_scale_factor(&mut self, scale_factor: f64) {
        self.render_context.set_scale_factor(scale_factor);
    }

    fn update(&mut self) -> f32 {
        let now = std::time::Instant::now();
        self.delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        self.delta_time
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.render_context.surface().get_current_texture()?;
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.render_context
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        for pass in &self.render_passes {
            (pass.execute)(&mut self.render_context, &mut encoder, &output_view);
        }

        self.render_context
            .queue()
            .submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())
    }
}
