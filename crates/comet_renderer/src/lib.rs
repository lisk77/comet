pub(crate) mod batch;
mod camera;
pub mod draw_batch;
pub(crate) mod gizmo_registry;
pub mod gpu_texture;
pub mod render_commands;
pub mod render_events;
pub mod render_graph;
pub mod render_pass;
pub mod render_resources;
pub mod render_state;
pub mod renderer2d;
pub mod sprite_instance;
pub mod vertex;

pub use draw_batch::{
    DrawCommand, GeometryDescriptor, IndexStreamDescriptor, VertexStreamDescriptor,
};
pub use gpu_texture::*;
pub use render_commands::PassDescriptor;
pub use render_graph::nodes::{PassNode, PostProcessNode};
pub use render_graph::{BuildContext, NodeState, RenderGraph, RenderNode};
pub use render_pass::{LoadOp, PassOutput};
pub use renderer2d::{RenderHandle2D, RenderHandle2DExt, Renderer2D, Renderer2DModule};
pub use sprite_instance::SpriteInstance;
pub use vertex::Vertex;
