pub use bundles::Bundle;
pub use comet_math as math;
pub use component::*;
pub use component_changes::ComponentChange;
pub use component_derive::*;
pub use component_tuple::ComponentTuple;
pub use ecs_module::{EcsModule, EcsModuleExt};
pub use entity::*;
pub use error::EcsError;
pub use id::*;
pub use material::Material;
pub use mesh::{Mesh, MeshData, MeshVertex};
pub use prefabs::{ErasedComponent, PrefabFactory};
pub use query::{
    Added, Changed, Query, QueryParam, QuerySpec, QuerySpecMut, With, WithAny, Without, WithoutAny,
};
pub use scene::*;
pub use scene_commands::{SceneCommand, SceneCommands};
pub use sparse_set::SparseSet;

pub type Tick = u32;

#[doc(hidden)]
pub mod __private {
    pub use comet_structs::Column;
}

mod archetypes;
mod bundles;
mod component;
mod component_changes;
mod component_tuple;
mod ecs_module;
mod entity;
mod error;
mod id;
mod material;
mod mesh;
pub mod prefabs;
mod query;
mod query_plan_cache;
mod scene;
mod scene_commands;
mod scene_internals;
mod sparse_set;
