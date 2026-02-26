/// Handle used to reference entities safely. Contains an index into the entity
/// storage and a generation counter to detect stale handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Entity {
    pub index: u32,
    pub gen: u32,
}

impl Default for Entity {
    fn default() -> Self {
        Self { index: 0, gen: 0 }
    }
}

impl Entity {
    pub fn id(&self) -> Entity {
        *self
    }
}
