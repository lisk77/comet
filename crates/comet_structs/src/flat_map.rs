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

pub struct FlatMap<K: PartialEq, V> {
  map: Vec<MapNode<K, V>>
}

impl<K: PartialEq, V> FlatMap<K, V> {
  pub fn new() -> Self {
    Self {
      map: Vec::new()
    }
  }

  pub fn insert(&mut self, key: K, value: V) {
    let node = MapNode::new(key, value);
    self.map.push(node);
  }

  pub fn remove(&mut self, key: K) {
    for node in self.map {
      if node.key() == *key {
        self.map.retain(|&n| n.key() != *key);
      }
    }
  }

  pub fn get(&self, key: K) -> Option<&V> {
    for node in self.map {
      if node.key() == *key {
        return Some(&node.value);
      }
    }
    None
  }

  pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
    for mut node in self.map {
      if node.key() == *key {
        return Some(node.value);
      }
    }
    None
  }
}
