use bit_set::BitSet;

#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
	id: u32,
	components: BitSet
}

impl Entity {
	pub(crate) fn new(id: u32) -> Self {
		Self {
			id,
			components: BitSet::new()
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
