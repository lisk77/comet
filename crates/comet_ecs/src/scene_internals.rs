use crate::Tick;
use std::{any::TypeId, sync::Arc};

#[derive(Clone)]
pub(crate) struct BundleSpawnPlan {
    pub(crate) archetype: usize,
    pub(crate) column_indices: Arc<[usize]>,
}

#[derive(Clone)]
pub(crate) struct BundleAddPlan {
    pub(crate) target_arch: usize,
    pub(crate) col_indices: Arc<[usize]>,
    pub(crate) type_ids: Arc<[TypeId]>,
}

#[derive(Clone, Copy)]
pub(crate) struct ComponentChangeState {
    pub(crate) added_tick: Tick,
    pub(crate) changed_tick: Tick,
}
