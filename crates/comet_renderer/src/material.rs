use std::any::TypeId;

use comet_ecs::Component;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShaderDataSchema {
    name: &'static str,
    size: usize,
    alignment: usize,
}

impl ShaderDataSchema {
    pub const fn new(name: &'static str, size: usize, alignment: usize) -> Self {
        Self {
            name,
            size,
            alignment,
        }
    }

    pub const fn name(self) -> &'static str {
        self.name
    }

    pub const fn size(self) -> usize {
        self.size
    }

    pub const fn alignment(self) -> usize {
        self.alignment
    }
}

pub trait ShaderData: 'static {
    type Encoded: bytemuck::Pod + bytemuck::Zeroable;

    fn encode(&self) -> Self::Encoded;
    fn schema() -> &'static ShaderDataSchema;
}

macro_rules! impl_scalar_shader_data {
    ($ty:ty, $name:literal) => {
        impl ShaderData for $ty {
            type Encoded = Self;

            fn encode(&self) -> Self::Encoded {
                *self
            }

            fn schema() -> &'static ShaderDataSchema {
                static SCHEMA: ShaderDataSchema =
                    ShaderDataSchema::new($name, size_of::<$ty>(), align_of::<$ty>());
                &SCHEMA
            }
        }
    };
}

impl_scalar_shader_data!(f32, "f32");
impl_scalar_shader_data!(u32, "u32");
impl_scalar_shader_data!(i32, "i32");

impl ShaderData for [f32; 4] {
    type Encoded = Self;

    fn encode(&self) -> Self::Encoded {
        *self
    }

    fn schema() -> &'static ShaderDataSchema {
        static SCHEMA: ShaderDataSchema = ShaderDataSchema::new("vec4<f32>", 16, 16);
        &SCHEMA
    }
}

#[derive(Clone, Copy)]
pub struct UniformDescriptor {
    name: &'static str,
    schema: fn() -> &'static ShaderDataSchema,
}

impl UniformDescriptor {
    pub const fn new(name: &'static str, schema: fn() -> &'static ShaderDataSchema) -> Self {
        Self { name, schema }
    }

    pub const fn name(self) -> &'static str {
        self.name
    }

    pub fn schema(self) -> &'static ShaderDataSchema {
        (self.schema)()
    }
}

pub struct MaterialDescriptor {
    name: &'static str,
    shader: &'static str,
    uniforms: &'static [UniformDescriptor],
}

impl MaterialDescriptor {
    pub const fn new(
        name: &'static str,
        shader: &'static str,
        uniforms: &'static [UniformDescriptor],
    ) -> Self {
        Self {
            name,
            shader,
            uniforms,
        }
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub const fn shader(&self) -> &'static str {
        self.shader
    }

    pub const fn uniforms(&self) -> &'static [UniformDescriptor] {
        self.uniforms
    }
}

pub struct EncodedUniform {
    name: &'static str,
    schema: &'static ShaderDataSchema,
    bytes: Vec<u8>,
}

impl EncodedUniform {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn schema(&self) -> &'static ShaderDataSchema {
        self.schema
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[derive(Default)]
pub struct MaterialEncoder {
    uniforms: Vec<EncodedUniform>,
}

impl MaterialEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write_uniform<T: ShaderData>(&mut self, name: &'static str, value: &T) {
        let encoded = value.encode();
        self.uniforms.push(EncodedUniform {
            name,
            schema: T::schema(),
            bytes: bytemuck::bytes_of(&encoded).to_vec(),
        });
    }

    pub fn uniforms(&self) -> &[EncodedUniform] {
        &self.uniforms
    }
}

pub trait Material: Component {
    fn material_type_id(&self) -> TypeId {
        self.component_type_id()
    }

    fn descriptor(&self) -> &'static MaterialDescriptor;
    fn encode(&self, encoder: &mut MaterialEncoder);
}
