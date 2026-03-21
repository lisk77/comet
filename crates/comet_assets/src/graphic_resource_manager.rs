use std::collections::HashMap;

use crate::{
    asset_handle::Asset,
    asset_manager::AssetManager,
    asset_path::{asset_root, resolve_asset_path},
    atlas_ref::AtlasRef,
    font::Font,
    texture_atlas::{TextureAtlas, TextureRegion},
    Image,
};
use comet_log::info;
use wgpu::{naga::ShaderStage, Device, ShaderModule};

pub struct GraphicResourceManager {
    asset_manager: AssetManager,
    texture_atlas_handle: Option<Asset<TextureAtlas>>,
    font_atlas: TextureAtlas,
    fonts: Vec<Font>,
    data_files: HashMap<String, String>,
    shaders: HashMap<String, ShaderModule>,
}

impl GraphicResourceManager {
    pub fn new() -> Self {
        Self {
            asset_manager: AssetManager::new(),
            texture_atlas_handle: None,
            font_atlas: TextureAtlas::empty(),
            fonts: Vec::new(),
            data_files: HashMap::new(),
            shaders: HashMap::new(),
        }
    }

    pub fn texture_atlas(&self) -> &TextureAtlas {
        let handle = self
            .texture_atlas_handle
            .expect("texture atlas requested before initialization");
        self.asset_manager
            .get_texture_atlas(handle)
            .expect("texture atlas handle is stale")
    }

    pub fn texture_atlas_handle(&self) -> Option<Asset<TextureAtlas>> {
        self.texture_atlas_handle
    }

    pub fn font_atlas(&self) -> &TextureAtlas {
        &self.font_atlas
    }

    pub fn set_font_atlas(&mut self, font_atlas: TextureAtlas) {
        self.font_atlas = font_atlas
    }

    pub fn texture_locations(&self) -> &HashMap<String, TextureRegion> {
        self.texture_atlas().textures()
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
        if let Some(handle) = self.texture_atlas_handle.take() {
            let _ = self.asset_manager.remove_texture_atlas(handle);
        }
        self.texture_atlas_handle = Some(self.asset_manager.add_texture_atlas(texture_atlas));
    }

    pub fn create_texture_atlas(&mut self, paths: Vec<String>) {
        self.set_texture_atlas(TextureAtlas::from_texture_paths(paths))
    }

    pub fn resolve_atlas_ref(&self, path: &'static str) -> Option<AtlasRef> {
        let atlas = self.texture_atlas_handle?;
        let region = self
            .asset_manager
            .get_texture_atlas(atlas)?
            .textures()
            .get(path)
            .copied()?;
        Some(AtlasRef::new(region, atlas))
    }

    pub fn load_string(&self, file_name: &str) -> anyhow::Result<String> {
        let path = resolve_asset_path(file_name);
        let txt = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Failed to load {}: {}", path.display(), e))?;

        Ok(txt)
    }

    pub fn load_binary(&self, file_name: &str) -> anyhow::Result<Vec<u8>> {
        let path = resolve_asset_path(file_name);
        let data = std::fs::read(path)?;

        Ok(data)
    }

    pub fn load_image(&self, file_name: &str, is_normal_map: bool) -> anyhow::Result<Image> {
        let data = self.load_binary(file_name)?;
        Image::from_bytes(&data, is_normal_map)
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

    pub fn texture_directory(&self) -> std::path::PathBuf {
        asset_root().join("textures")
    }
}
