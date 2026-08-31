use crate::{gpu_mesh::GpuMesh, gpu_texture::GpuTexture};
use comet_assets;
use comet_ecs::{Mesh, MeshId};
use comet_log::{cassert, error};
use std::{collections::HashMap, sync::Arc};

struct CachedGpuMesh {
    mesh: Arc<GpuMesh>,
    last_used_frame: u64,
}

pub struct RenderResources {
    bind_groups: HashMap<String, Vec<Arc<wgpu::BindGroup>>>,
    bind_group_layouts: HashMap<String, Vec<Arc<wgpu::BindGroupLayout>>>,
    buffers: HashMap<String, Vec<Arc<wgpu::Buffer>>>,
    samplers: HashMap<String, wgpu::Sampler>,
    gpu_textures: HashMap<String, Arc<GpuTexture>>,
    gpu_meshes: HashMap<MeshId, CachedGpuMesh>,
    mesh_frame: u64,
    asset_atlas_handles: HashMap<String, comet_assets::Asset<comet_assets::TextureAtlas>>,
}

impl RenderResources {
    pub fn new() -> Self {
        Self {
            bind_groups: HashMap::new(),
            bind_group_layouts: HashMap::new(),
            buffers: HashMap::new(),
            samplers: HashMap::new(),
            gpu_textures: HashMap::new(),
            gpu_meshes: HashMap::new(),
            mesh_frame: 0,
            asset_atlas_handles: HashMap::new(),
        }
    }

    /// Get all bind groups associated with a render pass.
    pub fn get_bind_groups(&self, render_pass_label: &str) -> Option<&Vec<Arc<wgpu::BindGroup>>> {
        self.bind_groups.get(render_pass_label)
    }

    /// Get all bind group layouts associated with a render pass.
    pub fn get_bind_group_layout(
        &self,
        render_pass_label: &str,
    ) -> Option<&Vec<Arc<wgpu::BindGroupLayout>>> {
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
    pub fn insert_bind_group(
        &mut self,
        render_pass_label: String,
        bind_group: Arc<wgpu::BindGroup>,
    ) {
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
    pub fn insert_bind_group_layout(
        &mut self,
        render_pass_label: String,
        layout: Arc<wgpu::BindGroupLayout>,
    ) {
        match self.bind_group_layouts.get_mut(&render_pass_label) {
            None => {
                self.bind_group_layouts
                    .insert(render_pass_label, vec![layout]);
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
    pub fn replace_buffer(
        &mut self,
        render_pass_label: String,
        pos: usize,
        buffer: Arc<wgpu::Buffer>,
    ) {
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

    pub(crate) fn begin_mesh_frame(&mut self) {
        self.mesh_frame = self.mesh_frame.saturating_add(1);
    }

    pub(crate) fn prepare_mesh(&mut self, device: &wgpu::Device, mesh: &Mesh) -> Arc<GpuMesh> {
        let id = mesh.data().id();
        if let Some(cached) = self.gpu_meshes.get_mut(&id) {
            cassert!(
                cached.mesh.matches(mesh.data()),
                "cached GPU mesh does not match mesh metadata"
            );
            cached.last_used_frame = self.mesh_frame;
            return Arc::clone(&cached.mesh);
        }
        let gpu_mesh = Arc::new(GpuMesh::new(device, mesh.data()));
        self.gpu_meshes.insert(
            id,
            CachedGpuMesh {
                mesh: Arc::clone(&gpu_mesh),
                last_used_frame: self.mesh_frame,
            },
        );
        gpu_mesh
    }

    pub(crate) fn evict_stale_meshes(&mut self, retention_frames: u64) {
        let oldest_retained_frame = self.mesh_frame.saturating_sub(retention_frames);
        self.gpu_meshes
            .retain(|_, cached| cached.last_used_frame >= oldest_retained_frame);
    }

    /// Get a cached asset atlas handle for metadata lookups.
    pub fn get_asset_atlas_handle(
        &self,
        key: &str,
    ) -> Option<comet_assets::Asset<comet_assets::TextureAtlas>> {
        self.asset_atlas_handles.get(key).copied()
    }

    /// Cache an asset atlas handle for lookups.
    pub fn insert_asset_atlas_handle(
        &mut self,
        key: String,
        handle: comet_assets::Asset<comet_assets::TextureAtlas>,
    ) {
        self.asset_atlas_handles.insert(key, handle);
    }

    /// Remove a cached asset atlas handle.
    pub fn remove_asset_atlas_handle(
        &mut self,
        key: &str,
    ) -> Option<comet_assets::Asset<comet_assets::TextureAtlas>> {
        self.asset_atlas_handles.remove(key)
    }
}
