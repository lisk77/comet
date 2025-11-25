use crate::archetypes::Archetypes;
use crate::prefabs::PrefabManager;
use crate::{Component, Entity, EntityId, IdQueue};
use comet_log::*;
use comet_structs::*;
use std::any::TypeId;

pub struct Scene {
    id_queue: IdQueue,
    next_id: u32,
    generations: Vec<u32>,
    entities: Vec<Option<Entity>>,
    components: ComponentStorage,
    archetypes: Archetypes,
    prefabs: PrefabManager,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            id_queue: IdQueue::new(),
            next_id: 0,
            generations: Vec::new(),
            entities: Vec::new(),
            components: ComponentStorage::new(),
            archetypes: Archetypes::new(),
            prefabs: PrefabManager::new(),
        }
    }

    /// Returns the number of how many entities exist in the current Scene.
    pub fn active_entities(&self) -> u32 {
        self.entities.len() as u32 - self.id_queue.size()
    }

    fn get_next_id(&mut self) {
        if self.id_queue.is_empty() {
            self.next_id = self.entities.len() as u32;
            return;
        }
        if self.next_id > self.id_queue.front().unwrap()
            || self.entities[self.next_id as usize] != None
        {
            self.next_id = self.id_queue.dequeue().unwrap();
        }
    }

    fn is_alive(&self, id: EntityId) -> bool {
        self.generations
            .get(id.index as usize)
            .is_some_and(|g| *g == id.gen)
            && self
                .entities
                .get(id.index as usize)
                .is_some_and(|e| e.is_some())
    }

    /// Retuns the `Vec` of `Option<Entity>` which contains all the entities in the current Scene.
    pub fn entities(&self) -> &Vec<Option<Entity>> {
        &self.entities
    }

    /// Creates a new entity and returns its ID.
    pub fn new_entity(&mut self) -> EntityId {
        let index = self.next_id;
        let gen = if (index as usize) >= self.generations.len() {
            self.generations.push(0);
            0
        } else {
            self.generations[index as usize]
        };

        if (index as usize) >= self.entities.len() {
            self.entities.push(Some(Entity::new(index, gen)));
        } else {
            self.entities[index as usize] = Some(Entity::new(index, gen));
        }

        let id = EntityId { index, gen };
        self.get_next_id();
        info!("Created entity! ID: {} (gen {})", id.index, id.gen);
        id
    }

    /// Gets an immutable reference to an entity by its ID.
    pub fn get_entity(&self, entity_id: EntityId) -> Option<&Entity> {
        if !self.is_alive(entity_id) {
            return None;
        }
        self.entities
            .get(entity_id.index as usize)
            .and_then(|e| e.as_ref())
    }

    /// Gets a mutable reference to an entity by its ID.
    pub fn get_entity_mut(&mut self, entity_id: EntityId) -> Option<&mut Entity> {
        if !self.is_alive(entity_id) {
            return None;
        }
        self.entities
            .get_mut(entity_id.index as usize)
            .and_then(|e| e.as_mut())
    }

    /// Deletes an entity by its ID.
    pub fn delete_entity(&mut self, entity_id: EntityId) {
        if !self.is_alive(entity_id) {
            return;
        }

        let idx = entity_id.index as usize;
        self.remove_entity_from_archetype(entity_id.index, self.get_component_set(idx));
        self.entities[idx] = None;
        info!("Deleted entity! ID: {}", entity_id.index);
        self.components.remove_entity(idx);
        if let Some(gen) = self.generations.get_mut(idx) {
            *gen = gen.wrapping_add(1);
        }
        self.id_queue.sorted_enqueue(entity_id.index);
        self.get_next_id();
    }

    fn create_archetype(&mut self, components: ComponentSet) {
        self.archetypes.create_archetype(components.clone());

        let mut matching_entities = Vec::new();
        for (entity_id, entity_option) in self.entities.iter().enumerate() {
            if let Some(_entity) = entity_option {
                let entity_component_set = self.get_component_set(entity_id);

                if components.is_subset(&entity_component_set) {
                    matching_entities.push(entity_id as u32);
                }
            }
        }

        for entity_id in matching_entities {
            self.add_entity_to_archetype(entity_id, components.clone());
        }
    }

    fn add_entity_to_archetype(&mut self, entity_id: u32, components: ComponentSet) {
        self.archetypes
            .add_entity_to_archetype(&components, entity_id);
    }

    fn remove_entity_from_archetype(&mut self, entity_id: u32, components: ComponentSet) {
        self.archetypes
            .remove_entity_from_archetype(&components, entity_id);
    }

    fn get_component_set(&self, entity_id: usize) -> ComponentSet {
        let components = match self.entities.get(entity_id) {
            Some(cmp) => match cmp.as_ref() {
                Some(e) => e.get_components().iter().collect::<Vec<usize>>(),
                None => {
                    error!("This entity ({}) does not have any components!", entity_id);
                    Vec::new()
                }
            },
            _ => {
                error!("This entity ({}) does not exist!", entity_id);
                Vec::new()
            }
        };

        let type_ids = components
            .iter()
            .map(|index| self.components.keys()[*index])
            .collect::<Vec<TypeId>>();
        ComponentSet::from_ids(type_ids)
    }

    /// Registers a new component in the scene.
    pub fn register_component<C: Component + 'static>(&mut self) {
        if !self.components.contains(&C::type_id()) {
            self.components.register_component::<C>(self.entities.len());
            self.create_archetype(ComponentSet::from_ids(vec![C::type_id()]));
            info!("Registered component: {}", C::type_name());
            return;
        }
        warn!("Component {} is already registered!", C::type_name());
    }

    /// Deregisters a component from the scene.
    pub fn deregister_component<C: Component + 'static>(&mut self) {
        if self.components.contains(&C::type_id()) {
            self.components.deregister_component::<C>();
            info!("Deregistered component: {}", C::type_name());
            return;
        }
        warn!("Component {} was not registered!", C::type_name());
    }

    /// Adds a component to an entity by its ID and an instance of the component.
    /// Overwrites the previous component if another component of the same type is added.
    pub fn add_component<C: Component + 'static>(&mut self, entity_id: EntityId, component: C) {
        if !self.is_alive(entity_id) {
            error!(
                "Attempted to add component {} to dead entity {}",
                C::type_name(),
                entity_id.index
            );
            return;
        }

        let old_component_set = self.get_component_set(entity_id.index as usize);
        if !old_component_set.to_vec().is_empty() {
            self.remove_entity_from_archetype(entity_id.index, old_component_set);
        }

        self.components
            .set_component(entity_id.index as usize, component);
        if let Some(component_index) = self
            .components
            .keys()
            .iter()
            .position(|x| *x == C::type_id())
        {
            if let Some(entity) = self.get_entity_mut(entity_id) {
                entity.add_component(component_index);
            } else {
                error!(
                    "Attempted to add component to non-existent entity {}",
                    entity_id.index
                );
            }
        } else {
            error!(
                "Component {} not registered, cannot add to entity {}",
                C::type_name(),
                entity_id.index
            );
        }

        let new_component_set = self.get_component_set(entity_id.index as usize);

        if !self.archetypes.contains_archetype(&new_component_set) {
            self.create_archetype(new_component_set.clone());
        }

        self.add_entity_to_archetype(entity_id.index, new_component_set);

        info!(
            "Added component {} to entity {}!",
            C::type_name(),
            entity_id.index
        );
    }

    pub fn remove_component<C: Component + 'static>(&mut self, entity_id: EntityId) {
        if !self.is_alive(entity_id) {
            return;
        }
        let old_component_set = self.get_component_set(entity_id.index as usize);
        self.remove_entity_from_archetype(entity_id.index, old_component_set);

        self.components
            .remove_component::<C>(entity_id.index as usize);
        if let Some(component_index) = self
            .components
            .keys()
            .iter()
            .position(|x| *x == C::type_id())
        {
            if let Some(entity) = self.get_entity_mut(entity_id) {
                entity.remove_component(component_index);
            }
        }

        let new_component_set = self.get_component_set(entity_id.index as usize);

        if !new_component_set.to_vec().is_empty() {
            if !self.archetypes.contains_archetype(&new_component_set) {
                self.create_archetype(new_component_set.clone());
            }

            self.add_entity_to_archetype(entity_id.index, new_component_set);
        }

        info!(
            "Removed component {} from entity {}!",
            C::type_name(),
            entity_id.index
        );
    }

    /// Returns a reference to a component of an entity by its ID.
    pub fn get_component<C: Component + 'static>(&self, entity_id: EntityId) -> Option<&C> {
        if !self.is_alive(entity_id) {
            return None;
        }
        self.components
            .get_component::<C>(entity_id.index as usize)
    }

    pub fn get_component_mut<C: Component + 'static>(
        &mut self,
        entity_id: EntityId,
    ) -> Option<&mut C> {
        if !self.is_alive(entity_id) {
            return None;
        }
        self.components
            .get_component_mut::<C>(entity_id.index as usize)
    }

    pub fn has<C: Component + 'static>(&self, entity_id: EntityId) -> bool {
        self.is_alive(entity_id)
            && self
                .components
                .get_component::<C>(entity_id.index as usize)
                .is_some()
    }

    /// Returns a list of entities that have the given components.
    pub fn get_entities_with(&self, components: Vec<TypeId>) -> Vec<EntityId> {
        let component_set = ComponentSet::from_ids(components);
        let mut result = Vec::new();

        for archetype_set in self.archetypes.component_sets() {
            if component_set.is_subset(archetype_set) {
                if let Some(entities) = self.archetypes.get_archetype(archetype_set) {
                    for index in entities.iter() {
                        if let Some(gen) = self.generations.get(*index as usize) {
                            if self
                                .entities
                                .get(*index as usize)
                                .is_some_and(|e| e.is_some())
                            {
                                result.push(EntityId {
                                    index: *index,
                                    gen: *gen,
                                });
                            }
                        }
                    }
                }
            }
        }

        result
    }

    /// Deletes all entities that have the given components.
    pub fn delete_entities_with(&mut self, components: Vec<TypeId>) {
        let entities = self.get_entities_with(components);
        for entity in entities {
            self.delete_entity(entity);
        }
    }

    /// Iterates over all entities that have the two given components and calls the given function.
    pub fn foreach<C: Component, K: Component>(&mut self, mut func: impl FnMut(&mut C, &mut K)) {
        if std::any::TypeId::of::<C>() == std::any::TypeId::of::<K>() {
            error!("foreach called with identical component types");
            return;
        }

        let required = ComponentSet::from_ids(vec![C::type_id(), K::type_id()]);
        let (c_set, k_set) = self
            .components
            .get_two_mut(&C::type_id(), &K::type_id());

        if let (Some(c_store), Some(k_store)) = (c_set, k_set) {
            for archetype_set in self.archetypes.component_sets() {
                if required.is_subset(archetype_set) {
                    if let Some(entities) = self.archetypes.get_archetype(archetype_set) {
                        for &idx in entities {
                            let idx = idx as usize;
                            if let (Some(c), Some(k)) =
                                (c_store.get_mut::<C>(idx), k_store.get_mut::<K>(idx))
                            {
                                func(c, k);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Registers a prefab with the given name and factory function.
    pub fn register_prefab(&mut self, name: &str, factory: crate::prefabs::PrefabFactory) {
        self.prefabs.register(name, factory);
    }

    /// Spawns a prefab with the given name.
    pub fn spawn_prefab(&mut self, name: &str) -> Option<EntityId> {
        if self.prefabs.has_prefab(name) {
            if let Some(factory) = self.prefabs.prefabs.get(&name.to_string()) {
                let factory = *factory; // Copy the function pointer
                Some(factory(self))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Checks if a prefab with the given name exists.
    pub fn has_prefab(&self, name: &str) -> bool {
        self.prefabs.has_prefab(name)
    }
}
