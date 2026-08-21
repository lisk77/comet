use super::super::node::{BuildContext, NodeState, RenderNode};
use crate::{
    camera::{CameraUniform, ResolvedViewport},
    draw_batch::{DrawBatch, DrawCommand, DrawStreamError, GeometryDescriptor},
    gpu_texture::GpuTexture,
    render_pass::LoadOp,
    Vertex,
};
use comet_math::m4;
use std::any::Any;
use std::sync::Arc;
use wgpu::util::DeviceExt;

pub struct PassNode {
    name: String,
    shader_src: &'static str,
    topology: wgpu::PrimitiveTopology,
    texture: Option<Arc<GpuTexture>>,
    run_after: Vec<String>,
    load: LoadOp,
    viewport: Option<ResolvedViewport>,

    pipeline: Option<wgpu::RenderPipeline>,
    texture_layout: Option<Arc<wgpu::BindGroupLayout>>,
    texture_bind_group: Option<Arc<wgpu::BindGroup>>,
    sampler: Option<wgpu::Sampler>,
    camera_layout: Option<Arc<wgpu::BindGroupLayout>>,
    camera_buffer: Option<Arc<wgpu::Buffer>>,
    camera_bind_group: Option<Arc<wgpu::BindGroup>>,
    geometry_descriptor: GeometryDescriptor,
    batch: Option<DrawBatch>,
}

impl PassNode {
    pub fn new(
        name: impl Into<String>,
        shader_src: &'static str,
        topology: wgpu::PrimitiveTopology,
        texture: Option<Arc<GpuTexture>>,
        run_after: Vec<&str>,
        load: LoadOp,
    ) -> Self {
        Self::with_geometry(
            name,
            shader_src,
            topology,
            texture,
            run_after,
            load,
            GeometryDescriptor::mesh(Vertex::desc()),
        )
    }

    pub fn with_geometry(
        name: impl Into<String>,
        shader_src: &'static str,
        topology: wgpu::PrimitiveTopology,
        texture: Option<Arc<GpuTexture>>,
        run_after: Vec<&str>,
        load: LoadOp,
        geometry_descriptor: GeometryDescriptor,
    ) -> Self {
        Self {
            name: name.into(),
            shader_src,
            topology,
            texture,
            run_after: run_after.into_iter().map(|s| s.to_string()).collect(),
            load,
            viewport: None,
            pipeline: None,
            texture_layout: None,
            texture_bind_group: None,
            sampler: None,
            camera_layout: None,
            camera_buffer: None,
            camera_bind_group: None,
            geometry_descriptor,
            batch: None,
        }
    }

    pub fn set_texture(&mut self, texture: Arc<GpuTexture>, device: &wgpu::Device) {
        self.texture = Some(texture.clone());
        if let (Some(layout), Some(sampler)) = (self.texture_layout.as_ref(), self.sampler.as_ref())
        {
            self.texture_bind_group = Some(Arc::new(device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(sampler),
                        },
                    ],
                    label: Some(&format!("{} Texture Bind Group", self.name)),
                },
            )));
        }
    }

    pub fn set_geometry(
        &mut self,
        verts: Vec<Vertex>,
        indices: Vec<u16>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), DrawStreamError> {
        let batch = self.batch.as_mut().ok_or(DrawStreamError::BatchNotBuilt)?;
        batch.write_vertex_stream(0, &verts, device, queue)?;
        batch.write_indices_u16(&indices, device, queue)?;
        batch.set_command(DrawCommand::Indexed {
            indices: 0..indices.len() as u32,
            base_vertex: 0,
            instances: 0..1,
        })
    }

    pub fn write_vertex_stream<T: bytemuck::Pod>(
        &mut self,
        slot: usize,
        data: &[T],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), DrawStreamError> {
        self.batch
            .as_mut()
            .ok_or(DrawStreamError::BatchNotBuilt)?
            .write_vertex_stream(slot, data, device, queue)
    }

    pub fn set_draw_command(&mut self, command: DrawCommand) -> Result<(), DrawStreamError> {
        self.batch
            .as_mut()
            .ok_or(DrawStreamError::BatchNotBuilt)?
            .set_command(command)
    }

    pub fn set_camera(&mut self, uniform: &CameraUniform, queue: &wgpu::Queue) {
        if let Some(buffer) = &self.camera_buffer {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[*uniform]));
        }
    }

    pub fn set_viewport(&mut self, viewport: Option<ResolvedViewport>) {
        self.viewport = viewport;
    }
}

impl RenderNode for PassNode {
    fn name(&self) -> &str {
        &self.name
    }

    fn run_after(&self) -> Vec<&str> {
        self.run_after.iter().map(|s| s.as_str()).collect()
    }

    fn load_op(&self) -> LoadOp {
        self.load.clone()
    }

    fn build(&mut self, ctx: BuildContext<'_>) {
        let device = ctx.device;
        let has_texture = self.texture.is_some();

        let texture_layout = if has_texture {
            Some(Arc::new(device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some(&format!("{} Texture Layout", self.name)),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                },
            )))
        } else {
            None
        };

        let camera_layout = Arc::new(device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some(&format!("{} Camera Layout", self.name)),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            },
        ));

        let sampler = if has_texture {
            Some(device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }))
        } else {
            None
        };

        let texture_bind_group = if let (Some(layout), Some(sampler), Some(tex)) = (
            texture_layout.as_ref(),
            sampler.as_ref(),
            self.texture.as_ref(),
        ) {
            Some(Arc::new(device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&tex.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(sampler),
                        },
                    ],
                    label: Some(&format!("{} Texture Bind Group", self.name)),
                },
            )))
        } else {
            None
        };

        let identity: [[f32; 4]; 4] = m4::IDENTITY.into();
        let camera_buffer = Arc::new(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} Camera Buffer", self.name)),
                contents: bytemuck::cast_slice(&[identity]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        ));

        let camera_bind_group = Arc::new(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} Camera Bind Group", self.name)),
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        }));

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("{} Shader", self.name)),
            source: wgpu::ShaderSource::Wgsl(self.shader_src.into()),
        });

        let mut layout_refs: Vec<&wgpu::BindGroupLayout> = Vec::new();
        if let Some(tl) = texture_layout.as_ref() {
            layout_refs.push(tl);
        }
        layout_refs.push(&camera_layout);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{} Pipeline Layout", self.name)),
            bind_group_layouts: &layout_refs,
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("{} Pipeline", self.name)),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &self
                    .geometry_descriptor
                    .vertex_streams()
                    .iter()
                    .map(|stream| stream.layout().clone())
                    .collect::<Vec<_>>(),
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: ctx.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: self.topology,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: if self.topology == wgpu::PrimitiveTopology::TriangleList {
                    Some(wgpu::Face::Back)
                } else {
                    None
                },
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let batch = DrawBatch::new(device, &self.geometry_descriptor, &self.name);

        self.texture_layout = texture_layout;
        self.texture_bind_group = texture_bind_group;
        self.sampler = sampler;
        self.camera_layout = Some(camera_layout);
        self.camera_buffer = Some(camera_buffer);
        self.camera_bind_group = Some(camera_bind_group);
        self.pipeline = Some(pipeline);
        self.batch = Some(batch);
    }

    fn run<'rpass>(&mut self, rpass: &mut wgpu::RenderPass<'rpass>, _state: &NodeState<'_>) {
        let (Some(pipeline), Some(cam_bg), Some(batch)) = (
            self.pipeline.as_ref(),
            self.camera_bind_group.as_ref(),
            self.batch.as_ref(),
        ) else {
            return;
        };

        if batch.command.is_empty() {
            return;
        }
        if let Err(error) = batch.validate() {
            comet_log::error!("Skipping invalid draw batch '{}': {}", self.name, error);
            return;
        }

        rpass.set_pipeline(pipeline);

        if let Some(viewport) = self.viewport {
            rpass.set_viewport(
                viewport.x as f32,
                viewport.y as f32,
                viewport.width as f32,
                viewport.height as f32,
                0.0,
                1.0,
            );
            rpass.set_scissor_rect(viewport.x, viewport.y, viewport.width, viewport.height);
        }

        if let Some(tex_bg) = &self.texture_bind_group {
            rpass.set_bind_group(0, tex_bg, &[]);
            rpass.set_bind_group(1, cam_bg, &[]);
        } else {
            rpass.set_bind_group(0, cam_bg, &[]);
        }

        for (slot, stream) in batch.vertex_streams.iter().enumerate() {
            let Some(slice) = stream.buffer.slice() else {
                return;
            };
            rpass.set_vertex_buffer(slot as u32, slice);
        }

        match &batch.command {
            DrawCommand::NonIndexed {
                vertices,
                instances,
            } => rpass.draw(vertices.clone(), instances.clone()),
            DrawCommand::Indexed {
                indices,
                base_vertex,
                instances,
            } => {
                let Some(index_stream) = batch.index_stream.as_ref() else {
                    return;
                };
                let Some(slice) = index_stream.buffer.slice() else {
                    return;
                };
                rpass.set_index_buffer(slice, index_stream.format);
                rpass.draw_indexed(indices.clone(), *base_vertex, instances.clone());
            }
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
