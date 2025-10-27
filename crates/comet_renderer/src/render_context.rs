use crate::{batch::Batch, render_resources::RenderResources};
use comet_colors::Color;
use std::{collections::HashMap, sync::Arc};
use winit::{dpi::PhysicalSize, window::Window};

pub struct RenderContext<'a> {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'a>,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    scale_factor: f64,
    clear_color: wgpu::Color,
    render_pipelines: HashMap<String, wgpu::RenderPipeline>,
    batches: HashMap<String, Batch>,
    resources: RenderResources,
}

impl<'a> RenderContext<'a> {
    pub fn new(window: Arc<Window>, clear_color: Option<impl Color>) -> Self {
        let size = window.inner_size();
        let scale_factor = window.scale_factor();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
            },
            None,
        ))
        .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let clear_color = match clear_color {
            Some(color) => color.to_wgpu(),
            None => wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        };

        Self {
            device,
            queue,
            surface,
            config,
            size,
            scale_factor,
            clear_color,
            render_pipelines: HashMap::new(),
            batches: HashMap::new(),
            resources: RenderResources::new(),
        }
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }

    pub fn configure_surface(&mut self) {
        self.surface.configure(&self.device, &self.config);
    }

    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut wgpu::SurfaceConfiguration {
        &mut self.config
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub fn set_size(&mut self, new_size: PhysicalSize<u32>) {
        self.size = new_size
    }

    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor
    }

    pub fn clear_color(&self) -> wgpu::Color {
        self.clear_color
    }

    pub fn get_pipeline(&self, label: String) -> Option<&wgpu::RenderPipeline> {
        self.render_pipelines.get(&label)
    }

    pub fn get_batch(&self, label: String) -> Option<&Batch> {
        self.batches.get(&label)
    }

    pub fn resources(&self) -> &RenderResources {
        &self.resources
    }

    pub fn resources_mut(&mut self) -> &mut RenderResources {
        &mut self.resources
    }
}
