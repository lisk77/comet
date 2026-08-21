#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct SpriteInstance {
    position_size: [f32; 4],
    rotation: [f32; 2],
    uv_min: [f32; 2],
    uv_max: [f32; 2],
    color: [f32; 4],
}

impl SpriteInstance {
    pub fn new(
        position: [f32; 2],
        half_size: [f32; 2],
        rotation_radians: f32,
        uv_rect: [f32; 4],
        color: [f32; 4],
    ) -> Self {
        Self {
            position_size: [position[0], position[1], half_size[0], half_size[1]],
            rotation: [rotation_radians.cos(), rotation_radians.sin()],
            uv_min: [uv_rect[0], uv_rect[1]],
            uv_max: [uv_rect[2], uv_rect[3]],
            color,
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
            3 => Float32x4,
            4 => Float32x2,
            5 => Float32x2,
            6 => Float32x2,
            7 => Float32x4
        ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRIBUTES,
        }
    }
}
