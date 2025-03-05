use std::collections::HashMap;
use comet_structs::ComponentSet;

#[derive(Debug, Clone)]
pub struct Archetypes {
	archetypes: HashMap<ComponentSet, Vec<u32>>
}

impl Archetypes {
	pub fn new() -> Self {
		Self {
			archetypes: HashMap::new()
		}
	}

	pub fn component_sets(&self) -> Vec<ComponentSet> {
		self.archetypes.keys().cloned().collect()
	}

	pub fn create_archetype(&mut self, components: ComponentSet) {
		self.archetypes.insert(components, Vec::new());
	}

	pub fn get_archetype(&self, components: &ComponentSet) -> Option<&Vec<u32>> {
		self.archetypes.get(components)
	}

	pub fn get_archetype_mut(&mut self, components: &ComponentSet) -> Option<&mut Vec<u32>> {
		self.archetypes.get_mut(components)
	}

	pub fn add_entity_to_archetype(&mut self, components: &ComponentSet, entity: u32) {
		if let Some(archetype) = self.archetypes.get_mut(components) {
			archetype.push(entity);
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