#[derive(Clone)]
pub struct FlatMap<K: PartialEq, V> {
    map: Vec<(K,V)>
}

impl<K: PartialEq + Clone, V: Clone> FlatMap<K, V> {
    pub fn new() -> Self {
        Self {
            map: Vec::new()
        }
    }

    pub fn keys(&self) -> Vec<K> {
        self.map.iter().map(|node| node.0.clone()).collect::<Vec<K>>()
    }

    pub fn values(&self) -> Vec<V> {
        self.map.iter().map(|node| node.1.clone()).collect::<Vec<V>>()
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.map.push((key,value));
    }

    pub fn remove(&mut self, key: &K) {
        self.map.retain(|node| node.0 != *key);
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        if let Some(node) = self.map.iter().find(|node| node.0 == *key) {
            Some(&node.1)
        }
        else {
            None
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if let Some(node) = self.map.iter_mut().find(|node| node.0 == *key) {
            Some(&mut node.1)
        }
        else {
            None
        }
    }

    pub fn contains(&self, key: &K) -> bool {
        self.map.iter().any(|node| node.0 == *key)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut (K, V)> {
        self.map.iter_mut()
    }
}