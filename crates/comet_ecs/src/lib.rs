pub use bundles::Bundle;
pub use comet_math as math;
pub use component::*;
pub use component_derive::*;
pub use component_tuple::{ComponentTuple, ComponentValueTuple};
pub use entity::*;
pub use id::*;
pub use prefabs::{ErasedComponent, PrefabFactory};
pub use query::{
    Added, Changed, Query, QueryParam, QuerySpec, QuerySpecMut, With, WithAny, Without, WithoutAny,
};
pub use scene::*;
pub use scene_commands::{SceneCommand, SceneCommands};
pub use sparse_set::SparseSet;

pub type Tick = u32;

mod archetypes;
mod bundles;
mod component;
mod component_tuple;
mod entity;
mod id;
pub mod prefabs;
mod query;
mod query_plan_cache;
mod scene;
mod scene_commands;
mod scene_internals;
mod sparse_set;
