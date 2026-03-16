#[derive(Clone)]
pub struct FlatMap<K: PartialEq, V> {
    map: Vec<(K, V)>,
}

impl<K: PartialEq + Clone, V: Clone> FlatMap<K, V> {
    pub fn new() -> Self {
        Self { map: Vec::new() }
    }

    pub fn keys(&self) -> Vec<K> {
        self.map
            .iter()
            .map(|node| node.0.clone())
            .collect::<Vec<K>>()
    }

    pub fn values(&self) -> Vec<V> {
        self.map
            .iter()
            .map(|node| node.1.clone())
            .collect::<Vec<V>>()
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.map.push((key, value));
    }

    pub fn remove(&mut self, key: &K) {
        self.map.retain(|node| node.0 != *key);
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        if let Some(node) = self.map.iter().find(|node| node.0 == *key) {
            Some(&node.1)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if let Some(node) = self.map.iter_mut().find(|node| node.0 == *key) {
            Some(&mut node.1)
        } else {
            None
        }
    }

    pub fn get_two_mut(&mut self, a: &K, b: &K) -> (Option<&mut V>, Option<&mut V>) {
        if a == b {
            let first = self.get_mut(a);
            return (first, None);
        }

        let mut first_index = None;
        let mut second_index = None;

        for (idx, (key, _)) in self.map.iter().enumerate() {
            if key == a {
                first_index = Some(idx);
            } else if key == b {
                second_index = Some(idx);
            }
        }

        match (first_index, second_index) {
            (Some(i), Some(j)) => {
                if i < j {
                    let (left, right) = self.map.split_at_mut(j);
                    (Some(&mut left[i].1), Some(&mut right[0].1))
                } else {
                    let (left, right) = self.map.split_at_mut(i);
                    (Some(&mut right[0].1), Some(&mut left[j].1))
                }
            }
            (Some(i), None) => (self.map.get_mut(i).map(|pair| &mut pair.1), None),
            (None, Some(j)) => (None, self.map.get_mut(j).map(|pair| &mut pair.1)),
            _ => (None, None),
        }
    }

    pub fn contains(&self, key: &K) -> bool {
        self.map.iter().any(|node| node.0 == *key)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut (K, V)> {
        self.map.iter_mut()
    }
}
