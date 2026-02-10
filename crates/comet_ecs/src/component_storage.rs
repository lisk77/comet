use comet_structs::Column;
use std::collections::HashMap;
use crate::{Component, EntityId, SparseSet};
use std::any::TypeId;

pub struct ComponentStore {
    entities: SparseSet<EntityId>,
    components: Column
}

impl ComponentStore {
    pub fn new<C: Component + 'static>(capacity: usize) -> Self {
        Self {
            entities: SparseSet::new(capacity, 64),
            components: Column::new::<C>(capacity),
        }
    }

    pub fn insert<C: Component + 'static>(&mut self, entity_id: EntityId, component: C) {
        let index = entity_id.index as usize;
        if let Some(dense_index) = self.entities.dense_index(index) {
            let _ = self.components.set::<C>(dense_index, component);
        } else {
            self.entities.insert(index, entity_id);
            self.components.push::<C>(component);
        }
    }

    pub fn remove<C: Component + 'static>(&mut self, entity_id: EntityId) -> Option<C> {
        let index = entity_id.index as usize;
        let dense_index = self.entities.dense_index(index)?;
        let last_index = self.entities.dense_len().saturating_sub(1);
        if dense_index != last_index {
            self.components.swap(dense_index, last_index);
        }
        let removed = self.components.remove::<C>(last_index);
        let _ = self.entities.remove(index);
        removed
    }

    pub fn remove_any(&mut self, entity_id: EntityId) -> bool {
        let index = entity_id.index as usize;
        let dense_index = match self.entities.dense_index(index) {
            Some(idx) => idx,
            None => return false,
        };
        let last_index = self.entities.dense_len().saturating_sub(1);
        if dense_index != last_index {
            self.components.swap(dense_index, last_index);
        }
        self.components.remove_any(last_index);
        let _ = self.entities.remove(index);
        true
    }

    pub fn get<C: Component + 'static>(&self, entity_id: EntityId) -> Option<&C> {
        let index = entity_id.index as usize;
        if let Some(ent) = self.entities.get(index) {
            if ent.index == entity_id.index && ent.gen == entity_id.gen {
                if let Some(dense_index) = self.entities.dense_index(index) {
                    return self.components.get::<C>(dense_index);
                }
            }
        }
        None

    }

    pub fn get_mut<C: Component + 'static>(&mut self, entity_id: EntityId) -> Option<&mut C> {
        let index = entity_id.index as usize;
        if let Some(ent) = self.entities.get(index) {
            if ent.index == entity_id.index && ent.gen == entity_id.gen {
                if let Some(dense_index) = self.entities.dense_index(index) {
                    return self.components.get_mut::<C>(dense_index);
                }
            }
        }
        None
    }

    pub fn set<C: Component + 'static>(&mut self, entity_id: EntityId, component: C) -> Option<()> {
        let index = entity_id.index as usize;
        if let Some(ent) = self.entities.get(index) {
            if ent.index == entity_id.index && ent.gen == entity_id.gen {
                if let Some(dense_index) = self.entities.dense_index(index) {
                    return self.components.set::<C>(dense_index, component);
                }
            }
        }
        None
    }

    pub fn has<C: Component + 'static>(&self, entity_id: EntityId) -> bool {
        let index = entity_id.index as usize;
        if let Some(ent) = self.entities.get(index) {
            return ent.index == entity_id.index
                && ent.gen == entity_id.gen
                && self.entities.dense_index(index).is_some();
        }
        false
    }

    pub fn get_entities(&self) -> Vec<EntityId> {
        self.entities.dense().to_vec()
    }
}

pub type ComponentStorage = HashMap<TypeId, ComponentStore>;
pub trait ComponentStorageExt {
    fn new() -> Self;
    fn contains_component(&self, type_id: &TypeId) -> bool;
    fn register_component<C: Component + 'static>(&mut self, capacity: usize);
    fn deregister_component<C: Component + 'static>(&mut self);
    fn get_two_mut(
        &mut self,
        a: &TypeId,
        b: &TypeId,
    ) -> (Option<&mut ComponentStore>, Option<&mut ComponentStore>);
    fn add_component<C: Component + 'static>(&mut self, entity_id: EntityId, component: C);
    fn remove_component<C: Component + 'static>(&mut self, entity_id: EntityId) -> Option<C>;
    fn get_component<C: Component + 'static>(&self, entity_id: EntityId) -> Option<&C>;
    fn get_component_mut<C: Component + 'static>(&mut self, entity_id: EntityId) -> Option<&mut C>;
    fn set_component<C: Component + 'static>(&mut self, entity_id: EntityId, component: C) -> Option<()>;
    fn has_component<C: Component + 'static>(&self, entity_id: EntityId) -> bool;
    fn remove_entity(&mut self, entity_id: EntityId);
}

impl ComponentStorageExt for ComponentStorage {
    fn new() -> Self {
        HashMap::new()
    }

    fn contains_component(&self, type_id: &TypeId) -> bool {
        self.contains_key(type_id)
    }

    fn register_component<C: Component + 'static>(&mut self, capacity: usize) {
        let type_id = C::type_id();
        if !self.contains_key(&type_id) {
            self.insert(type_id, ComponentStore::new::<C>(capacity));
        }
    }

    fn deregister_component<C: Component + 'static>(&mut self) {
        let type_id = C::type_id();
        self.remove(&type_id);
    }

    fn get_two_mut(
        &mut self,
        a: &TypeId,
        b: &TypeId,
    ) -> (Option<&mut ComponentStore>, Option<&mut ComponentStore>) {
        if a == b {
            return (None, None);
        }

        let a_ptr = self.get_mut(a).map(|c| c as *mut ComponentStore);
        let b_ptr = self.get_mut(b).map(|c| c as *mut ComponentStore);

        match (a_ptr, b_ptr) {
            (Some(a_ptr), Some(b_ptr)) => unsafe { (Some(&mut *a_ptr), Some(&mut *b_ptr)) },
            _ => (None, None),
        }
    }

    fn add_component<C: Component + 'static>(&mut self, entity_id: EntityId, component: C) {
        let type_id = C::type_id();
        if let Some(store) = self.get_mut(&type_id) {
            store.insert(entity_id, component);
        }
    }

    fn remove_component<C: Component + 'static>(&mut self, entity_id: EntityId) -> Option<C> {
        let type_id = C::type_id();
        self.get_mut(&type_id)?.remove::<C>(entity_id)
    }

    fn get_component<C: Component + 'static>(&self, entity_id: EntityId) -> Option<&C> {
        let type_id = C::type_id();
        self.get(&type_id)?.get::<C>(entity_id)
    }

    fn get_component_mut<C: Component + 'static>(&mut self, entity_id: EntityId) -> Option<&mut C> {
        let type_id = C::type_id();
        self.get_mut(&type_id)?.get_mut::<C>(entity_id)
    }

    fn set_component<C: Component + 'static>(&mut self, entity_id: EntityId, component: C) -> Option<()> {
        let type_id = C::type_id();
        self.get_mut(&type_id)?.set::<C>(entity_id, component)
    }

    fn has_component<C: Component + 'static>(&self, entity_id: EntityId) -> bool {
        let type_id = C::type_id();
        self.get(&type_id)
            .and_then(|store| store.get::<C>(entity_id))
            .is_some()
    }

    fn remove_entity(&mut self, entity_id: EntityId) {
        for store in self.values_mut() {
            let _ = store.remove_any(entity_id);
        }
    }
}
