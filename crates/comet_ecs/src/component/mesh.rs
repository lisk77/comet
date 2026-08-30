use std::sync::Arc;

use crate::Component;

#[derive(Clone, Copy, Debug, PartialEq)]
struct MeshVertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
    color: [f32; 4],
}

impl MeshVertex {
    fn new(position: [f32; 3], tex_coords: [f32; 2], color: [f32; 4]) -> Self {
        Self {
            position,
            tex_coords,
            color,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    data: Arc<MeshData>,
}

#[derive(Debug, PartialEq)]
pub struct MeshData {
    vertices: Vec<MeshVertex>,
    indices: Vec<u32>,
}

impl Mesh {
    fn new(vertices: Vec<MeshVertex>, indices: Vec<u32>) -> Self {
        assert!(
            !vertices.is_empty(),
            "mesh must contain at least one vertex"
        );
        assert!(
            indices.iter().all(|&index| index < vertices.len() as u32),
            "mesh index is out of bounds"
        );
        Self {
            data: Arc::new(MeshData { vertices, indices }),
        }
    }

    pub fn triangle() -> Self {
        Self::new(
            vec![
                MeshVertex::new([-0.5, -0.5, 0.0], [0.0, 1.0], [1.0; 4]),
                MeshVertex::new([0.5, -0.5, 0.0], [1.0, 1.0], [1.0; 4]),
                MeshVertex::new([0.0, 0.5, 0.0], [0.5, 0.0], [1.0; 4]),
            ],
            vec![0, 1, 2],
        )
    }

    pub fn quad() -> Self {
        Self::new(
            vec![
                MeshVertex::new([-0.5, -0.5, 0.0], [0.0, 1.0], [1.0; 4]),
                MeshVertex::new([0.5, -0.5, 0.0], [1.0, 1.0], [1.0; 4]),
                MeshVertex::new([0.5, 0.5, 0.0], [1.0, 0.0], [1.0; 4]),
                MeshVertex::new([-0.5, 0.5, 0.0], [0.0, 0.0], [1.0; 4]),
            ],
            vec![0, 1, 2, 2, 3, 0],
        )
    }

    pub fn data(&self) -> &Arc<MeshData> {
        &self.data
    }
}

impl Component for Mesh {}
