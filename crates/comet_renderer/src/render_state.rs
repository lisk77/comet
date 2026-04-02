use crate::{batch::Batch, gpu_texture::GpuTexture, render_resources::RenderResources, Vertex};
use comet_colors::Color;
use std::{collections::HashMap, sync::Arc};
use winit::{dpi::PhysicalSize, window::Window};

pub struct RenderState {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    scale_factor: f64,
    clear_color: wgpu::Color,
    render_pipelines: HashMap<String, wgpu::RenderPipeline>,
    batches: HashMap<String, Batch>,
    resources: RenderResources,
}

impl RenderState {
    pub fn new(window: Arc<Window>, clear_color: Option<impl Color>) -> Self {
        let size = window.inner_size();
        let scale_factor = window.scale_factor();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

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
            present_mode: wgpu::PresentMode::Fifo,
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
            window,
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

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn surface(&self) -> &wgpu::Surface<'static> {
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

    pub fn insert_pipeline(&mut self, label: String, pipeline: wgpu::RenderPipeline) {
        self.render_pipelines.insert(label, pipeline);
    }

    pub fn get_pipeline(&self, label: String) -> Option<&wgpu::RenderPipeline> {
        self.render_pipelines.get(&label)
    }

    pub fn get_batch(&self, label: String) -> Option<&Batch> {
        self.batches.get(&label)
    }

    pub fn get_batch_mut(&mut self, label: String) -> Option<&mut Batch> {
        self.batches.get_mut(&label)
    }

    pub fn new_batch(&mut self, label: String, vertex_data: Vec<Vertex>, index_data: Vec<u16>) {
        self.batches.insert(
            label.clone(),
            Batch::new(label, &self.device, vertex_data, index_data),
        );
    }

    pub fn update_batch_buffers(
        &mut self,
        label: String,
        vertex_data: Vec<Vertex>,
        index_data: Vec<u16>,
    ) {
        if let Some(batch) = self.batches.get_mut(&label) {
            batch.update_vertex_buffer(&self.device, &self.queue, vertex_data);
            batch.update_index_buffer(&self.device, &self.queue, index_data);
        } else {
            let batch = Batch::new(label.clone(), &self.device, vertex_data, index_data);
            self.batches.insert(label, batch);
        }
    }

    pub fn resources(&self) -> &RenderResources {
        &self.resources
    }

    pub fn resources_mut(&mut self) -> &mut RenderResources {
        &mut self.resources
    }

    pub fn create_intermediate_texture(&mut self, name: String, width: u32, height: u32, format: wgpu::TextureFormat) {
        let gpu_tex = GpuTexture::create_2d_texture(
            &self.device,
            width,
            height,
            format,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            wgpu::FilterMode::Linear,
            Some(&name),
        );
        self.resources.insert_gpu_texture(name, Arc::new(gpu_tex));
    }
}
