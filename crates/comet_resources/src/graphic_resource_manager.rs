use std::{collections::HashMap, path::Path};

use crate::{
    font::Font,
    texture_atlas::{TextureAtlas, TextureRegion},
    Texture,
};
use comet_log::info;
use wgpu::{naga::ShaderStage, Device, Queue, ShaderModule};

pub struct GraphicResourceManager {
    texture_atlas: TextureAtlas,
    font_atlas: TextureAtlas,
    fonts: Vec<Font>,
    data_files: HashMap<String, String>,
    shaders: HashMap<String, ShaderModule>,
}

impl GraphicResourceManager {
    pub fn new() -> Self {
        Self {
            texture_atlas: TextureAtlas::empty(),
            font_atlas: TextureAtlas::empty(),
            fonts: Vec::new(),
            data_files: HashMap::new(),
            shaders: HashMap::new(),
        }
    }

    pub fn texture_atlas(&self) -> &TextureAtlas {
        &self.texture_atlas
    }

    pub fn font_atlas(&self) -> &TextureAtlas {
        &self.font_atlas
    }

    pub fn set_font_atlas(&mut self, font_atlas: TextureAtlas) {
        self.font_atlas = font_atlas
    }

    pub fn texture_locations(&self) -> &HashMap<String, TextureRegion> {
        &self.texture_atlas.textures()
    }

    pub fn data_files(&self) -> &HashMap<String, String> {
        &self.data_files
    }

    pub fn fonts(&self) -> &Vec<Font> {
        &self.fonts
    }

    pub fn fonts_mut(&mut self) -> &mut Vec<Font> {
        &mut self.fonts
    }

    pub fn get_glyph(&self, font: &str, ch: char) -> Option<&TextureRegion> {
        self.fonts
            .iter()
            .find(|f| f.name() == font)
            .and_then(|f| f.get_glyph(ch))
    }

    pub fn set_texture_atlas(&mut self, texture_atlas: TextureAtlas) {
        self.texture_atlas = texture_atlas;
    }

    pub fn create_texture_atlas(&mut self, paths: Vec<String>) {
        self.texture_atlas = TextureAtlas::from_texture_paths(paths)
    }

    pub fn load_string(&self, file_name: &str) -> anyhow::Result<String> {
        let base_path = std::env::var("OUT_DIR")
            .map(|p| Path::new(&p).to_path_buf())
            .unwrap_or_else(|_| Path::new(".").to_path_buf());

        let path = base_path.join(file_name);
        let txt = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Failed to load {}: {}", path.display(), e))?;

        Ok(txt)
    }

    pub fn load_binary(&self, file_name: &str) -> anyhow::Result<Vec<u8>> {
        let path = Path::new(std::env::var("OUT_DIR")?.as_str())
            .join("res")
            .join(file_name);
        let data = std::fs::read(path)?;

        Ok(data)
    }

    pub fn load_texture(
        &self,
        file_name: &str,
        is_normal_map: bool,
        device: &Device,
        queue: &Queue,
    ) -> anyhow::Result<Texture> {
        let data = self.load_binary(file_name)?;
        Texture::from_bytes(device, queue, &data, file_name, is_normal_map)
    }

    /// `file_name` is the full name, so with the extension
    /// `shader_stage` is only needed if it is a GLSL shader, so default to None if it isn't GLSL
    pub fn load_shader(
        &mut self,
        device: &Device,
        shader_stage: Option<ShaderStage>,
        file_name: &str,
    ) -> anyhow::Result<()> {
        let shader_source = self.load_string(file_name)?;

        let module = match file_name.split('.').last() {
            Some("wgsl") => device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(file_name),
                source: wgpu::ShaderSource::Wgsl(shader_source.into()),
            }),
            Some("glsl") => {
                if let Some(stage) = shader_stage {
                    device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some(file_name),
                        source: wgpu::ShaderSource::Glsl {
                            shader: shader_source.into(),
                            stage,
                            defines: Default::default(),
                        },
                    })
                } else {
                    return Err(anyhow::anyhow!("GLSL shader needs a stage"));
                }
            }
            _ => return Err(anyhow::anyhow!("Unsupported shader type")),
        };

        self.shaders.insert(file_name.to_string(), module);
        Ok(())
    }

    /// Loads the shader from a source code string
    /// Right now only works with wgsl
    pub fn load_shader_from_string(
        &mut self,
        device: &Device,
        shader_name: &str,
        shader_src: &str,
    ) -> anyhow::Result<()> {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(shader_name),
            source: wgpu::ShaderSource::Wgsl(shader_src.into()),
        });

        self.shaders.insert(shader_name.to_string(), module);
        Ok(())
    }

    pub fn get_shader(&self, shader: &str) -> Option<&ShaderModule> {
        self.shaders.get(shader)
    }

    pub fn load_font(&mut self, path: &str, size: f32) {
        info!("Loading font: {}", path);
        let font = Font::new(path, size);
        info!("Font {} loaded!", font.name());
        self.fonts.push(font);
    }
}
