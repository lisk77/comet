use std::any::TypeId;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentSet {
	set: HashSet<TypeId>
}

impl ComponentSet {
	pub fn new() -> Self {
		Self {
			set: HashSet::new()
		}
	}

	pub fn from_ids(ids: Vec<TypeId>) -> Self {
		Self {
			set: ids.into_iter().collect()
		}
	}

	pub fn is_subset(&self, other: &ComponentSet) -> bool {
		self.set.is_subset(&other.set)
	}
}

impl Hash for ComponentSet {
	fn hash<H: Hasher>(&self, state: &mut H) {
		let mut types: Vec<TypeId> = self.set.iter().cloned().collect();
		types.sort();
		types.hash(state);
	}
}