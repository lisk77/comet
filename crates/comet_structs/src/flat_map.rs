#[derive(Clone)]
pub struct MapNode<K, V> {
    key: K,
    value: V
}

impl<K, V> MapNode<K,V> {
    pub fn new(key: K, value: V) -> Self {
        Self {
            key,
            value
        }
    }

    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn value(&self) -> &V {
        &self.value
    }
}

#[derive(Clone)]
pub struct FlatMap<K: PartialEq, V> {
    map: Vec<MapNode<K, V>>
}

impl<K: PartialEq, V> FlatMap<K, V> {
    pub fn new() -> Self {
        Self {
            map: Vec::new()
        }
    }

    pub fn keys(&self) -> Vec<&K> {
        self.map.iter().map(|node| node.key()).collect::<Vec<&K>>()
    }

    pub fn values(&self) -> Vec<&V> {
        self.map.iter().map(|node| node.value()).collect::<Vec<&V>>()
    }

    pub fn keys_mut(&mut self) -> impl Iterator<Item = &mut K> {
        self.map.iter_mut().map(|node| &mut node.key)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.map.iter_mut().map(|node| &mut node.value)
    }

    pub fn insert(&mut self, key: K, value: V) {
        let node = MapNode::new(key, value);
        self.map.push(node);
    }

    pub fn remove(&mut self, key: &K) {
        self.map.retain(|node| node.key() != key);
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.iter()
            .find(|node| node.key() == key)
            .map(|node| node.value())
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.map.iter_mut()
            .find(|node| node.key() == key)
            .map(|node| node.value_mut())
    }

    pub fn contains(&self, key: &K) -> bool {
        self.map.iter().any(|node| node.key() == key)
    }
}

// Add this method to MapNode to allow mutable access to value
impl<K, V> MapNode<K, V> {
    fn value_mut(&mut self) -> &mut V {
        &mut self.value
    }
}