mod vertex;

pub use vertex::*;

use comet_log::cassert;
use comet_math::{v2, v3};
use std::sync::Arc;

use crate::Component;

#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    data: Arc<MeshData>,
}

#[derive(Debug, PartialEq)]
pub struct MeshData {
    vertex_descriptor: VertexDescriptor,
    vertices: Arc<[u8]>,
    vertex_count: u32,
    indices: Arc<[u32]>,
}

impl Mesh {
    pub fn new<V: Vertex>(vertices: Vec<V>, indices: Vec<u32>) -> Self {
        cassert!(
            !vertices.is_empty(),
            "mesh must contain at least one vertex"
        );
        cassert!(
            vertices.len() <= u32::MAX as usize,
            "mesh contains too many vertices"
        );
        cassert!(
            indices.iter().all(|&index| index < vertices.len() as u32),
            "mesh index is out of bounds"
        );

        let vertex_descriptor = V::descriptor();
        let vertex_count = vertices.len() as u32;
        let mut encoded_vertices = Vec::with_capacity(vertices.len() * vertex_descriptor.stride());
        for vertex in &vertices {
            let start = encoded_vertices.len();
            vertex.encode(&mut encoded_vertices);
            let encoded_size = encoded_vertices.len() - start;
            cassert!(
                encoded_size == vertex_descriptor.stride(),
                "encoded vertex size {} does not match descriptor stride {}",
                encoded_size,
                vertex_descriptor.stride()
            );
        }
        let vertices = encoded_vertices.into();
        Self {
            data: Arc::new(MeshData {
                vertex_descriptor,
                vertices,
                vertex_count,
                indices: indices.into(),
            }),
        }
    }

    pub fn triangle() -> Self {
        Self::new(
            vec![
                ModelVertex::new(
                    v3::new(-0.5, -0.5, 0.0),
                    v3::new(0.0, 0.0, 1.0),
                    v2::new(0.0, 1.0),
                ),
                ModelVertex::new(
                    v3::new(0.5, -0.5, 0.0),
                    v3::new(0.0, 0.0, 1.0),
                    v2::new(1.0, 1.0),
                ),
                ModelVertex::new(
                    v3::new(0.0, 0.5, 0.0),
                    v3::new(0.0, 0.0, 1.0),
                    v2::new(0.5, 0.0),
                ),
            ],
            vec![0, 1, 2],
        )
    }

    pub fn quad() -> Self {
        Self::new(
            vec![
                ModelVertex::new(
                    v3::new(-0.5, -0.5, 0.0),
                    v3::new(0.0, 0.0, 1.0),
                    v2::new(0.0, 1.0),
                ),
                ModelVertex::new(
                    v3::new(0.5, -0.5, 0.0),
                    v3::new(0.0, 0.0, 1.0),
                    v2::new(1.0, 1.0),
                ),
                ModelVertex::new(
                    v3::new(0.5, 0.5, 0.0),
                    v3::new(0.0, 0.0, 1.0),
                    v2::new(1.0, 0.0),
                ),
                ModelVertex::new(
                    v3::new(-0.5, 0.5, 0.0),
                    v3::new(0.0, 0.0, 1.0),
                    v2::new(0.0, 0.0),
                ),
            ],
            vec![0, 1, 2, 2, 3, 0],
        )
    }

    pub fn data(&self) -> &Arc<MeshData> {
        &self.data
    }
}

impl MeshData {
    pub fn vertex_descriptor(&self) -> &VertexDescriptor {
        &self.vertex_descriptor
    }

    pub fn vertices(&self) -> &[u8] {
        &self.vertices
    }

    pub fn vertex_count(&self) -> u32 {
        self.vertex_count
    }

    pub fn indices(&self) -> &[u32] {
        &self.indices
    }
}

impl Component for Mesh {}
