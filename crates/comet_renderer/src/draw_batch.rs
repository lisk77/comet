use std::{fmt, mem::size_of, ops::Range};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DrawStreamError {
    BatchNotBuilt,
    InvalidVertexSlot {
        slot: usize,
        stream_count: usize,
    },
    VertexStrideMismatch {
        slot: usize,
        expected: u64,
        actual: u64,
    },
    MissingIndexStream,
    IndexFormatMismatch {
        expected: wgpu::IndexFormat,
        actual: wgpu::IndexFormat,
    },
    InvalidRange {
        kind: &'static str,
        start: u32,
        end: u32,
    },
    DrawOutOfBounds {
        stream: usize,
        kind: &'static str,
        requested: u32,
        available: u32,
    },
}

impl fmt::Display for DrawStreamError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BatchNotBuilt => write!(formatter, "draw batch has not been built"),
            Self::InvalidVertexSlot { slot, stream_count } => write!(
                formatter,
                "vertex stream slot {slot} does not exist (batch has {stream_count} streams)"
            ),
            Self::VertexStrideMismatch {
                slot,
                expected,
                actual,
            } => write!(
                formatter,
                "vertex stream {slot} expects {expected}-byte elements, got {actual}-byte elements"
            ),
            Self::MissingIndexStream => write!(formatter, "draw batch has no index stream"),
            Self::IndexFormatMismatch { expected, actual } => write!(
                formatter,
                "index stream expects {expected:?} data, got {actual:?} data"
            ),
            Self::InvalidRange { kind, start, end } => {
                write!(formatter, "invalid {kind} range {start}..{end}")
            }
            Self::DrawOutOfBounds {
                stream,
                kind,
                requested,
                available,
            } => write!(
                formatter,
                "{kind} draw requests {requested} elements from stream {stream}, but only {available} are uploaded"
            ),
        }
    }
}

impl std::error::Error for DrawStreamError {}

pub(crate) struct DynamicGpuBuffer {
    buffer: wgpu::Buffer,
    capacity_bytes: u64,
    len_bytes: u64,
    usage: wgpu::BufferUsages,
    label: String,
}

impl DynamicGpuBuffer {
    fn capacity_for(required: u64) -> u64 {
        required.max(256).next_power_of_two()
    }

    pub(crate) fn new(
        device: &wgpu::Device,
        label: impl Into<String>,
        usage: wgpu::BufferUsages,
        initial_data: &[u8],
        initial_capacity_bytes: u64,
    ) -> Self {
        let label = label.into();
        let capacity_bytes =
            Self::capacity_for(initial_capacity_bytes.max(initial_data.len() as u64));
        let mapped_at_creation = !initial_data.is_empty();
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&label),
            size: capacity_bytes,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation,
        });

        if mapped_at_creation {
            buffer.slice(..).get_mapped_range_mut()[..initial_data.len()]
                .copy_from_slice(initial_data);
            buffer.unmap();
        }

        Self {
            buffer,
            capacity_bytes,
            len_bytes: initial_data.len() as u64,
            usage,
            label,
        }
    }

    pub(crate) fn write<T: bytemuck::Pod>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &[T],
    ) {
        let bytes = bytemuck::cast_slice(data);
        let required = bytes.len() as u64;
        if required > self.capacity_bytes {
            self.capacity_bytes = Self::capacity_for(required);
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&self.label),
                size: self.capacity_bytes,
                usage: self.usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        if !bytes.is_empty() {
            queue.write_buffer(&self.buffer, 0, bytes);
        }
        self.len_bytes = required;
    }

    pub(crate) fn len_bytes(&self) -> u64 {
        self.len_bytes
    }

    pub(crate) fn slice(&self) -> Option<wgpu::BufferSlice<'_>> {
        (self.len_bytes > 0).then(|| self.buffer.slice(..self.len_bytes))
    }
}

pub struct VertexStreamDescriptor {
    label: String,
    layout: wgpu::VertexBufferLayout<'static>,
    initial_data: Vec<u8>,
    initial_capacity_bytes: u64,
}

impl VertexStreamDescriptor {
    pub fn dynamic(label: impl Into<String>, layout: wgpu::VertexBufferLayout<'static>) -> Self {
        assert!(
            layout.array_stride > 0,
            "vertex stream stride must be non-zero"
        );
        Self {
            label: label.into(),
            layout,
            initial_data: Vec::new(),
            initial_capacity_bytes: 256,
        }
    }

    pub fn with_initial_data<T: bytemuck::Pod>(mut self, data: &[T]) -> Self {
        assert_eq!(
            size_of::<T>() as u64,
            self.layout.array_stride,
            "vertex data type must match the stream layout stride"
        );
        self.initial_data = bytemuck::cast_slice(data).to_vec();
        self
    }

    pub fn with_initial_capacity_elements<T: bytemuck::Pod>(mut self, count: usize) -> Self {
        assert_eq!(
            size_of::<T>() as u64,
            self.layout.array_stride,
            "vertex capacity type must match the stream layout stride"
        );
        self.initial_capacity_bytes = (size_of::<T>() as u64)
            .checked_mul(count as u64)
            .expect("vertex stream capacity overflow");
        self
    }

    pub(crate) fn layout(&self) -> &wgpu::VertexBufferLayout<'static> {
        &self.layout
    }
}

pub struct IndexStreamDescriptor {
    label: String,
    format: wgpu::IndexFormat,
    initial_data: Vec<u8>,
    initial_capacity_bytes: u64,
}

impl IndexStreamDescriptor {
    pub fn dynamic(label: impl Into<String>, format: wgpu::IndexFormat) -> Self {
        Self {
            label: label.into(),
            format,
            initial_data: Vec::new(),
            initial_capacity_bytes: 256,
        }
    }

    pub fn with_initial_data_u16(mut self, data: &[u16]) -> Self {
        assert_eq!(
            self.format,
            wgpu::IndexFormat::Uint16,
            "u16 index data requires a Uint16 index stream"
        );
        self.initial_data = bytemuck::cast_slice(data).to_vec();
        self
    }

    pub fn with_initial_data_u32(mut self, data: &[u32]) -> Self {
        assert_eq!(
            self.format,
            wgpu::IndexFormat::Uint32,
            "u32 index data requires a Uint32 index stream"
        );
        self.initial_data = bytemuck::cast_slice(data).to_vec();
        self
    }

    pub fn with_initial_capacity_elements(mut self, count: usize) -> Self {
        self.initial_capacity_bytes = self
            .index_size()
            .checked_mul(count as u64)
            .expect("index stream capacity overflow");
        self
    }

    fn index_size(&self) -> u64 {
        match self.format {
            wgpu::IndexFormat::Uint16 => size_of::<u16>() as u64,
            wgpu::IndexFormat::Uint32 => size_of::<u32>() as u64,
        }
    }
}

pub struct GeometryDescriptor {
    vertex_streams: Vec<VertexStreamDescriptor>,
    index_stream: Option<IndexStreamDescriptor>,
}

impl GeometryDescriptor {
    pub fn new(
        vertex_streams: Vec<VertexStreamDescriptor>,
        index_stream: Option<IndexStreamDescriptor>,
    ) -> Self {
        Self {
            vertex_streams,
            index_stream,
        }
    }

    pub fn mesh(layout: wgpu::VertexBufferLayout<'static>) -> Self {
        Self {
            vertex_streams: vec![VertexStreamDescriptor::dynamic("Vertex Buffer", layout)],
            index_stream: Some(IndexStreamDescriptor::dynamic(
                "Index Buffer",
                wgpu::IndexFormat::Uint16,
            )),
        }
    }

    pub(crate) fn vertex_streams(&self) -> &[VertexStreamDescriptor] {
        &self.vertex_streams
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DrawCommand {
    NonIndexed {
        vertices: Range<u32>,
        instances: Range<u32>,
    },
    Indexed {
        indices: Range<u32>,
        base_vertex: i32,
        instances: Range<u32>,
    },
}

impl DrawCommand {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::NonIndexed {
                vertices,
                instances,
            } => vertices.is_empty() || instances.is_empty(),
            Self::Indexed {
                indices, instances, ..
            } => indices.is_empty() || instances.is_empty(),
        }
    }
}

impl Default for DrawCommand {
    fn default() -> Self {
        Self::NonIndexed {
            vertices: 0..0,
            instances: 0..0,
        }
    }
}

pub(crate) struct VertexStream {
    pub(crate) buffer: DynamicGpuBuffer,
    pub(crate) layout: wgpu::VertexBufferLayout<'static>,
}

impl VertexStream {
    fn element_count(&self) -> u32 {
        (self.buffer.len_bytes() / self.layout.array_stride) as u32
    }
}

pub(crate) struct IndexStream {
    pub(crate) buffer: DynamicGpuBuffer,
    pub(crate) format: wgpu::IndexFormat,
}

impl IndexStream {
    fn element_count(&self) -> u32 {
        let index_size = match self.format {
            wgpu::IndexFormat::Uint16 => size_of::<u16>() as u64,
            wgpu::IndexFormat::Uint32 => size_of::<u32>() as u64,
        };
        (self.buffer.len_bytes() / index_size) as u32
    }
}

pub struct DrawBatch {
    pub(crate) vertex_streams: Vec<VertexStream>,
    pub(crate) index_stream: Option<IndexStream>,
    pub(crate) command: DrawCommand,
}

impl DrawBatch {
    pub fn new(device: &wgpu::Device, descriptor: &GeometryDescriptor, pass_label: &str) -> Self {
        let vertex_streams = descriptor
            .vertex_streams
            .iter()
            .map(|stream| VertexStream {
                buffer: DynamicGpuBuffer::new(
                    device,
                    format!("{} {}", pass_label, stream.label),
                    wgpu::BufferUsages::VERTEX,
                    &stream.initial_data,
                    stream.initial_capacity_bytes,
                ),
                layout: stream.layout.clone(),
            })
            .collect();

        let index_stream = descriptor.index_stream.as_ref().map(|stream| IndexStream {
            buffer: DynamicGpuBuffer::new(
                device,
                format!("{} {}", pass_label, stream.label),
                wgpu::BufferUsages::INDEX,
                &stream.initial_data,
                stream.initial_capacity_bytes,
            ),
            format: stream.format,
        });

        Self {
            vertex_streams,
            index_stream,
            command: DrawCommand::default(),
        }
    }

    pub fn write_vertex_stream<T: bytemuck::Pod>(
        &mut self,
        slot: usize,
        data: &[T],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), DrawStreamError> {
        let stream_count = self.vertex_streams.len();
        let Some(stream) = self.vertex_streams.get_mut(slot) else {
            return Err(DrawStreamError::InvalidVertexSlot { slot, stream_count });
        };
        let actual = size_of::<T>() as u64;
        if actual != stream.layout.array_stride {
            return Err(DrawStreamError::VertexStrideMismatch {
                slot,
                expected: stream.layout.array_stride,
                actual,
            });
        }
        stream.buffer.write(device, queue, data);
        Ok(())
    }

    pub fn write_indices_u16(
        &mut self,
        data: &[u16],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), DrawStreamError> {
        self.write_indices(data, wgpu::IndexFormat::Uint16, device, queue)
    }

    pub fn write_indices_u32(
        &mut self,
        data: &[u32],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), DrawStreamError> {
        self.write_indices(data, wgpu::IndexFormat::Uint32, device, queue)
    }

    fn write_indices<T: bytemuck::Pod>(
        &mut self,
        data: &[T],
        actual: wgpu::IndexFormat,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), DrawStreamError> {
        let Some(stream) = self.index_stream.as_mut() else {
            return Err(DrawStreamError::MissingIndexStream);
        };
        if stream.format != actual {
            return Err(DrawStreamError::IndexFormatMismatch {
                expected: stream.format,
                actual,
            });
        }
        stream.buffer.write(device, queue, data);
        Ok(())
    }

    pub fn set_command(&mut self, command: DrawCommand) -> Result<(), DrawStreamError> {
        self.validate_command(&command)?;
        self.command = command;
        Ok(())
    }

    pub(crate) fn validate(&self) -> Result<(), DrawStreamError> {
        self.validate_command(&self.command)
    }

    fn validate_command(&self, command: &DrawCommand) -> Result<(), DrawStreamError> {
        match command {
            DrawCommand::NonIndexed {
                vertices,
                instances,
            } => {
                validate_range("vertex", vertices)?;
                validate_range("instance", instances)?;
            }
            DrawCommand::Indexed {
                indices, instances, ..
            } => {
                validate_range("index", indices)?;
                validate_range("instance", instances)?;
            }
        }
        if command.is_empty() {
            return Ok(());
        }

        match command {
            DrawCommand::NonIndexed {
                vertices,
                instances,
            } => self.validate_vertex_streams(vertices.end, instances.end),
            DrawCommand::Indexed {
                indices, instances, ..
            } => {
                let index_stream = self
                    .index_stream
                    .as_ref()
                    .ok_or(DrawStreamError::MissingIndexStream)?;
                let available = index_stream.element_count();
                if indices.end > available {
                    return Err(DrawStreamError::DrawOutOfBounds {
                        stream: 0,
                        kind: "index",
                        requested: indices.end,
                        available,
                    });
                }
                self.validate_vertex_streams(1, instances.end)
            }
        }
    }

    fn validate_vertex_streams(
        &self,
        required_vertices: u32,
        required_instances: u32,
    ) -> Result<(), DrawStreamError> {
        for (slot, stream) in self.vertex_streams.iter().enumerate() {
            let (kind, requested) = match stream.layout.step_mode {
                wgpu::VertexStepMode::Vertex => ("vertex", required_vertices),
                wgpu::VertexStepMode::Instance => ("instance", required_instances),
            };
            let available = stream.element_count();
            if requested > available {
                return Err(DrawStreamError::DrawOutOfBounds {
                    stream: slot,
                    kind,
                    requested,
                    available,
                });
            }
        }
        Ok(())
    }
}

fn validate_range(kind: &'static str, range: &Range<u32>) -> Result<(), DrawStreamError> {
    if range.start > range.end {
        return Err(DrawStreamError::InvalidRange {
            kind,
            start: range.start,
            end: range.end,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_range, DrawCommand, DrawStreamError};

    #[test]
    fn empty_draw_commands_are_detected() {
        assert!(DrawCommand::NonIndexed {
            vertices: 0..3,
            instances: 0..0,
        }
        .is_empty());
        assert!(DrawCommand::Indexed {
            indices: 0..0,
            base_vertex: 0,
            instances: 0..1,
        }
        .is_empty());
        assert!(!DrawCommand::Indexed {
            indices: 0..6,
            base_vertex: 0,
            instances: 0..1,
        }
        .is_empty());
    }

    #[test]
    fn reversed_ranges_are_rejected() {
        assert_eq!(
            validate_range("vertex", &(3..2)),
            Err(DrawStreamError::InvalidRange {
                kind: "vertex",
                start: 3,
                end: 2,
            })
        );
    }
}
