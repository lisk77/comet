mod render_context;

use render_context::*;

use crate::renderer::Renderer;
use comet_colors::Color;
use comet_resources::{graphic_resource_manager::GraphicResourceManager, Vertex};
use std::iter;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct Renderer2D_<'a> {
    render_context: RenderContext<'a>,
    universal_render_pipeline: wgpu::RenderPipeline,
    graphic_resource_manager: GraphicResourceManager,
    vertex_vec: Vec<Vertex>,
    vertex_buffer: wgpu::Buffer,
    index_vec: Vec<u32>,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    clear_color: wgpu::Color,
}

impl<'a> Renderer2D_<'a> {
    pub fn new(window: Arc<Window>, clear_color: Option<impl Color>) -> Renderer2D_<'a> {
        let render_context = RenderContext::new(window.clone(), clear_color);
        let graphic_resource_manager = GraphicResourceManager::new();
        let clear_color = match clear_color {
            Some(color) => color.to_wgpu(),
            None => wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        };

        let universal_renderpipeline_module =
            render_context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Universal Render Pipeline Shader Module"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("base.wgsl").into()),
                });

        let universal_renderpipeline_layout =
            render_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Universal Render Pipeline Layout"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let universal_render_pipeline =
            render_context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Universal Render Pipeline"),
                    layout: Some(&universal_renderpipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &universal_renderpipeline_module,
                        entry_point: "vs_main",
                        buffers: &[Vertex::desc()],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &universal_renderpipeline_module,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: render_context.config.format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    cache: None,
                });

        let vertex_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: &[],
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });

        let index_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: &[],
                    usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                });

        Self {
            render_context,
            universal_render_pipeline,
            graphic_resource_manager,
            vertex_buffer,
            vertex_vec: vec![],
            index_buffer,
            index_vec: vec![],
            num_indices: 0,
            clear_color,
        }
    }
}

impl<'a> Renderer for Renderer2D_<'a> {
    fn new(window: Arc<Window>, clear_color: Option<impl Color>) -> Renderer2D_<'a> {
        Self::new(window, clear_color)
    }

    fn size(&self) -> PhysicalSize<u32> {
        self.render_context.size()
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.render_context.resize(new_size)
    }

    fn update(&mut self) -> f32 {
        self.render_context.update()
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.render_context.surface.get_current_texture()?;
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.render_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Universal Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.universal_render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.render_context
            .queue
            .submit(iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}
