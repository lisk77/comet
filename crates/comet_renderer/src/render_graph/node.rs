use std::any::Any;
use std::sync::Arc;
use crate::gpu_texture::GpuTexture;
use crate::render_pass::LoadOp;

pub struct BuildContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub format: wgpu::TextureFormat,
    pub width: u32,
    pub height: u32,
}

pub struct NodeState<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub inputs: &'a [Arc<GpuTexture>],
    pub width: u32,
    pub height: u32,
}

pub trait RenderNode: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn inputs(&self) -> Vec<&str> { vec![] }
    fn output(&self) -> Option<&str> { None }
    fn render_target(&self) -> Option<&str> { None }
    fn output_format(&self) -> Option<wgpu::TextureFormat> { None }
    fn run_after(&self) -> Vec<&str> { vec![] }
    fn load_op(&self) -> LoadOp;

    fn build(&mut self, ctx: BuildContext<'_>);
    fn run<'rpass>(&mut self, rpass: &mut wgpu::RenderPass<'rpass>, state: &NodeState<'_>);

    fn on_resize(
        &mut self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _width: u32,
        _height: u32,
    ) {
    }

    fn as_any_mut(&mut self) -> &mut dyn Any;
}
