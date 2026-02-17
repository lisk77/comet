pub use comet_math as math;
pub use component::*;
pub use component_derive::*;
pub use entity::*;
pub use id::*;
pub use bundles::Bundle;
pub use prefabs::ErasedComponent;
pub use prefabs::PrefabFactory;
pub use scene::*;
pub use sparse_set::SparseSet;

mod archetypes;
mod bundles;
mod component;
mod entity;
mod id;
pub mod prefabs;
mod scene;
mod sparse_set;
