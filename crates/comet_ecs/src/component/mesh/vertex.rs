use comet_log::cassert;
use comet_macros::Vertex;
use comet_math::{v2, v3, v4};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VertexSemantic {
    Position,
    Normal,
    Tangent,
    TexCoord(u32),
    Color(u32),
    JointIndices(u32),
    JointWeights(u32),
    Custom(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VertexFormat {
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
    Uint8x2,
    Uint8x4,
    Sint8x2,
    Sint8x4,
    Uint16x2,
    Uint16x4,
    Sint16x2,
    Sint16x4,
    Uint32,
    Uint32x2,
    Uint32x3,
    Uint32x4,
    Sint32,
    Sint32x2,
    Sint32x3,
    Sint32x4,
}

impl VertexFormat {
    pub const fn size(self) -> usize {
        match self {
            Self::Float32 | Self::Uint32 | Self::Sint32 => 4,
            Self::Uint8x2 | Self::Sint8x2 => 2,
            Self::Uint8x4 | Self::Sint8x4 | Self::Uint16x2 | Self::Sint16x2 => 4,
            Self::Uint16x4 | Self::Sint16x4 | Self::Float32x2 | Self::Uint32x2 | Self::Sint32x2 => {
                8
            }
            Self::Float32x3 | Self::Uint32x3 | Self::Sint32x3 => 12,
            Self::Float32x4 | Self::Uint32x4 | Self::Sint32x4 => 16,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VertexAttribute {
    semantic: VertexSemantic,
    offset: usize,
    format: VertexFormat,
}

impl VertexAttribute {
    pub const fn new(semantic: VertexSemantic, offset: usize, format: VertexFormat) -> Self {
        Self {
            semantic,
            offset,
            format,
        }
    }

    pub const fn semantic(&self) -> VertexSemantic {
        self.semantic
    }

    pub const fn offset(&self) -> usize {
        self.offset
    }

    pub const fn format(&self) -> VertexFormat {
        self.format
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VertexDescriptor {
    stride: usize,
    attributes: Vec<VertexAttribute>,
}

impl VertexDescriptor {
    pub fn new(stride: usize, attributes: Vec<VertexAttribute>) -> Self {
        cassert!(stride > 0, "vertex stride must be greater than zero");
        cassert!(
            !attributes.is_empty(),
            "vertex must contain at least one attribute"
        );
        cassert!(
            attributes.iter().all(|attribute| {
                attribute
                    .offset
                    .checked_add(attribute.format.size())
                    .is_some_and(|end| end <= stride)
            }),
            "vertex attribute exceeds the vertex stride"
        );
        for (index, attribute) in attributes.iter().enumerate() {
            cassert!(
                attributes[..index]
                    .iter()
                    .all(|existing| existing.semantic != attribute.semantic),
                "vertex descriptor contains duplicate semantics"
            );
        }
        Self { stride, attributes }
    }

    pub fn stride(&self) -> usize {
        self.stride
    }

    pub fn attributes(&self) -> &[VertexAttribute] {
        &self.attributes
    }
}

pub trait Vertex: Send + Sync + 'static {
    fn descriptor() -> VertexDescriptor;
    fn encode(&self, output: &mut Vec<u8>);

    fn encode_slice(vertices: &[Self]) -> Vec<u8>
    where
        Self: Sized,
    {
        let descriptor = Self::descriptor();
        let mut output = Vec::with_capacity(vertices.len() * descriptor.stride());
        for vertex in vertices {
            let start = output.len();
            vertex.encode(&mut output);
            let encoded_size = output.len() - start;
            cassert!(
                encoded_size == descriptor.stride(),
                "encoded vertex size {} does not match descriptor stride {}",
                encoded_size,
                descriptor.stride()
            );
        }
        output
    }
}

pub trait VertexValue {
    const FORMAT: VertexFormat;
    const SIZE: usize;

    fn encode(&self, output: &mut Vec<u8>);
}

macro_rules! impl_scalar_vertex_value {
    ($type:ty, $format:expr) => {
        impl VertexValue for $type {
            const FORMAT: VertexFormat = $format;
            const SIZE: usize = std::mem::size_of::<Self>();

            fn encode(&self, output: &mut Vec<u8>) {
                output.extend_from_slice(&self.to_le_bytes());
            }
        }
    };
}

macro_rules! impl_array_vertex_value {
    ($type:ty, $format:expr) => {
        impl VertexValue for $type {
            const FORMAT: VertexFormat = $format;
            const SIZE: usize = std::mem::size_of::<Self>();

            fn encode(&self, output: &mut Vec<u8>) {
                for value in self {
                    output.extend_from_slice(&value.to_le_bytes());
                }
            }
        }
    };
}

impl_scalar_vertex_value!(f32, VertexFormat::Float32);
impl_scalar_vertex_value!(u32, VertexFormat::Uint32);
impl_scalar_vertex_value!(i32, VertexFormat::Sint32);
impl_array_vertex_value!([f32; 2], VertexFormat::Float32x2);
impl_array_vertex_value!([f32; 3], VertexFormat::Float32x3);
impl_array_vertex_value!([f32; 4], VertexFormat::Float32x4);
impl_array_vertex_value!([u16; 2], VertexFormat::Uint16x2);
impl_array_vertex_value!([u16; 4], VertexFormat::Uint16x4);
impl_array_vertex_value!([i16; 2], VertexFormat::Sint16x2);
impl_array_vertex_value!([i16; 4], VertexFormat::Sint16x4);
impl_array_vertex_value!([u32; 2], VertexFormat::Uint32x2);
impl_array_vertex_value!([u32; 3], VertexFormat::Uint32x3);
impl_array_vertex_value!([u32; 4], VertexFormat::Uint32x4);
impl_array_vertex_value!([i32; 2], VertexFormat::Sint32x2);
impl_array_vertex_value!([i32; 3], VertexFormat::Sint32x3);
impl_array_vertex_value!([i32; 4], VertexFormat::Sint32x4);

impl VertexValue for [u8; 2] {
    const FORMAT: VertexFormat = VertexFormat::Uint8x2;
    const SIZE: usize = 2;

    fn encode(&self, output: &mut Vec<u8>) {
        output.extend_from_slice(self);
    }
}

impl VertexValue for [u8; 4] {
    const FORMAT: VertexFormat = VertexFormat::Uint8x4;
    const SIZE: usize = 4;

    fn encode(&self, output: &mut Vec<u8>) {
        output.extend_from_slice(self);
    }
}

impl VertexValue for [i8; 2] {
    const FORMAT: VertexFormat = VertexFormat::Sint8x2;
    const SIZE: usize = 2;

    fn encode(&self, output: &mut Vec<u8>) {
        output.extend(self.iter().map(|value| *value as u8));
    }
}

impl VertexValue for [i8; 4] {
    const FORMAT: VertexFormat = VertexFormat::Sint8x4;
    const SIZE: usize = 4;

    fn encode(&self, output: &mut Vec<u8>) {
        output.extend(self.iter().map(|value| *value as u8));
    }
}

impl VertexValue for v2 {
    const FORMAT: VertexFormat = VertexFormat::Float32x2;
    const SIZE: usize = 8;

    fn encode(&self, output: &mut Vec<u8>) {
        VertexValue::encode(&[self.x(), self.y()], output);
    }
}

impl VertexValue for v3 {
    const FORMAT: VertexFormat = VertexFormat::Float32x3;
    const SIZE: usize = 12;

    fn encode(&self, output: &mut Vec<u8>) {
        VertexValue::encode(&[self.x(), self.y(), self.z()], output);
    }
}

impl VertexValue for v4 {
    const FORMAT: VertexFormat = VertexFormat::Float32x4;
    const SIZE: usize = 16;

    fn encode(&self, output: &mut Vec<u8>) {
        VertexValue::encode(&[self.x(), self.y(), self.z(), self.w()], output);
    }
}

#[derive(Vertex, Clone, Copy, Debug, PartialEq)]
pub struct ModelVertex {
    #[position]
    pub position: v3,
    #[normal]
    pub normal: v3,
    #[tex_coord(0)]
    pub tex_coord: v2,
}

impl ModelVertex {
    pub const fn new(position: v3, normal: v3, tex_coord: v2) -> Self {
        Self {
            position,
            normal,
            tex_coord,
        }
    }
}
