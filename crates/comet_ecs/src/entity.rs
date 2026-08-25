/// Handle used to reference entities safely. Contains an index into the entity
/// storage and a generation counter to detect stale handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Entity {
    pub(crate) index: u32,
    pub(crate) gen: u32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct EntityLocation {
    pub(crate) archetype: usize,
    pub(crate) row: usize,
    pub(crate) gen: u32,
}
