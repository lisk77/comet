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
