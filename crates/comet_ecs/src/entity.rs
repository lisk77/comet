use bit_set::BitSet;

/// Handle used to reference entities safely. Contains an index into the entity
/// storage and a generation counter to detect stale handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityId {
    pub index: u32,
    pub gen: u32,
}

impl Default for EntityId {
    fn default() -> Self {
        Self { index: 0, gen: 0 }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    id: EntityId,
    components: BitSet,
}

impl Entity {
    pub(crate) fn new(index: u32, gen: u32) -> Self {
        Self {
            id: EntityId { index, gen },
            components: BitSet::new(),
        }
    }

    pub fn id(&self) -> EntityId {
        self.id
    }

    pub(crate) fn add_component(&mut self, component_index: usize) {
        self.components.insert(component_index);
    }

    pub(crate) fn remove_component(&mut self, component_index: usize) {
        self.components.remove(component_index);
    }

    pub(crate) fn get_components(&self) -> &BitSet {
        &self.components
    }
}
