use comet_structs::ComponentSet;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct Archetypes {
    archetypes: HashMap<ComponentSet, HashSet<u32>>,
}

impl Archetypes {
    pub fn new() -> Self {
        Self {
            archetypes: HashMap::new(),
        }
    }

    pub fn component_sets(&self) -> Vec<ComponentSet> {
        self.archetypes.keys().cloned().collect()
    }

    pub fn create_archetype(&mut self, components: ComponentSet) {
        self.archetypes.insert(components, HashSet::new());
    }

    pub fn get_archetype(&self, components: &ComponentSet) -> Option<&HashSet<u32>> {
        self.archetypes.get(components)
    }

    pub fn add_entity_to_archetype(&mut self, components: &ComponentSet, entity: u32) {
        if let Some(archetype) = self.archetypes.get_mut(components) {
            archetype.insert(entity);
        }
    }

    pub fn remove_entity_from_archetype(&mut self, components: &ComponentSet, entity: u32) {
        if let Some(archetype) = self.archetypes.get_mut(components) {
            archetype.retain(|&id| id != entity);
        }
    }

    pub fn remove_archetype(&mut self, components: &ComponentSet) {
        self.archetypes.remove(components);
    }

    pub fn contains_archetype(&self, components: &ComponentSet) -> bool {
        self.archetypes.contains_key(components)
    }
}
