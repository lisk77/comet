use crate::renderer::Renderer;
use crate::{camera::CameraManager, render_context::RenderContext};
use comet_colors::Color;
use comet_resources::graphic_resource_manager::GraphicResourceManager;
use std::sync::Arc;
use winit::{dpi::PhysicalSize, window::Window};

pub struct Renderer2D<'a> {
    render_context: RenderContext<'a>,
    resource_manager: GraphicResourceManager,
    camera_manager: CameraManager,
    delta_time: f32,
}

impl<'a> Renderer for Renderer2D<'a> {
    fn new(window: Arc<Window>, clear_color: Option<impl Color>) -> Self {
        Self {
            render_context: RenderContext::new(window, clear_color),
            resource_manager: GraphicResourceManager::new(),
            camera_manager: CameraManager::new(),
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
        todo!()
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        todo!()
    }
}

