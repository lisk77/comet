use bit_set::BitSet;
use crate::ComponentSet;

#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
	id: u32,
	components: BitSet
}

impl Entity {
	pub fn new(id: u32) -> Self {
		let mut components = BitSet::new();
		components.insert(0);
		Self {
			id,
			components
		}
	}

	pub fn id(&self) -> &u32 {
		&self.id
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