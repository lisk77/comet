use crate::{FlatMap, SparseSet};
use comet_log::*;
use std::any::TypeId;

pub type ComponentStorage = FlatMap<TypeId, SparseSet>;

impl ComponentStorage {
    pub fn register_component<T: 'static>(&mut self, capacity: usize) {
        if !self.contains(&TypeId::of::<T>()) {
            self.insert(TypeId::of::<T>(), SparseSet::new::<T>(capacity, 1000));
        } else {
            error!("Component {:?} already exists", TypeId::of::<T>());
        }
    }

    pub fn deregister_component<T: 'static>(&mut self) {
        if self.contains(&TypeId::of::<T>()) {
            self.remove(&TypeId::of::<T>());
        } else {
            error!("Component {:?} does not exist", TypeId::of::<T>());
        }
    }

    pub fn set_component<T: 'static>(&mut self, index: usize, element: T) {
        if let Some(sparse_set) = self.get_mut(&TypeId::of::<T>()) {
            sparse_set.insert(index, element);
        } else {
            error!("Component {:?} is not registered", TypeId::of::<T>());
        }
    }

    pub fn remove_component<T: 'static>(&mut self, index: usize) -> Option<T> {
        if let Some(sparse_set) = self.get_mut(&TypeId::of::<T>()) {
            sparse_set.remove(index)
        } else {
            error!("Component {:?} is not registered", TypeId::of::<T>());
            None
        }
    }

    pub fn get_component<T: 'static>(&self, index: usize) -> Option<&T> {
        if let Some(sparse_set) = self.get(&TypeId::of::<T>()) {
            sparse_set.get(index)
        } else {
            error!("Component {:?} is not registered", TypeId::of::<T>());
            None
        }
    }

    pub fn get_component_mut<T: 'static>(&mut self, index: usize) -> Option<&mut T> {
        if let Some(sparse_set) = self.get_mut(&TypeId::of::<T>()) {
            sparse_set.get_mut(index)
        } else {
            error!("Component {:?} is not registered", TypeId::of::<T>());
            None
        }
    }
}
