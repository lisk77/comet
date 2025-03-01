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

pub struct FlatMap<K, V> {
  map: Vec<MapNode<K, V>
}

impl<K, V> FlatMap<K, V> {
  pub fn new() -> Self {
    Self {
      map: Vec::new()
    }
  }

  pub fn insert(key: K, value: V) {
    let node = MapNode::new(key, value);
    self.map.push(node);
  }

  pub fn remove(key: K) {
    for node in self.map {
      if node.key() == key {
        self.map.remove(node);
      }
    }
  }

  pub fn get(key: K) -> Option<&V> {
    for node in self.map {
      if node.key() == key {
        return Some(&node.value);
      }
    }
    return None;
  }

  pub fn get_mut(key: K) -> Option<&mut V> {
    for node in self.map {
      if node.key() == key {
        return Some(&mut node.value);
      }
    }
    return None;
  }
}
