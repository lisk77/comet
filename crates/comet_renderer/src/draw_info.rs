use wgpu::{BindGroupLayout, BufferUsages, Device};
use wgpu::util::DeviceExt;
use comet_resources::{Texture, Vertex};
use comet_log::*;

pub struct DrawInfo {
    name: String,
    texture: wgpu::BindGroup,
    vertex_data: Vec<Vertex>,
    index_data: Vec<u16>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl DrawInfo {
    pub fn new(
        name: String,
        device: &Device,
        texture: &Texture,
        texture_bind_group_layout: &BindGroupLayout,
        texture_sampler: &wgpu::Sampler,
        vertex_data: Vec<Vertex>,
        index_data: Vec<u16>
    ) -> Self {
        let texture_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ],
            label: Some(format!("{} Texture", name).as_str()),
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(format!("{} Vertex Buffer", &name).as_str()),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let num_indices = index_data.len() as u32;
        
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(format!("{} Index Buffer", &name).as_str()),
            contents: bytemuck::cast_slice(&index_data),
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
        });

        Self {
            name,
            texture: texture_bind,
            vertex_data,
            index_data,
            vertex_buffer,
            index_buffer,
            num_indices
        }
    }
    
    pub fn name(&self) -> &String {
        &self.name
    } 
    
    pub fn texture(&self) -> &wgpu::BindGroup {
        &self.texture
    }
    
    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }
    
    pub fn vertex_data(&self) -> &Vec<Vertex> {
        &self.vertex_data
    }
    
    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }
    
    pub fn index_data(&self) -> &Vec<u16> {
        &self.index_data
    }
    
    pub fn num_indices(&self) -> u32 {
        self.num_indices
    }
    
    pub fn update_vertex_buffer(&mut self, device: &Device, queue: &wgpu::Queue, vertex_data: Vec<Vertex>) {
        let new_vertex_size = vertex_data.len() as u64 * size_of::<Vertex>() as u64;
        match vertex_data == self.vertex_data {
            true => {},
            false => {
                match new_vertex_size > self.vertex_buffer.size() {
                    false => queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertex_data)),
                    true => {
                        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(format!("{} Vertex Buffer", self.name).as_str()),
                            contents: bytemuck::cast_slice(&vertex_data),
                            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                        });
                    }
                }
                self.vertex_data = vertex_data;
            }
        }
    }

    pub fn update_index_buffer(&mut self, device: &Device, queue: &wgpu::Queue, index_data: Vec<u16>) {
        let new_index_size = index_data.len() as u64 * size_of::<u16>() as u64;
        match index_data == self.index_data {
            true => {},
            false => {
                match new_index_size > self.index_buffer.size() {
                    false => queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&index_data)),
                    true => {
                        self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(format!("{} Index Buffer", self.name).as_str()),
                            contents: bytemuck::cast_slice(&index_data),
                            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                        });
                    }
                }
                self.num_indices = index_data.len() as u32;
                self.index_data = index_data;
            }
        }
    }
    
    pub fn set_texture(&mut self, device: &Device, layout: &BindGroupLayout, texture: &Texture) {
        self.texture = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some(format!("{} Texture Bind Group", self.name).as_str()),
        });
    }
}