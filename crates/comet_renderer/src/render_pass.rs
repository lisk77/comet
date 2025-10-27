use crate::render_context::RenderContext;

pub struct RenderPass {
    pub name: String,
    pub execute: Box<
        dyn Fn(&mut RenderContext, &mut wgpu::CommandEncoder, &wgpu::TextureView) + Send + Sync,
    >,
}

impl RenderPass {
    pub fn new(
        name: String,
        execute: Box<
            dyn Fn(&mut RenderContext, &mut wgpu::CommandEncoder, &wgpu::TextureView) + Send + Sync,
        >,
    ) -> Self {
        Self { name, execute }
    }
}
