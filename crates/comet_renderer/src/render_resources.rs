use std::collections::HashMap;

pub struct RenderResources {
    bind_groups: HashMap<String, Vec<wgpu::BindGroup>>,
    bind_group_layouts: HashMap<String, Vec<wgpu::BindGroupLayout>>,
    buffers: HashMap<String, Vec<wgpu::Buffer>>,
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

    pub fn get_bindgroups(&self, label: String) -> Option<&Vec<wgpu::BindGroup>> {
        self.bind_groups.get(&label)
    }

    pub fn insert_bindgroup(&mut self, label: String, bind_group: wgpu::BindGroup) {
        match self.bind_groups.get_mut(&label) {
            None => {
                self.bind_groups.insert(label, vec![bind_group]);
            }
            Some(v) => v.push(bind_group),
        };
    }

    pub fn get_bind_group_layout(&self, label: String) -> Option<&Vec<wgpu::BindGroupLayout>> {
        self.bind_group_layouts.get(&label)
    }

    pub fn insert_bind_group_layout(&mut self, label: String, layout: wgpu::BindGroupLayout) {
        match self.bind_group_layouts.get_mut(&label) {
            None => {
                self.bind_group_layouts.insert(label, vec![layout]);
            }
            Some(v) => v.push(layout),
        }
    }

    pub fn get_buffer(&self, label: String) -> Option<&Vec<wgpu::Buffer>> {
        self.buffers.get(&label)
    }

    pub fn insert_buffer(&mut self, label: String, buffer: wgpu::Buffer) {
        match self.buffers.get_mut(&label) {
            None => {
                self.buffers.insert(label, vec![buffer]);
            }
            Some(v) => v.push(buffer),
        }
    }

    pub fn get_sampler(&self, label: String) -> Option<&wgpu::Sampler> {
        self.samplers.get(&label)
    }

    pub fn insert_sampler(&mut self, label: String, sampler: wgpu::Sampler) {
        self.samplers.insert(label, sampler);
    }
}
