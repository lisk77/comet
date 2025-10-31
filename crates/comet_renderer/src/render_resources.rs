use comet_log::error;
use std::{collections::HashMap, sync::Arc};

pub struct RenderResources {
    bind_groups: HashMap<String, Vec<Arc<wgpu::BindGroup>>>,
    bind_group_layouts: HashMap<String, Vec<Arc<wgpu::BindGroupLayout>>>,
    buffers: HashMap<String, Vec<Arc<wgpu::Buffer>>>,
    samplers: HashMap<String, wgpu::Sampler>,
}

impl RenderResources {
    pub fn new() -> Self {
        Self {
            bind_groups: HashMap::new(),
            bind_group_layouts: HashMap::new(),
            buffers: HashMap::new(),
            samplers: HashMap::new(),
        }
    }

    pub fn get_bind_groups(&self, label: &str) -> Option<&Vec<Arc<wgpu::BindGroup>>> {
        self.bind_groups.get(label)
    }

    pub fn get_bind_group_layout(&self, label: &str) -> Option<&Vec<Arc<wgpu::BindGroupLayout>>> {
        self.bind_group_layouts.get(label)
    }

    pub fn get_buffer(&self, label: &str) -> Option<&Vec<Arc<wgpu::Buffer>>> {
        self.buffers.get(label)
    }

    pub fn get_sampler(&self, label: &str) -> Option<&wgpu::Sampler> {
        self.samplers.get(label)
    }

    pub fn insert_bind_group(&mut self, label: String, bind_group: Arc<wgpu::BindGroup>) {
        match self.bind_groups.get_mut(&label) {
            None => {
                self.bind_groups.insert(label, vec![bind_group]);
            }
            Some(v) => v.push(bind_group),
        };
    }

    pub fn replace_bind_group(
        &mut self,
        label: String,
        pos: usize,
        bind_group: Arc<wgpu::BindGroup>,
    ) {
        match self.bind_groups.get_mut(&label) {
            None => {
                error!("Render pass {} does not exist", label);
                return;
            }
            Some(v) => {
                if v.len() <= pos {
                    error!(
                        "Position {} is out of bounds for the bind groups of render pass {}",
                        pos, label
                    );
                    return;
                }
                v[pos] = bind_group;
            }
        }
    }

    pub fn insert_bind_group_layout(&mut self, label: String, layout: Arc<wgpu::BindGroupLayout>) {
        match self.bind_group_layouts.get_mut(&label) {
            None => {
                self.bind_group_layouts.insert(label, vec![layout]);
            }
            Some(v) => v.push(layout),
        }
    }
    pub fn insert_buffer(&mut self, label: String, buffer: Arc<wgpu::Buffer>) {
        match self.buffers.get_mut(&label) {
            None => {
                self.buffers.insert(label, vec![buffer]);
            }
            Some(v) => v.push(buffer),
        }
    }

    pub fn replace_buffer(&mut self, label: String, pos: usize, buffer: Arc<wgpu::Buffer>) {
        match self.buffers.get_mut(&label) {
            None => {
                error!("Render pass {} does not exist", label);
                return;
            }
            Some(v) => {
                if v.len() <= pos {
                    error!(
                        "Position {} is out of bounds for the buffers of render pass {}",
                        pos, label
                    );
                    return;
                }
                v[pos] = buffer;
            }
        }
    }

    pub fn insert_sampler(&mut self, label: String, sampler: wgpu::Sampler) {
        self.samplers.insert(label, sampler);
    }
}
