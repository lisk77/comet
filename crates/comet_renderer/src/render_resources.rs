use comet_log::error;
use std::{collections::HashMap, sync::Arc};
use crate::gpu_texture::GpuTexture;
use comet_assets;

pub struct RenderResources {
    bind_groups: HashMap<String, Vec<Arc<wgpu::BindGroup>>>,
    bind_group_layouts: HashMap<String, Vec<Arc<wgpu::BindGroupLayout>>>,
    buffers: HashMap<String, Vec<Arc<wgpu::Buffer>>>,
    samplers: HashMap<String, wgpu::Sampler>,
    gpu_textures: HashMap<String, Arc<GpuTexture>>,
    asset_atlas_handles: HashMap<String, comet_assets::Asset<comet_assets::TextureAtlas>>,
    asset_font_handles: HashMap<String, comet_assets::Asset<comet_assets::Font>>,
}

impl RenderResources {
    pub fn new() -> Self {
        Self {
            bind_groups: HashMap::new(),
            bind_group_layouts: HashMap::new(),
            buffers: HashMap::new(),
            samplers: HashMap::new(),
            gpu_textures: HashMap::new(),
            asset_atlas_handles: HashMap::new(),
            asset_font_handles: HashMap::new(),
        }
    }

    /// Get all bind groups associated with a render pass.
    pub fn get_bind_groups(&self, render_pass_label: &str) -> Option<&Vec<Arc<wgpu::BindGroup>>> {
        self.bind_groups.get(render_pass_label)
    }

    /// Get all bind group layouts associated with a render pass.
    pub fn get_bind_group_layout(&self, render_pass_label: &str) -> Option<&Vec<Arc<wgpu::BindGroupLayout>>> {
        self.bind_group_layouts.get(render_pass_label)
    }

    /// Replace a bind group layout at a specific position for a render pass.
    pub fn replace_bind_group_layout(
        &mut self,
        render_pass_label: String,
        pos: usize,
        bind_group_layout: Arc<wgpu::BindGroupLayout>,
    ) {
        match self.bind_group_layouts.get_mut(&render_pass_label) {
            None => {
                error!("Render pass {} does not exist", render_pass_label);
                return;
            }
            Some(v) => {
                if v.len() <= pos {
                    error!(
                        "Position {} is out of bounds for the bind group layouts of render pass {}",
                        pos, render_pass_label
                    );
                    return;
                }
                v[pos] = bind_group_layout;
            }
        }
    }

    /// Get all buffers associated with a render pass.
    pub fn get_buffer(&self, render_pass_label: &str) -> Option<&Vec<Arc<wgpu::Buffer>>> {
        self.buffers.get(render_pass_label)
    }

    /// Get a sampler associated with a render pass.
    pub fn get_sampler(&self, render_pass_label: &str) -> Option<&wgpu::Sampler> {
        self.samplers.get(render_pass_label)
    }

    /// Insert a bind group for a render pass.
    pub fn insert_bind_group(&mut self, render_pass_label: String, bind_group: Arc<wgpu::BindGroup>) {
        match self.bind_groups.get_mut(&render_pass_label) {
            None => {
                self.bind_groups.insert(render_pass_label, vec![bind_group]);
            }
            Some(v) => v.push(bind_group),
        };
    }

    /// Replace a bind group at a specific position for a render pass.    
    pub fn replace_bind_group(
        &mut self,
        render_pass_label: String,
        pos: usize,
        bind_group: Arc<wgpu::BindGroup>,
    ) {
        match self.bind_groups.get_mut(&render_pass_label) {
            None => {
                error!("Render pass {} does not exist", render_pass_label);
                return;
            }
            Some(v) => {
                if v.len() <= pos {
                    error!(
                        "Position {} is out of bounds for the bind groups of render pass {}",
                        pos, render_pass_label
                    );
                    return;
                }
                v[pos] = bind_group;
            }
        }
    }

    /// Insert a bind group layout for a render pass.         
    pub fn insert_bind_group_layout(&mut self, render_pass_label: String, layout: Arc<wgpu::BindGroupLayout>) {
        match self.bind_group_layouts.get_mut(&render_pass_label) {
            None => {
                self.bind_group_layouts.insert(render_pass_label, vec![layout]);
            }
            Some(v) => v.push(layout),
        }
    }

    /// Add a buffer for a render pass.
    pub fn insert_buffer(&mut self, render_pass_label: String, buffer: Arc<wgpu::Buffer>) {
        match self.buffers.get_mut(&render_pass_label) {
            None => {
                self.buffers.insert(render_pass_label, vec![buffer]);
            }
            Some(v) => v.push(buffer),
        }
    }

    /// Replace a buffer at a specific position for a render pass.
    pub fn replace_buffer(&mut self, render_pass_label: String, pos: usize, buffer: Arc<wgpu::Buffer>) {
        match self.buffers.get_mut(&render_pass_label) {
            None => {
                error!("Render pass {} does not exist", render_pass_label);
                return;
            }
            Some(v) => {
                if v.len() <= pos {
                    error!(
                        "Position {} is out of bounds for the buffers of render pass {}",
                        pos, render_pass_label
                    );
                    return;
                }
                v[pos] = buffer;
            }
        }
    }

    /// Insert a sampler for a render pass.
    pub fn insert_sampler(&mut self, render_pass_label: String, sampler: wgpu::Sampler) {
        self.samplers.insert(render_pass_label, sampler);
    }

    /// Get a cached GPU texture for a render pass.
    pub fn get_gpu_texture(&self, render_pass_label: &str) -> Option<&Arc<GpuTexture>> {
        self.gpu_textures.get(render_pass_label)
    }

    /// Add a GPU texture to a render pass.
    pub fn insert_gpu_texture(&mut self, render_pass_label: String, texture: Arc<GpuTexture>) {
        self.gpu_textures.insert(render_pass_label, texture);
    }

    /// Replace a GPU texture of a render pass.
    pub fn replace_gpu_texture(&mut self, render_pass_label: String, texture: Arc<GpuTexture>) {
        self.gpu_textures.insert(render_pass_label, texture);
    }

    /// Remove a GPU texture from a render pass.
    pub fn remove_gpu_texture(&mut self, render_pass_label: &str) -> Option<Arc<GpuTexture>> {
        self.gpu_textures.remove(render_pass_label)
    }

    /// Get a cached asset atlas handle for metadata lookups.
    pub fn get_asset_atlas_handle(&self, key: &str) -> Option<comet_assets::Asset<comet_assets::TextureAtlas>> {
        self.asset_atlas_handles.get(key).copied()
    }

    /// Cache an asset atlas handle for lookups.
    pub fn insert_asset_atlas_handle(&mut self, key: String, handle: comet_assets::Asset<comet_assets::TextureAtlas>) {
        self.asset_atlas_handles.insert(key, handle);
    }

    /// Remove a cached asset atlas handle.
    pub fn remove_asset_atlas_handle(&mut self, key: &str) -> Option<comet_assets::Asset<comet_assets::TextureAtlas>> {
        self.asset_atlas_handles.remove(key)
    }

    /// Get a cached asset font handle by name.
    pub fn get_asset_font_handle(&self, key: &str) -> Option<comet_assets::Asset<comet_assets::Font>> {
        self.asset_font_handles.get(key).copied()
    }

    /// Cache an asset font handle by name.
    pub fn insert_asset_font_handle(&mut self, key: String, handle: comet_assets::Asset<comet_assets::Font>) {
        self.asset_font_handles.insert(key, handle);
    }

    /// Remove a cached asset font handle.
    pub fn remove_asset_font_handle(&mut self, key: &str) -> Option<comet_assets::Asset<comet_assets::Font>> {
        self.asset_font_handles.remove(key)
    }
}
