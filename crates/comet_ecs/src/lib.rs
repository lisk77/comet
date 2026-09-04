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
pub use prefabs::{ErasedComponent, PrefabFactory};
#[doc(hidden)]
pub use query::QueryItem;
pub use query::{
    Added, AddedAll, AddedAny, AddedAtLeast, AddedAtLeastOf, AddedAtMost, AddedAtMostOf,
    AddedExactly, AddedExactlyOf, Amount, AtLeast, AtLeastOf, AtMost, AtMostOf, Changed,
    ChangedAll, ChangedAny, ChangedAtLeast, ChangedAtLeastOf, ChangedAtMost, ChangedAtMostOf,
    ChangedExactly, ChangedExactlyOf, Count, CountOf, Exactly, ExactlyOf, First, Last, Modified,
    ModifiedAll, ModifiedAny, ModifiedAtLeast, ModifiedAtLeastOf, ModifiedAtMost, ModifiedAtMostOf,
    ModifiedExactly, ModifiedExactlyOf, Not, Or, Query, QueryParam, QuerySpec, QuerySpecMut, Range,
    Skip, Spawned, Take, With, WithAll, WithAny, Without, WithoutAll, WithoutAny,
};
pub use scene::*;
pub use scene_commands::{SceneCommand, SceneCommands};
pub use sparse_set::SparseSet;

pub type Tick = u32;

#[doc(hidden)]
pub mod __private {
    pub use crate::bundles::BundleInfo;
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
pub mod prefabs;
mod query;
mod query_plan_cache;
mod scene;
mod scene_commands;
mod scene_internals;
mod sparse_set;
