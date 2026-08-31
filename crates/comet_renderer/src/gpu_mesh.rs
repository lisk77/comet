use crate::draw_batch::DynamicGpuBuffer;
use comet_ecs::{MeshData, MeshId, VertexDescriptor, VertexFormat, VertexSemantic};
use std::{collections::HashSet, fmt, ops::Range, sync::Arc};
use wgpu::util::DeviceExt;

const VERTEX_ALIGNMENT: usize = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GpuVertexLayout {
    array_stride: wgpu::BufferAddress,
    attributes: Vec<wgpu::VertexAttribute>,
}

impl GpuVertexLayout {
    fn from_contract(
        descriptor: &VertexDescriptor,
        contract: &[MeshVertexAttribute],
        max_vertex_buffer_array_stride: u32,
    ) -> Result<Self, MeshLayoutError> {
        if descriptor.stride() % VERTEX_ALIGNMENT != 0 {
            return Err(MeshLayoutError::UnalignedStride(descriptor.stride()));
        }
        if descriptor.stride() > max_vertex_buffer_array_stride as usize {
            return Err(MeshLayoutError::StrideTooLarge {
                stride: descriptor.stride(),
                maximum: max_vertex_buffer_array_stride,
            });
        }
        let mut attributes = Vec::with_capacity(contract.len());
        for required in contract {
            let Some(attribute) = descriptor
                .attributes()
                .iter()
                .find(|attribute| attribute.semantic() == required.semantic)
            else {
                return Err(MeshLayoutError::MissingAttribute(required.semantic));
            };
            if attribute.offset() % VERTEX_ALIGNMENT != 0 {
                return Err(MeshLayoutError::UnalignedOffset {
                    semantic: required.semantic,
                    offset: attribute.offset(),
                });
            }
            if attribute.format() != required.format {
                return Err(MeshLayoutError::FormatMismatch {
                    semantic: required.semantic,
                    expected: required.format,
                    actual: attribute.format(),
                });
            }
            attributes.push(wgpu::VertexAttribute {
                format: vertex_format(attribute.format()),
                offset: attribute.offset() as wgpu::BufferAddress,
                shader_location: required.shader_location,
            });
        }
        Ok(Self {
            array_stride: descriptor.stride() as wgpu::BufferAddress,
            attributes,
        })
    }

    pub(crate) fn as_wgpu(&self) -> wgpu::VertexBufferLayout<'_> {
        wgpu::VertexBufferLayout {
            array_stride: self.array_stride,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &self.attributes,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MeshVertexAttribute {
    semantic: VertexSemantic,
    format: VertexFormat,
    shader_location: u32,
}

impl MeshVertexAttribute {
    pub(crate) const fn new(
        semantic: VertexSemantic,
        format: VertexFormat,
        shader_location: u32,
    ) -> Self {
        Self {
            semantic,
            format,
            shader_location,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum MeshLayoutError {
    UnalignedStride(usize),
    StrideTooLarge {
        stride: usize,
        maximum: u32,
    },
    MissingAttribute(VertexSemantic),
    UnalignedOffset {
        semantic: VertexSemantic,
        offset: usize,
    },
    FormatMismatch {
        semantic: VertexSemantic,
        expected: VertexFormat,
        actual: VertexFormat,
    },
}

impl fmt::Display for MeshLayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnalignedStride(stride) => write!(
                formatter,
                "mesh vertex stride {stride} is not aligned to 4 bytes"
            ),
            Self::StrideTooLarge { stride, maximum } => write!(
                formatter,
                "mesh vertex stride {stride} exceeds the device limit of {maximum} bytes"
            ),
            Self::MissingAttribute(semantic) => {
                write!(
                    formatter,
                    "mesh is missing required {semantic:?} vertex data"
                )
            }
            Self::UnalignedOffset { semantic, offset } => write!(
                formatter,
                "mesh {semantic:?} vertex data offset {offset} is not aligned to {VERTEX_ALIGNMENT} bytes"
            ),
            Self::FormatMismatch {
                semantic,
                expected,
                actual,
            } => write!(
                formatter,
                "mesh {semantic:?} vertex data uses {actual:?}, expected {expected:?}"
            ),
        }
    }
}

impl std::error::Error for MeshLayoutError {}

pub(crate) struct GpuMesh {
    id: MeshId,
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: Option<wgpu::Buffer>,
    vertex_descriptor: VertexDescriptor,
    pub(crate) vertex_count: u32,
    pub(crate) index_count: u32,
}

impl GpuMesh {
    pub(crate) fn new(device: &wgpu::Device, mesh: &MeshData) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Vertex Buffer"),
            contents: mesh.vertices(),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = (!mesh.indices().is_empty()).then(|| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Mesh Index Buffer"),
                contents: bytemuck::cast_slice(mesh.indices()),
                usage: wgpu::BufferUsages::INDEX,
            })
        });
        Self {
            id: mesh.id(),
            vertex_buffer,
            index_buffer,
            vertex_descriptor: mesh.vertex_descriptor().clone(),
            vertex_count: mesh.vertex_count(),
            index_count: mesh.indices().len() as u32,
        }
    }

    pub(crate) fn matches(&self, mesh: &MeshData) -> bool {
        self.id == mesh.id()
            && self.vertex_buffer.size() >= mesh.vertices().len() as u64
            && self
                .index_buffer
                .as_ref()
                .map(wgpu::Buffer::size)
                .unwrap_or(0)
                >= std::mem::size_of_val(mesh.indices()) as u64
            && self.vertex_descriptor == *mesh.vertex_descriptor()
            && self.vertex_count == mesh.vertex_count()
            && self.index_count == mesh.indices().len() as u32
    }
}

pub(crate) struct GpuMeshDraw {
    pub(crate) mesh: Arc<GpuMesh>,
    layout: GpuVertexLayout,
    pub(crate) instances: Range<u32>,
}

impl GpuMeshDraw {
    pub(crate) fn layout(&self) -> &GpuVertexLayout {
        &self.layout
    }
}

pub(crate) struct GpuMeshDrawBatch {
    contract: Vec<MeshVertexAttribute>,
    instance_layout: wgpu::VertexBufferLayout<'static>,
    instance_buffer: DynamicGpuBuffer,
    draws: Vec<GpuMeshDraw>,
    invalid_meshes: HashSet<MeshId>,
}

impl GpuMeshDrawBatch {
    pub(crate) fn new(
        device: &wgpu::Device,
        pass_label: &str,
        contract: Vec<MeshVertexAttribute>,
        instance_layout: wgpu::VertexBufferLayout<'static>,
        initial_instance_capacity: usize,
    ) -> Self {
        let initial_capacity_bytes = instance_layout
            .array_stride
            .checked_mul(initial_instance_capacity as u64)
            .expect("mesh instance buffer capacity overflow");
        Self {
            contract,
            instance_layout,
            instance_buffer: DynamicGpuBuffer::new(
                device,
                format!("{pass_label} Instance Buffer"),
                wgpu::BufferUsages::VERTEX,
                &[],
                initial_capacity_bytes,
            ),
            draws: Vec::new(),
            invalid_meshes: HashSet::new(),
        }
    }

    pub(crate) fn write_instances<T: bytemuck::Pod>(
        &mut self,
        instances: &[T],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), u64> {
        let actual = std::mem::size_of::<T>() as u64;
        if actual != self.instance_layout.array_stride {
            return Err(actual);
        }
        self.instance_buffer.write(device, queue, instances);
        Ok(())
    }

    pub(crate) fn validate_mesh(
        &mut self,
        mesh: &MeshData,
        max_vertex_buffer_array_stride: u32,
    ) -> Result<(), MeshLayoutError> {
        GpuVertexLayout::from_contract(
            mesh.vertex_descriptor(),
            &self.contract,
            max_vertex_buffer_array_stride,
        )
        .map(|_| ())
    }

    pub(crate) fn mark_invalid(&mut self, mesh: MeshId) -> bool {
        self.invalid_meshes.insert(mesh)
    }

    pub(crate) fn set_draws(
        &mut self,
        draws: &[(Arc<GpuMesh>, Range<u32>)],
        max_vertex_buffer_array_stride: u32,
    ) -> Vec<(MeshId, MeshLayoutError)> {
        self.draws.clear();
        let mut errors = Vec::new();
        for (mesh, instances) in draws {
            if instances.is_empty() {
                continue;
            }
            match GpuVertexLayout::from_contract(
                &mesh.vertex_descriptor,
                &self.contract,
                max_vertex_buffer_array_stride,
            ) {
                Ok(layout) => self.draws.push(GpuMeshDraw {
                    mesh: Arc::clone(mesh),
                    layout,
                    instances: instances.clone(),
                }),
                Err(error) => {
                    if self.invalid_meshes.insert(mesh.id) {
                        errors.push((mesh.id, error));
                    }
                }
            }
        }
        errors
    }

    pub(crate) fn instance_layout(&self) -> &wgpu::VertexBufferLayout<'static> {
        &self.instance_layout
    }

    pub(crate) fn instance_slice(&self) -> Option<wgpu::BufferSlice<'_>> {
        self.instance_buffer.slice()
    }

    pub(crate) fn draws(&self) -> &[GpuMeshDraw] {
        &self.draws
    }
}

fn vertex_format(format: VertexFormat) -> wgpu::VertexFormat {
    match format {
        VertexFormat::Float32 => wgpu::VertexFormat::Float32,
        VertexFormat::Float32x2 => wgpu::VertexFormat::Float32x2,
        VertexFormat::Float32x3 => wgpu::VertexFormat::Float32x3,
        VertexFormat::Float32x4 => wgpu::VertexFormat::Float32x4,
        VertexFormat::Uint8x2 => wgpu::VertexFormat::Uint8x2,
        VertexFormat::Uint8x4 => wgpu::VertexFormat::Uint8x4,
        VertexFormat::Sint8x2 => wgpu::VertexFormat::Sint8x2,
        VertexFormat::Sint8x4 => wgpu::VertexFormat::Sint8x4,
        VertexFormat::Uint16x2 => wgpu::VertexFormat::Uint16x2,
        VertexFormat::Uint16x4 => wgpu::VertexFormat::Uint16x4,
        VertexFormat::Sint16x2 => wgpu::VertexFormat::Sint16x2,
        VertexFormat::Sint16x4 => wgpu::VertexFormat::Sint16x4,
        VertexFormat::Uint32 => wgpu::VertexFormat::Uint32,
        VertexFormat::Uint32x2 => wgpu::VertexFormat::Uint32x2,
        VertexFormat::Uint32x3 => wgpu::VertexFormat::Uint32x3,
        VertexFormat::Uint32x4 => wgpu::VertexFormat::Uint32x4,
        VertexFormat::Sint32 => wgpu::VertexFormat::Sint32,
        VertexFormat::Sint32x2 => wgpu::VertexFormat::Sint32x2,
        VertexFormat::Sint32x3 => wgpu::VertexFormat::Sint32x3,
        VertexFormat::Sint32x4 => wgpu::VertexFormat::Sint32x4,
    }
}
