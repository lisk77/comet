use std::{borrow::Cow, fmt};

pub(crate) struct CompiledShader {
    module: wgpu::naga::Module,
    vertex_entry: String,
    fragment_entry: String,
}

impl CompiledShader {
    pub(crate) fn compile(path: &str, source: &str) -> Result<Self, ShaderCompileError> {
        let module = wgpu::naga::front::wgsl::parse_str(source)
            .map_err(|error| ShaderCompileError::new(path, error.emit_to_string(source)))?;
        wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            wgpu::naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .map_err(|error| ShaderCompileError::new(path, error.emit_to_string(source)))?;

        let vertex_entries = module
            .entry_points
            .iter()
            .filter(|entry| entry.stage == wgpu::naga::ShaderStage::Vertex)
            .collect::<Vec<_>>();
        let fragment_entries = module
            .entry_points
            .iter()
            .filter(|entry| entry.stage == wgpu::naga::ShaderStage::Fragment)
            .collect::<Vec<_>>();
        if vertex_entries.len() != 1 {
            return Err(ShaderCompileError::new(
                path,
                format!(
                    "shader must define exactly one vertex entry point, found {}",
                    vertex_entries.len()
                ),
            ));
        }
        if fragment_entries.len() != 1 {
            return Err(ShaderCompileError::new(
                path,
                format!(
                    "shader must define exactly one fragment entry point, found {}",
                    fragment_entries.len()
                ),
            ));
        }

        Ok(Self {
            vertex_entry: vertex_entries[0].name.clone(),
            fragment_entry: fragment_entries[0].name.clone(),
            module,
        })
    }

    pub(crate) fn create_module(&self, device: &wgpu::Device, label: &str) -> wgpu::ShaderModule {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Naga(Cow::Owned(self.module.clone())),
        })
    }

    pub(crate) fn vertex_entry(&self) -> &str {
        &self.vertex_entry
    }

    pub(crate) fn fragment_entry(&self) -> &str {
        &self.fragment_entry
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShaderCompileError {
    path: String,
    message: String,
}

impl ShaderCompileError {
    fn new(path: &str, message: impl Into<String>) -> Self {
        Self {
            path: path.to_string(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ShaderCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "shader '{}': {}", self.path, self.message)
    }
}

impl std::error::Error for ShaderCompileError {}
