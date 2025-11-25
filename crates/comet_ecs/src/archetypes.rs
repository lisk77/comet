use comet_structs::ComponentSet;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Archetypes {
    archetypes: HashMap<ComponentSet, Vec<u32>>,
}

impl Archetypes {
    pub fn new() -> Self {
        Self {
            archetypes: HashMap::new(),
        }
    }

    pub fn component_sets(&self) -> impl Iterator<Item = &ComponentSet> {
        self.archetypes.keys()
    }

    pub fn create_archetype(&mut self, components: ComponentSet) {
        self.archetypes.entry(components).or_insert_with(Vec::new);
    }

    pub fn get_archetype(&self, components: &ComponentSet) -> Option<&Vec<u32>> {
        self.archetypes.get(components)
    }

    pub fn add_entity_to_archetype(&mut self, components: &ComponentSet, entity: u32) {
        if let Some(archetype) = self.archetypes.get_mut(components) {
            if !archetype.iter().any(|&e| e == entity) {
                archetype.push(entity);
            }
        }
    }

    pub fn remove_entity_from_archetype(&mut self, components: &ComponentSet, entity: u32) {
        if let Some(archetype) = self.archetypes.get_mut(components) {
            if let Some(pos) = archetype.iter().position(|&id| id == entity) {
                archetype.swap_remove(pos);
            }
        }
    }

    pub fn remove_archetype(&mut self, components: &ComponentSet) {
        self.archetypes.remove(components);
    }

    pub fn contains_archetype(&self, components: &ComponentSet) -> bool {
        self.archetypes.contains_key(components)
    }
}
