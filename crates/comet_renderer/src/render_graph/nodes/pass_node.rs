use super::super::node::{BuildContext, NodeState, RenderNode};
use super::pass_pipeline;
use crate::{
    camera::{CameraUniform, ResolvedViewport},
    draw_batch::{DrawBatch, DrawCommand, DrawStreamError, GeometryDescriptor},
    gpu_mesh::{GpuMesh, GpuMeshDrawBatch, GpuVertexLayout, MeshVertexAttribute},
    gpu_texture::GpuTexture,
    render_pass::LoadOp,
    Vertex,
};
use comet_math::m4;
use std::{ops::Range, sync::Arc};
use wgpu::util::DeviceExt;

enum PassGeometry {
    Dynamic {
        descriptor: GeometryDescriptor,
        pipeline: Option<wgpu::RenderPipeline>,
        batch: Option<DrawBatch>,
    },
    Meshes {
        contract: Vec<MeshVertexAttribute>,
        instance_layout: wgpu::VertexBufferLayout<'static>,
        initial_instance_capacity: usize,
        pipelines: Vec<(GpuVertexLayout, wgpu::RenderPipeline)>,
        batch: Option<GpuMeshDrawBatch>,
    },
}

pub struct PassNode {
    name: String,
    shader_src: &'static str,
    topology: wgpu::PrimitiveTopology,
    texture: Option<Arc<GpuTexture>>,
    run_after: Vec<String>,
    load: LoadOp,
    viewport: Option<ResolvedViewport>,

    shader: Option<wgpu::ShaderModule>,
    output_format: Option<wgpu::TextureFormat>,
    texture_layout: Option<Arc<wgpu::BindGroupLayout>>,
    texture_bind_group: Option<Arc<wgpu::BindGroup>>,
    sampler: Option<wgpu::Sampler>,
    camera_layout: Option<Arc<wgpu::BindGroupLayout>>,
    camera_buffer: Option<Arc<wgpu::Buffer>>,
    camera_bind_group: Option<Arc<wgpu::BindGroup>>,
    geometry: PassGeometry,
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
            shader: None,
            output_format: None,
            texture_layout: None,
            texture_bind_group: None,
            sampler: None,
            camera_layout: None,
            camera_buffer: None,
            camera_bind_group: None,
            geometry: PassGeometry::Dynamic {
                descriptor: geometry_descriptor,
                pipeline: None,
                batch: None,
            },
        }
    }

    pub(crate) fn with_meshes(
        name: impl Into<String>,
        shader_src: &'static str,
        topology: wgpu::PrimitiveTopology,
        texture: Option<Arc<GpuTexture>>,
        run_after: Vec<&str>,
        load: LoadOp,
        contract: Vec<MeshVertexAttribute>,
        instance_layout: wgpu::VertexBufferLayout<'static>,
        initial_instance_capacity: usize,
    ) -> Self {
        Self {
            name: name.into(),
            shader_src,
            topology,
            texture,
            run_after: run_after.into_iter().map(|s| s.to_string()).collect(),
            load,
            viewport: None,
            shader: None,
            output_format: None,
            texture_layout: None,
            texture_bind_group: None,
            sampler: None,
            camera_layout: None,
            camera_buffer: None,
            camera_bind_group: None,
            geometry: PassGeometry::Meshes {
                contract,
                instance_layout,
                initial_instance_capacity,
                pipelines: Vec::new(),
                batch: None,
            },
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
        verts: &[Vertex],
        indices: &[u16],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), DrawStreamError> {
        let PassGeometry::Dynamic { batch, .. } = &mut self.geometry else {
            return Err(DrawStreamError::BatchNotBuilt);
        };
        let batch = batch.as_mut().ok_or(DrawStreamError::BatchNotBuilt)?;
        batch.write_vertex_stream(0, verts, device, queue)?;
        batch.write_indices_u16(indices, device, queue)?;
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
        let PassGeometry::Dynamic { batch, .. } = &mut self.geometry else {
            return Err(DrawStreamError::BatchNotBuilt);
        };
        batch
            .as_mut()
            .ok_or(DrawStreamError::BatchNotBuilt)?
            .write_vertex_stream(slot, data, device, queue)
    }

    pub fn set_draw_command(&mut self, command: DrawCommand) -> Result<(), DrawStreamError> {
        let PassGeometry::Dynamic { batch, .. } = &mut self.geometry else {
            return Err(DrawStreamError::BatchNotBuilt);
        };
        batch
            .as_mut()
            .ok_or(DrawStreamError::BatchNotBuilt)?
            .set_command(command)
    }

    pub(crate) fn write_mesh_instances<T: bytemuck::Pod>(
        &mut self,
        instances: &[T],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), DrawStreamError> {
        let PassGeometry::Meshes {
            instance_layout,
            batch,
            ..
        } = &mut self.geometry
        else {
            return Err(DrawStreamError::BatchNotBuilt);
        };
        let expected = instance_layout.array_stride;
        batch
            .as_mut()
            .ok_or(DrawStreamError::BatchNotBuilt)?
            .write_instances(instances, device, queue)
            .map_err(|actual| DrawStreamError::VertexStrideMismatch {
                slot: 1,
                expected,
                actual,
            })
    }

    pub(crate) fn accepts_mesh(
        &mut self,
        mesh: &comet_ecs::MeshData,
        max_vertex_buffer_array_stride: u32,
    ) -> bool {
        let PassGeometry::Meshes { batch, .. } = &mut self.geometry else {
            return false;
        };
        let Some(batch) = batch.as_mut() else {
            return false;
        };
        match batch.validate_mesh(mesh, max_vertex_buffer_array_stride) {
            Ok(()) => true,
            Err(error) => {
                if batch.mark_invalid(mesh.id()) {
                    comet_log::error!("Skipping incompatible mesh {:?}: {}", mesh.id(), error);
                }
                false
            }
        }
    }

    pub(crate) fn set_mesh_draws(
        &mut self,
        draws: &[(Arc<GpuMesh>, Range<u32>)],
        device: &wgpu::Device,
    ) -> Result<(), DrawStreamError> {
        let (missing_layouts, instance_layout) = {
            let PassGeometry::Meshes {
                pipelines, batch, ..
            } = &mut self.geometry
            else {
                return Err(DrawStreamError::BatchNotBuilt);
            };
            let batch = batch.as_mut().ok_or(DrawStreamError::BatchNotBuilt)?;
            for (mesh, error) in
                batch.set_draws(draws, device.limits().max_vertex_buffer_array_stride)
            {
                comet_log::error!("Skipping incompatible mesh {:?}: {}", mesh, error);
            }
            let mut missing_layouts = Vec::new();
            for draw in batch.draws() {
                if !pipelines.iter().any(|(layout, _)| layout == draw.layout())
                    && !missing_layouts.iter().any(|layout| layout == draw.layout())
                {
                    missing_layouts.push(draw.layout().clone());
                }
            }
            (missing_layouts, batch.instance_layout().clone())
        };

        let mut new_pipelines = Vec::with_capacity(missing_layouts.len());
        for layout in missing_layouts {
            let Some(pipeline) =
                self.create_pipeline(device, &[layout.as_wgpu(), instance_layout.clone()])
            else {
                return Err(DrawStreamError::BatchNotBuilt);
            };
            new_pipelines.push((layout, pipeline));
        }
        let PassGeometry::Meshes { pipelines, .. } = &mut self.geometry else {
            return Err(DrawStreamError::BatchNotBuilt);
        };
        pipelines.extend(new_pipelines);
        Ok(())
    }

    pub fn set_camera(&mut self, uniform: &CameraUniform, queue: &wgpu::Queue) {
        if let Some(buffer) = &self.camera_buffer {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[*uniform]));
        }
    }

    pub fn set_viewport(&mut self, viewport: Option<ResolvedViewport>) {
        self.viewport = viewport;
    }

    fn create_pipeline(
        &self,
        device: &wgpu::Device,
        vertex_buffers: &[wgpu::VertexBufferLayout<'_>],
    ) -> Option<wgpu::RenderPipeline> {
        Some(pass_pipeline::create(
            device,
            &self.name,
            self.shader.as_ref()?,
            self.output_format?,
            self.topology,
            self.texture_layout.as_deref(),
            self.camera_layout.as_deref()?,
            vertex_buffers,
        ))
    }
}

impl RenderNode for PassNode {
    fn name(&self) -> &str {
        &self.name
    }

    fn run_after(&self) -> &[String] {
        &self.run_after
    }

    fn load_op(&self) -> LoadOp {
        self.load.clone()
    }

    fn draw_count(&self) -> u32 {
        match &self.geometry {
            PassGeometry::Dynamic {
                batch: Some(batch), ..
            } => u32::from(!batch.command.is_empty() && batch.validate().is_ok()),
            PassGeometry::Meshes {
                batch: Some(batch), ..
            } => batch.draws().len() as u32,
            _ => 0,
        }
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

        self.shader = Some(shader);
        self.output_format = Some(ctx.format);
        self.texture_layout = texture_layout;
        self.texture_bind_group = texture_bind_group;
        self.sampler = sampler;
        self.camera_layout = Some(camera_layout);
        self.camera_buffer = Some(camera_buffer);
        self.camera_bind_group = Some(camera_bind_group);

        match &self.geometry {
            PassGeometry::Dynamic { descriptor, .. } => {
                let vertex_buffers = descriptor
                    .vertex_streams()
                    .iter()
                    .map(|stream| stream.layout().clone())
                    .collect::<Vec<_>>();
                let pipeline = self.create_pipeline(device, &vertex_buffers);
                let batch = DrawBatch::new(device, descriptor, &self.name);
                let PassGeometry::Dynamic {
                    pipeline: target_pipeline,
                    batch: target_batch,
                    ..
                } = &mut self.geometry
                else {
                    return;
                };
                *target_pipeline = pipeline;
                *target_batch = Some(batch);
            }
            PassGeometry::Meshes {
                contract,
                instance_layout,
                initial_instance_capacity,
                ..
            } => {
                let batch = GpuMeshDrawBatch::new(
                    device,
                    &self.name,
                    contract.clone(),
                    instance_layout.clone(),
                    *initial_instance_capacity,
                );
                let PassGeometry::Meshes {
                    batch: target_batch,
                    ..
                } = &mut self.geometry
                else {
                    return;
                };
                *target_batch = Some(batch);
            }
        }
    }

    fn run<'rpass>(&mut self, rpass: &mut wgpu::RenderPass<'rpass>, _state: &NodeState<'_>) {
        let Some(camera_bind_group) = self.camera_bind_group.as_ref() else {
            return;
        };

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

        if let Some(texture_bind_group) = self.texture_bind_group.as_ref() {
            rpass.set_bind_group(0, texture_bind_group, &[]);
            rpass.set_bind_group(1, camera_bind_group, &[]);
        } else {
            rpass.set_bind_group(0, camera_bind_group, &[]);
        }

        match &self.geometry {
            PassGeometry::Dynamic {
                pipeline: Some(pipeline),
                batch: Some(batch),
                ..
            } => {
                if batch.command.is_empty() {
                    return;
                }
                if let Err(error) = batch.validate() {
                    comet_log::error!("Skipping invalid draw batch '{}': {}", self.name, error);
                    return;
                }
                rpass.set_pipeline(pipeline);
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
            PassGeometry::Meshes {
                pipelines,
                batch: Some(batch),
                ..
            } => {
                let Some(instance_slice) = batch.instance_slice() else {
                    return;
                };
                rpass.set_vertex_buffer(1, instance_slice);
                for draw in batch.draws() {
                    let Some((_, pipeline)) =
                        pipelines.iter().find(|(layout, _)| layout == draw.layout())
                    else {
                        continue;
                    };
                    rpass.set_pipeline(pipeline);
                    rpass.set_vertex_buffer(0, draw.mesh.vertex_buffer.slice(..));
                    if let Some(index_buffer) = draw.mesh.index_buffer.as_ref() {
                        rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        rpass.draw_indexed(0..draw.mesh.index_count, 0, draw.instances.clone());
                    } else {
                        rpass.draw(0..draw.mesh.vertex_count, draw.instances.clone());
                    }
                }
            }
            _ => {}
        }
    }

    fn pass_mut(&mut self) -> Option<&mut PassNode> {
        Some(self)
    }
}
