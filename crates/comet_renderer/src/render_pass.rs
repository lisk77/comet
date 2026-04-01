use std::sync::Arc;
use crate::{gpu_texture::GpuTexture, render_context::RenderContext};

#[derive(Debug, Clone)]
pub struct PassOutput(pub(crate) String);

impl PassOutput {
    pub fn name(&self) -> &str {
        &self.0
    }
}

pub struct RenderPass {
    pub label: String,
    pub inputs: Vec<String>,
    pub output: Option<String>,
    pub output_format: Option<wgpu::TextureFormat>,
    pub execute: Box<
        dyn Fn(String, &mut RenderContext, &mut wgpu::CommandEncoder, &wgpu::TextureView, &[Arc<GpuTexture>])
            + Send
            + Sync,
    >,
}

impl RenderPass {
    pub fn new(
        label: String,
        inputs: Vec<String>,
        output: Option<String>,
        output_format: Option<wgpu::TextureFormat>,
        execute: Box<
            dyn Fn(String, &mut RenderContext, &mut wgpu::CommandEncoder, &wgpu::TextureView, &[Arc<GpuTexture>])
                + Send
                + Sync,
        >,
    ) -> Self {
        Self { label, inputs, output, output_format, execute }
    }
}

pub fn universal_clear_execute(
    label: String,
    ctx: &mut RenderContext,
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    _inputs: &[Arc<GpuTexture>],
) {
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(format!("{} Render Pass", label.clone()).as_str()),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(ctx.clear_color()),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
    });

    render_pass.set_pipeline(&ctx.get_pipeline(label.clone()).unwrap());

    let groups = ctx.resources().get_bind_groups(&label).unwrap();
    for i in 0..groups.len() {
        render_pass.set_bind_group(i as u32, groups.get(i).unwrap(), &[]);
    }

    render_pass.set_vertex_buffer(
        0,
        ctx.get_batch(label.clone())
            .unwrap()
            .vertex_buffer()
            .slice(..),
    );

    render_pass.set_index_buffer(
        ctx.get_batch(label.clone())
            .unwrap()
            .index_buffer()
            .slice(..),
        wgpu::IndexFormat::Uint16,
    );

    render_pass.draw_indexed(
        0..ctx.get_batch(label.clone()).unwrap().num_indices(),
        0,
        0..1,
    );
}

pub fn universal_load_execute(
    label: String,
    ctx: &mut RenderContext,
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    _inputs: &[Arc<GpuTexture>],
) {
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(format!("{} Render Pass", label.clone()).as_str()),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
    });

    render_pass.set_pipeline(&ctx.get_pipeline(label.clone()).unwrap());

    let groups = ctx.resources().get_bind_groups(&label).unwrap();
    for i in 0..groups.len() {
        render_pass.set_bind_group(i as u32, groups.get(i).unwrap(), &[]);
    }

    render_pass.set_vertex_buffer(
        0,
        ctx.get_batch(label.clone())
            .unwrap()
            .vertex_buffer()
            .slice(..),
    );

    render_pass.set_index_buffer(
        ctx.get_batch(label.clone())
            .unwrap()
            .index_buffer()
            .slice(..),
        wgpu::IndexFormat::Uint16,
    );

    render_pass.draw_indexed(
        0..ctx.get_batch(label.clone()).unwrap().num_indices(),
        0,
        0..1,
    );
}
