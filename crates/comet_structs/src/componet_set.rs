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

	pub fn powerset(ids: Vec<TypeId>) -> Vec<HashSet<TypeId>> {
		let n = ids.len();
		let mut subsets: Vec<HashSet<TypeId>> = Vec::with_capacity(1 << n);
	
		for mask in 0..(1 << n) {
			let mut subset = HashSet::new();
        	for i in 0..n {
            	if (mask & (1 << i)) != 0 {
                	subset.insert(ids[i].clone());
            	}
        	}
        	subsets.push(subset);
		}
		subsets.remove(0);

		subsets
	}

	pub fn is_subset(&self, other: &ComponentSet) -> bool {
		self.set.is_subset(&other.set)
	}

	pub fn to_vec(&self) -> Vec<TypeId> {
		self.set.iter().cloned().collect()
	}
}

impl Hash for ComponentSet {
	fn hash<H: Hasher>(&self, state: &mut H) {
		let mut types: Vec<TypeId> = self.set.iter().cloned().collect();
		types.sort();
		types.hash(state);
	}
}