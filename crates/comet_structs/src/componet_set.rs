use std::any::TypeId;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentSet {
    set: HashSet<TypeId>,
}

impl ComponentSet {
    pub fn new() -> Self {
        Self {
            set: HashSet::new(),
        }
    }

    pub fn from_ids(ids: Vec<TypeId>) -> Self {
        Self {
            set: ids.into_iter().collect(),
        }
    }

    pub fn compute_subsets_up_to_size_3(ids: Vec<TypeId>) -> Vec<ComponentSet> {
        let mut result = Vec::new();
        let n = ids.len();

        for i in 0..n {
            result.push(ComponentSet::from_ids(vec![ids[i]]));
        }

        for i in 0..n {
            for j in (i + 1)..n {
                result.push(ComponentSet::from_ids(vec![ids[i], ids[j]]));
            }
        }

        for i in 0..n {
            for j in (i + 1)..n {
                for k in (j + 1)..n {
                    result.push(ComponentSet::from_ids(vec![ids[i], ids[j], ids[k]]));
                }
            }
        }

        result
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

    pub fn size(&self) -> usize {
        self.set.len()
    }
}

impl Hash for ComponentSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut types: Vec<TypeId> = self.set.iter().cloned().collect();
        types.sort();
        types.hash(state);
    }
}
