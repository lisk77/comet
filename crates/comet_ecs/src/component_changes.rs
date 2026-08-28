use crate::{Entity, Tick};
use std::any::TypeId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemovedComponent {
    pub entity: Entity,
    pub component_type: TypeId,
}

#[derive(Clone, Copy)]
pub(crate) struct ComponentChangeState {
    pub(crate) added_tick: Tick,
    pub(crate) changed_tick: Tick,
}
