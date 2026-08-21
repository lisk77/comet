use wgpu::util::DeviceExt;

pub struct DynamicGpuBuffer {
    buffer: wgpu::Buffer,
    capacity: u64,
    usage: wgpu::BufferUsages,
    label: String,
}

impl DynamicGpuBuffer {
    fn capacity_for(required: u64) -> u64 {
        required.max(256).next_power_of_two()
    }

    pub fn new(
        device: &wgpu::Device,
        label: impl Into<String>,
        usage: wgpu::BufferUsages,
        initial_data: &[u8],
        initial_capacity: u64,
    ) -> Self {
        let label = label.into();
        let requested_capacity =
            Self::capacity_for(initial_capacity.max(initial_data.len() as u64));
        let (buffer, capacity) = if initial_data.is_empty() {
            (
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&label),
                    size: requested_capacity,
                    usage: usage | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                requested_capacity,
            )
        } else {
            (
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&label),
                    contents: initial_data,
                    usage: usage | wgpu::BufferUsages::COPY_DST,
                }),
                initial_data.len() as u64,
            )
        };

        Self {
            buffer,
            capacity,
            usage,
            label,
        }
    }

    pub fn write<T: bytemuck::Pod>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &[T],
    ) {
        let bytes = bytemuck::cast_slice(data);
        let required = bytes.len() as u64;
        if required > self.capacity {
            self.capacity = Self::capacity_for(required);
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&self.label),
                size: self.capacity,
                usage: self.usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if !bytes.is_empty() {
            queue.write_buffer(&self.buffer, 0, bytes);
        }
    }

    pub fn slice(&self) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(..)
    }
}

pub struct VertexStreamDescriptor {
    pub label: String,
    pub layout: wgpu::VertexBufferLayout<'static>,
    pub initial_data: Vec<u8>,
    pub initial_capacity: u64,
}

impl VertexStreamDescriptor {
    pub fn dynamic(label: impl Into<String>, layout: wgpu::VertexBufferLayout<'static>) -> Self {
        Self {
            label: label.into(),
            layout,
            initial_data: Vec::new(),
            initial_capacity: 256,
        }
    }

    pub fn with_initial_data<T: bytemuck::Pod>(mut self, data: &[T]) -> Self {
        self.initial_data = bytemuck::cast_slice(data).to_vec();
        self
    }

    pub fn with_initial_capacity(mut self, capacity: u64) -> Self {
        self.initial_capacity = capacity;
        self
    }
}

pub struct IndexStreamDescriptor {
    pub label: String,
    pub format: wgpu::IndexFormat,
    pub initial_data: Vec<u8>,
    pub initial_capacity: u64,
}

impl IndexStreamDescriptor {
    pub fn dynamic(label: impl Into<String>, format: wgpu::IndexFormat) -> Self {
        Self {
            label: label.into(),
            format,
            initial_data: Vec::new(),
            initial_capacity: 256,
        }
    }

    pub fn with_initial_data<T: bytemuck::Pod>(mut self, data: &[T]) -> Self {
        self.initial_data = bytemuck::cast_slice(data).to_vec();
        self
    }
}

pub struct GeometryDescriptor {
    pub vertex_streams: Vec<VertexStreamDescriptor>,
    pub index_stream: Option<IndexStreamDescriptor>,
}

impl GeometryDescriptor {
    pub fn mesh(layout: wgpu::VertexBufferLayout<'static>) -> Self {
        Self {
            vertex_streams: vec![VertexStreamDescriptor::dynamic("Vertex Buffer", layout)],
            index_stream: Some(IndexStreamDescriptor::dynamic(
                "Index Buffer",
                wgpu::IndexFormat::Uint16,
            )),
        }
    }
}

#[derive(Clone, Debug)]
pub enum DrawCommand {
    NonIndexed {
        vertex_count: u32,
        instance_count: u32,
    },
    Indexed {
        index_count: u32,
        base_vertex: i32,
        instance_count: u32,
    },
}

impl Default for DrawCommand {
    fn default() -> Self {
        Self::NonIndexed {
            vertex_count: 0,
            instance_count: 0,
        }
    }
}

pub(crate) struct VertexStream {
    pub(crate) buffer: DynamicGpuBuffer,
}

pub(crate) struct IndexStream {
    pub(crate) buffer: DynamicGpuBuffer,
    pub(crate) format: wgpu::IndexFormat,
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
                    stream.initial_capacity,
                ),
            })
            .collect();

        let index_stream = descriptor.index_stream.as_ref().map(|stream| IndexStream {
            buffer: DynamicGpuBuffer::new(
                device,
                format!("{} {}", pass_label, stream.label),
                wgpu::BufferUsages::INDEX,
                &stream.initial_data,
                stream.initial_capacity,
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
    ) -> Result<(), usize> {
        let Some(stream) = self.vertex_streams.get_mut(slot) else {
            return Err(slot);
        };
        stream.buffer.write(device, queue, data);
        Ok(())
    }

    pub fn write_indices<T: bytemuck::Pod>(
        &mut self,
        data: &[T],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), ()> {
        let Some(stream) = self.index_stream.as_mut() else {
            return Err(());
        };
        stream.buffer.write(device, queue, data);
        Ok(())
    }

    pub fn set_command(&mut self, command: DrawCommand) {
        self.command = command;
    }
}
