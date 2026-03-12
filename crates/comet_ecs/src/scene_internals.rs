use crate::Tick;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct BundleSpawnPlan {
    pub(crate) archetype: usize,
    pub(crate) column_indices: Arc<[usize]>,
}

#[derive(Clone, Copy)]
pub(crate) struct ComponentChangeState {
    pub(crate) added_tick: Tick,
    pub(crate) changed_tick: Tick,
}
