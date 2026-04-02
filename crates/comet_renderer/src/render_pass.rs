use std::sync::Arc;
use crate::render_state::RenderState;

#[derive(Debug, Clone)]
pub struct PassOutput(pub(crate) String);

impl PassOutput {
    pub fn name(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub enum LoadOp {
    Background,
    Color(wgpu::Color),
    Load,
}

pub struct PassCache {
    pub layouts: Vec<Arc<wgpu::BindGroupLayout>>,
    pub sampler: Arc<wgpu::Sampler>,
    pub bind_groups: Option<Vec<Arc<wgpu::BindGroup>>>,
}

impl PassCache {
    pub fn new(layouts: Vec<Arc<wgpu::BindGroupLayout>>, sampler: Arc<wgpu::Sampler>) -> Self {
        Self { layouts, sampler, bind_groups: None }
    }

    pub fn invalidate(&mut self) {
        self.bind_groups = None;
    }
}

pub struct RenderPass {
    pub label: String,
    pub inputs: Vec<String>,
    pub output: Option<String>,
    pub render_target: Option<String>,
    pub output_format: Option<wgpu::TextureFormat>,
    pub load: LoadOp,
    pub cache: Option<PassCache>,
    pub execute: Box<
        dyn for<'rpass> Fn(String, &mut RenderState, &mut wgpu::RenderPass<'rpass>, &[&wgpu::BindGroup])
            + Send
            + Sync,
    >,
}

impl RenderPass {
    pub fn new(
        label: String,
        inputs: Vec<String>,
        output: Option<String>,
        render_target: Option<String>,
        output_format: Option<wgpu::TextureFormat>,
        load: LoadOp,
        cache: Option<PassCache>,
        execute: Box<
            dyn for<'rpass> Fn(String, &mut RenderState, &mut wgpu::RenderPass<'rpass>, &[&wgpu::BindGroup])
                + Send
                + Sync,
        >,
    ) -> Self {
        Self { label, inputs, output, render_target, output_format, load, cache, execute }
    }
}

pub fn universal_execute(
    label: String,
    ctx: &mut RenderState,
    rpass: &mut wgpu::RenderPass<'_>,
    _inputs: &[&wgpu::BindGroup],
) {
    rpass.set_pipeline(ctx.get_pipeline(label.clone()).unwrap());
    let groups = ctx.resources().get_bind_groups(&label).unwrap();
    for i in 0..groups.len() {
        rpass.set_bind_group(i as u32, groups.get(i).unwrap(), &[]);
    }
    rpass.set_vertex_buffer(0, ctx.get_batch(label.clone()).unwrap().vertex_buffer().slice(..));
    rpass.set_index_buffer(ctx.get_batch(label.clone()).unwrap().index_buffer().slice(..), wgpu::IndexFormat::Uint16);
    rpass.draw_indexed(0..ctx.get_batch(label.clone()).unwrap().num_indices(), 0, 0..1);
}
