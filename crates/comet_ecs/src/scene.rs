use crate::archetypes::{Archetypes, ComponentInfo};
use crate::bundles::Bundle;
use crate::prefabs::{ErasedComponent, PrefabManager};
use crate::query_plan_cache::QueryPlanCache;
use crate::{Component, Entity, EntityLocation, IdQueue};
use comet_log::*;
use comet_structs::ComponentSet;
use std::alloc::Layout;
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ptr;

pub trait ComponentTuple {
    fn type_ids() -> Vec<TypeId>;
}

impl ComponentTuple for () {
    fn type_ids() -> Vec<TypeId> {
        Vec::new()
    }
}

macro_rules! impl_component_tuple {
    ($($name:ident),+ $(,)?) => {
        impl<$($name: Component),+> ComponentTuple for ($($name,)+) {
            fn type_ids() -> Vec<TypeId> {
                vec![$($name::type_id()),+]
            }
        }
    };
}

impl_component_tuple!(A);
impl_component_tuple!(A, B);
impl_component_tuple!(A, B, C);

pub struct Scene {
    id_queue: IdQueue,
    next_id: u32,
    generations: Vec<u32>,
    entities: Vec<Option<Entity>>,
    component_registry: Vec<Option<TypeId>>,
    component_index: HashMap<TypeId, usize>,
    component_info: HashMap<TypeId, ComponentInfo>,
    entity_locations: Vec<Option<EntityLocation>>,
    archetypes: Archetypes,
    archetype_version: usize,
    query_plan_cache: RefCell<QueryPlanCache>,
    prefabs: PrefabManager,
}

impl Scene {
    pub fn new() -> Self {
        let mut scene = Self {
            id_queue: IdQueue::new(),
            next_id: 0,
            generations: Vec::new(),
            entities: Vec::new(),
            component_registry: Vec::new(),
            component_index: HashMap::new(),
            component_info: HashMap::new(),
            entity_locations: Vec::new(),
            archetypes: Archetypes::new(),
            archetype_version: 0,
            query_plan_cache: RefCell::new(QueryPlanCache::default()),
            prefabs: PrefabManager::new(),
        };
        let empty_set = ComponentSet::new();
        let _ = scene.archetypes.get_or_create(
            empty_set,
            &scene.component_info,
            &scene.component_registry,
        );
        scene
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

    fn is_alive(&self, id: Entity) -> bool {
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
    pub fn new_entity(&mut self) -> Entity {
        let index = self.next_id;
        let gen = if (index as usize) >= self.generations.len() {
            self.generations.push(0);
            0
        } else {
            self.generations[index as usize]
        };
        let id = Entity { index, gen };

        if (index as usize) >= self.entities.len() {
            self.entities.push(Some(id));
        } else {
            self.entities[index as usize] = Some(id);
        }

        let empty_set = ComponentSet::new();
        let empty_arch = self.archetypes.get_or_create(
            empty_set,
            &self.component_info,
            &self.component_registry,
        );
        let row = self.archetypes.get_mut(empty_arch).push_entity(id);
        if index as usize >= self.entity_locations.len() {
            self.entity_locations.resize(index as usize + 1, None);
        }
        self.entity_locations[index as usize] = Some(EntityLocation {
            archetype: empty_arch,
            row,
            gen,
        });
        self.get_next_id();
        info!("Created entity! ID: {} (gen {})", id.index, id.gen);
        id
    }

    /// Gets an immutable reference to an entity by its ID.
    pub fn get_entity(&self, entity_id: Entity) -> Option<&Entity> {
        if !self.is_alive(entity_id) {
            return None;
        }
        self.entities
            .get(entity_id.index as usize)
            .and_then(|e| e.as_ref())
    }

    /// Gets a mutable reference to an entity by its ID.
    pub fn get_entity_mut(&mut self, entity_id: Entity) -> Option<&mut Entity> {
        if !self.is_alive(entity_id) {
            return None;
        }
        self.entities
            .get_mut(entity_id.index as usize)
            .and_then(|e| e.as_mut())
    }

    /// Deletes an entity by its ID.
    pub fn delete_entity(&mut self, entity_id: Entity) {
        if !self.is_alive(entity_id) {
            return;
        }

        let idx = entity_id.index as usize;
        if let Some(loc) = self.entity_locations.get(idx).and_then(|l| *l) {
            let last_row = self.archetypes.get(loc.archetype).len().saturating_sub(1);
            if loc.row != last_row {
                let swapped_entity = {
                    let arch = self.archetypes.get_mut(loc.archetype);
                    arch.swap_rows(loc.row, last_row);
                    arch.entities()[loc.row]
                };
                let swapped_idx = swapped_entity.index as usize;
                if let Some(entry) = self.entity_locations.get_mut(swapped_idx) {
                    *entry = Some(EntityLocation {
                        archetype: loc.archetype,
                        row: loc.row,
                        gen: swapped_entity.gen,
                    });
                }
            }

            let arch = self.archetypes.get_mut(loc.archetype);
            for col in arch.columns_mut() {
                let _ = col.drop_last();
            }
            arch.pop_entity();
        }

        self.entities[idx] = None;
        if idx < self.entity_locations.len() {
            self.entity_locations[idx] = None;
        }
        info!("Deleted entity! ID: {}", idx);
        if let Some(gen) = self.generations.get_mut(idx) {
            *gen = gen.wrapping_add(1);
        }
        self.id_queue.sorted_enqueue(entity_id.index);
        self.get_next_id();
    }

    fn get_location(&self, entity_id: Entity) -> Option<EntityLocation> {
        self.entity_locations
            .get(entity_id.index as usize)
            .and_then(|l| *l)
            .filter(|l| l.gen == entity_id.gen)
    }

    fn set_location(&mut self, entity_id: Entity, archetype: usize, row: usize) {
        let index = entity_id.index as usize;
        if index >= self.entity_locations.len() {
            self.entity_locations.resize(index + 1, None);
        }
        self.entity_locations[index] = Some(EntityLocation {
            archetype,
            row,
            gen: entity_id.gen,
        });
    }

    fn get_two_archetypes_mut(
        &mut self,
        a: usize,
        b: usize,
    ) -> (
        &mut crate::archetypes::Archetype,
        &mut crate::archetypes::Archetype,
    ) {
        self.archetypes.get_two_mut(a, b)
    }

    fn ensure_archetype(&mut self, set: ComponentSet) -> usize {
        let before = self.archetypes.len();
        let id = self
            .archetypes
            .get_or_create(set, &self.component_info, &self.component_registry);
        self.bump_archetype_version_if_changed(before);
        id
    }

    fn bump_archetype_version_if_changed(&mut self, before: usize) {
        if self.archetypes.len() != before {
            self.archetype_version = self.archetype_version.wrapping_add(1);
        }
    }

    fn normalized_tags(tags: &[TypeId]) -> Vec<TypeId> {
        let mut normalized = Vec::with_capacity(tags.len());
        for tag in tags {
            if !normalized.contains(tag) {
                normalized.push(*tag);
            }
        }
        normalized
    }

    fn take_last_component_of_type(
        components: &mut Vec<ErasedComponent>,
        type_id: TypeId,
    ) -> Option<ErasedComponent> {
        let idx = components
            .iter()
            .rposition(|component| component.type_id == type_id)?;
        Some(components.swap_remove(idx))
    }

    pub(crate) fn cached_single_plan(
        &self,
        component: TypeId,
        tags: &[TypeId],
    ) -> Vec<(usize, usize)> {
        let normalized_tags = Self::normalized_tags(tags);

        {
            let mut cache = self.query_plan_cache.borrow_mut();
            cache.sync_version(self.archetype_version);
            if let Some(matches) = cache.get_single_cloned(component, &normalized_tags) {
                return matches;
            }
        }

        let mut matches = Vec::new();
        for (arch_id, arch) in self.archetypes.iter().enumerate() {
            if let Some(col_idx) = arch.column_index(component) {
                if normalized_tags
                    .iter()
                    .all(|t| arch.column_index(*t).is_some())
                {
                    matches.push((arch_id, col_idx));
                }
            }
        }

        let mut cache = self.query_plan_cache.borrow_mut();
        cache.sync_version(self.archetype_version);
        cache.insert_single(component, &normalized_tags, matches.clone());
        matches
    }

    pub(crate) fn cached_pair_plan(
        &self,
        a: TypeId,
        b: TypeId,
        tags: &[TypeId],
    ) -> Vec<(usize, usize, usize)> {
        let normalized_tags = Self::normalized_tags(tags);

        {
            let mut cache = self.query_plan_cache.borrow_mut();
            cache.sync_version(self.archetype_version);
            if let Some(matches) = cache.get_pair_cloned(a, b, &normalized_tags) {
                return matches;
            }
        }

        let mut matches = Vec::new();
        for (arch_id, arch) in self.archetypes.iter().enumerate() {
            if let (Some(a_idx), Some(b_idx)) = (arch.column_index(a), arch.column_index(b)) {
                if normalized_tags
                    .iter()
                    .all(|t| arch.column_index(*t).is_some())
                {
                    matches.push((arch_id, a_idx, b_idx));
                }
            }
        }

        let mut cache = self.query_plan_cache.borrow_mut();
        cache.sync_version(self.archetype_version);
        cache.insert_pair(a, b, &normalized_tags, matches.clone());
        matches
    }

    fn has_live_component_instances(&self, type_id: TypeId) -> bool {
        self.archetypes
            .iter()
            .any(|arch| arch.column_index(type_id).is_some() && !arch.is_empty())
    }

    /// Registers a new component in the scene.
    pub fn register_component<C: Component + 'static>(&mut self) {
        let type_id = C::type_id();
        if self.component_info.contains_key(&type_id) {
            warn!("Component {} is already registered!", C::type_name());
            return;
        }

        let drop_fn: unsafe fn(*mut u8) = |ptr| unsafe { ptr::drop_in_place(ptr as *mut C) };
        let info = ComponentInfo {
            type_id,
            layout: Layout::new::<C>(),
            drop_fn,
        };
        self.component_info.insert(type_id, info);

        if !self.component_index.contains_key(&type_id) {
            let index = if let Some((i, _)) = self
                .component_registry
                .iter()
                .enumerate()
                .find(|(_, v)| v.is_none())
            {
                self.component_registry[i] = Some(type_id);
                i
            } else {
                self.component_registry.push(Some(type_id));
                self.component_registry.len() - 1
            };
            self.component_index.insert(type_id, index);
        }

        info!("Registered component: {}", C::type_name());
    }

    /// Deregisters a component from the scene.
    pub fn deregister_component<C: Component + 'static>(&mut self) {
        let type_id = C::type_id();
        if !self.component_info.contains_key(&type_id) {
            warn!("Component {} was not registered!", C::type_name());
            return;
        }

        if self.has_live_component_instances(type_id) {
            error!(
                "Cannot deregister component {} while live entities still contain it",
                C::type_name()
            );
            return;
        }

        if let Some(index) = self.component_index.remove(&type_id) {
            if let Some(slot) = self.component_registry.get_mut(index) {
                *slot = None;
            }
        }

        if self.component_info.remove(&type_id).is_some() {
            info!("Deregistered component: {}", C::type_name());
        } else {
            warn!("Component {} was not registered!", C::type_name());
        }
    }

    fn validate_components_registered(&self, components: &[ErasedComponent]) -> bool {
        for component in components {
            if !self.component_info.contains_key(&component.type_id) {
                error!(
                    "Component TypeId {:?} not registered, cannot add bundle/components",
                    component.type_id
                );
                return false;
            }
        }
        true
    }

    /// Adds a component to an entity by its ID and an instance of the component.
    /// Overwrites the previous component if another component of the same type is added.
    pub fn add_component<C: Component + 'static>(&mut self, entity_id: Entity, component: C) {
        if !self.is_alive(entity_id) {
            error!(
                "Attempted to add component {} to dead entity {}",
                C::type_name(),
                entity_id.index
            );
            return;
        }

        let type_id = C::type_id();
        if !self.component_info.contains_key(&type_id) {
            error!(
                "Component {} not registered, cannot add to entity {}",
                C::type_name(),
                entity_id.index
            );
            return;
        }

        let loc = match self.get_location(entity_id) {
            Some(loc) => loc,
            None => return,
        };
        let old_arch_id = loc.archetype;

        if self
            .archetypes
            .get(old_arch_id)
            .column_index(type_id)
            .is_some()
        {
            let arch = self.archetypes.get_mut(old_arch_id);
            if let Some(col_idx) = arch.column_index(type_id) {
                let _ = arch.columns_mut()[col_idx].set::<C>(loc.row, component);
            }
            return;
        }

        let before = self.archetypes.len();
        let new_arch_id = self.archetypes.get_or_create_add_edge(
            old_arch_id,
            type_id,
            &self.component_info,
            &self.component_index,
            &self.component_registry,
        );
        self.bump_archetype_version_if_changed(before);

        let old_len = self.archetypes.get(old_arch_id).len();
        if old_len == 0 {
            return;
        }

        if loc.row != old_len - 1 {
            let swapped = {
                let arch = self.archetypes.get_mut(old_arch_id);
                arch.swap_rows(loc.row, old_len - 1);
                arch.entities()[loc.row]
            };
            self.set_location(swapped, old_arch_id, loc.row);
        }

        let (old_arch, new_arch) = self.get_two_archetypes_mut(old_arch_id, new_arch_id);
        let new_row = new_arch.push_entity(entity_id);

        if let Some(new_idx) = new_arch.column_index(type_id) {
            new_arch.columns_mut()[new_idx].push::<C>(component);
        }

        for new_idx in 0..new_arch.types().len() {
            let t = new_arch.types()[new_idx];
            if t == type_id {
                continue;
            }
            if let Some(old_idx) = old_arch.column_index(t) {
                let _ = old_arch.columns_mut()[old_idx]
                    .move_last_to(&mut new_arch.columns_mut()[new_idx]);
            }
        }

        for old_idx in 0..old_arch.types().len() {
            let t = old_arch.types()[old_idx];
            if new_arch.column_index(t).is_none() {
                let _ = old_arch.columns_mut()[old_idx].drop_last();
            }
        }

        old_arch.pop_entity();
        self.set_location(entity_id, new_arch_id, new_row);

        info!(
            "Added component {} to entity {}!",
            C::type_name(),
            entity_id.index
        );
    }

    pub fn remove_component<C: Component + 'static>(&mut self, entity_id: Entity) {
        if !self.is_alive(entity_id) {
            return;
        }
        let type_id = C::type_id();
        let loc = match self.get_location(entity_id) {
            Some(loc) => loc,
            None => return,
        };
        let old_arch_id = loc.archetype;
        if self
            .archetypes
            .get(old_arch_id)
            .column_index(type_id)
            .is_none()
        {
            return;
        }

        let before = self.archetypes.len();
        let new_arch_id = self.archetypes.get_or_create_remove_edge(
            old_arch_id,
            type_id,
            &self.component_info,
            &self.component_index,
            &self.component_registry,
        );
        self.bump_archetype_version_if_changed(before);
        let old_len = self.archetypes.get(old_arch_id).len();
        if old_len == 0 {
            return;
        }

        if loc.row != old_len - 1 {
            let swapped = {
                let arch = self.archetypes.get_mut(old_arch_id);
                arch.swap_rows(loc.row, old_len - 1);
                arch.entities()[loc.row]
            };
            self.set_location(swapped, old_arch_id, loc.row);
        }

        let (old_arch, new_arch) = self.get_two_archetypes_mut(old_arch_id, new_arch_id);
        let new_row = new_arch.push_entity(entity_id);

        for new_idx in 0..new_arch.types().len() {
            let t = new_arch.types()[new_idx];
            if let Some(old_idx) = old_arch.column_index(t) {
                let _ = old_arch.columns_mut()[old_idx]
                    .move_last_to(&mut new_arch.columns_mut()[new_idx]);
            }
        }

        for old_idx in 0..old_arch.types().len() {
            let t = old_arch.types()[old_idx];
            if new_arch.column_index(t).is_none() {
                let _ = old_arch.columns_mut()[old_idx].drop_last();
            }
        }

        old_arch.pop_entity();
        self.set_location(entity_id, new_arch_id, new_row);

        info!(
            "Removed component {} from entity {}!",
            C::type_name(),
            entity_id.index
        );
    }

    /// Returns a reference to a component of an entity by its ID.
    pub fn get_component<C: Component + 'static>(&self, entity_id: Entity) -> Option<&C> {
        if !self.is_alive(entity_id) {
            return None;
        }
        let loc = self.get_location(entity_id)?;
        let arch = self.archetypes.get(loc.archetype);
        let col_idx = arch.column_index(C::type_id())?;
        arch.columns().get(col_idx)?.get::<C>(loc.row)
    }

    pub fn get_component_mut<C: Component + 'static>(
        &mut self,
        entity_id: Entity,
    ) -> Option<&mut C> {
        if !self.is_alive(entity_id) {
            return None;
        }
        let loc = self.get_location(entity_id)?;
        let arch = self.archetypes.get_mut(loc.archetype);
        let col_idx = arch.column_index(C::type_id())?;
        arch.columns_mut().get_mut(col_idx)?.get_mut::<C>(loc.row)
    }

    pub fn has<C: Component + 'static>(&self, entity_id: Entity) -> bool {
        self.is_alive(entity_id) && self.get_component::<C>(entity_id).is_some()
    }

    fn component_indices_from_type_ids(&self, components: &[TypeId]) -> Option<Vec<usize>> {
        let mut indices = Vec::with_capacity(components.len());
        for type_id in components {
            let index = self.component_index.get(type_id).copied()?;
            indices.push(index);
        }
        Some(indices)
    }

    /// Returns a list of entities that have the given component indices.
    fn get_entities_with_indices(&self, components: &[usize]) -> Vec<Entity> {
        let component_set = ComponentSet::from_indices(components.to_vec());
        let mut result = Vec::new();

        for arch in self.archetypes.iter() {
            if component_set.is_subset(arch.set()) {
                for entity in arch.entities() {
                    if self.is_alive(*entity) {
                        result.push(*entity);
                    }
                }
            }
        }

        result
    }

    /// Deletes all entities that have the given component indices.
    fn delete_entities_with_indices(&mut self, components: &[usize]) {
        let entities = self.get_entities_with_indices(components);
        for entity in entities {
            self.delete_entity(entity);
        }
    }

    /// Deletes all entities that have the given components.
    pub fn delete_entities_with(&mut self, components: Vec<TypeId>) {
        let Some(indices) = self.component_indices_from_type_ids(&components) else {
            return;
        };
        self.delete_entities_with_indices(&indices);
    }

    /// Deletes all entities that have the component tuple.
    pub fn delete_entities_with_types<Cs: ComponentTuple>(&mut self) {
        self.delete_entities_with(Cs::type_ids());
    }

    /// Iterates over all entities that have the two given components and calls the given function.
    pub fn foreach<C: Component, K: Component>(&mut self, mut func: impl FnMut(&mut C, &mut K)) {
        if C::type_id() == K::type_id() {
            error!("foreach called with identical component types");
            return;
        }

        let c_index = match self.component_index.get(&C::type_id()).copied() {
            Some(index) => index,
            None => return,
        };
        let k_index = match self.component_index.get(&K::type_id()).copied() {
            Some(index) => index,
            None => return,
        };
        let required = ComponentSet::from_indices(vec![c_index, k_index]);
        for arch in self.archetypes.iter_mut() {
            if required.is_subset(arch.set()) {
                let c_idx = match arch.column_index(C::type_id()) {
                    Some(idx) => idx,
                    None => continue,
                };
                let k_idx = match arch.column_index(K::type_id()) {
                    Some(idx) => idx,
                    None => continue,
                };

                let len = arch.len();
                if c_idx < k_idx {
                    let (left, right) = arch.columns_mut().split_at_mut(k_idx);
                    let c_col = &mut left[c_idx];
                    let k_col = &mut right[0];
                    for row in 0..len {
                        if let (Some(c), Some(k)) =
                            (c_col.get_mut::<C>(row), k_col.get_mut::<K>(row))
                        {
                            func(c, k);
                        }
                    }
                } else {
                    let (left, right) = arch.columns_mut().split_at_mut(c_idx);
                    let k_col = &mut left[k_idx];
                    let c_col = &mut right[0];
                    for row in 0..len {
                        if let (Some(c), Some(k)) =
                            (c_col.get_mut::<C>(row), k_col.get_mut::<K>(row))
                        {
                            func(c, k);
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
    pub fn spawn_prefab(&mut self, name: &str) -> Option<Entity> {
        if self.prefabs.has_prefab(name) {
            if let Some(factory) = self.prefabs.prefabs.get(&name.to_string()) {
                let factory = *factory;
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

    pub(crate) fn archetypes(&self) -> &crate::archetypes::Archetypes {
        &self.archetypes
    }

    pub(crate) fn archetypes_mut(&mut self) -> &mut crate::archetypes::Archetypes {
        &mut self.archetypes
    }

    pub fn spawn_bundle<B: Bundle>(&mut self, bundle: B) -> Entity {
        self.spawn_with_components(bundle.into_components())
    }

    pub fn add_bundle<B: Bundle>(&mut self, entity: Entity, bundle: B) {
        self.add_with_components(entity, bundle.into_components());
    }

    pub fn add_with_components(&mut self, entity_id: Entity, mut components: Vec<ErasedComponent>) {
        if !self.is_alive(entity_id) || components.is_empty() {
            return;
        }
        if !self.validate_components_registered(&components) {
            return;
        }

        let loc = match self.get_location(entity_id) {
            Some(loc) => loc,
            None => return,
        };
        let old_arch_id = loc.archetype;
        let old_set = self.archetypes.get(old_arch_id).set().clone();
        let mut component_set = old_set.clone();
        for component in &components {
            let index = self
                .component_index
                .get(&component.type_id)
                .copied()
                .unwrap_or_else(|| panic!("Component {:?} missing index", component.type_id));
            component_set.insert(index);
        }
        let new_arch_id = self.ensure_archetype(component_set);
        let old_len = self.archetypes.get(old_arch_id).len();
        if old_len == 0 {
            return;
        }

        if old_arch_id == new_arch_id {
            let arch = self.archetypes.get_mut(old_arch_id);
            for component in components.drain(..) {
                if let Some(col_idx) = arch.column_index(component.type_id) {
                    (component.set_fn)(component.value, &mut arch.columns_mut()[col_idx], loc.row);
                }
            }
        } else {
            if loc.row != old_len - 1 {
                let swapped = {
                    let arch = self.archetypes.get_mut(old_arch_id);
                    arch.swap_rows(loc.row, old_len - 1);
                    arch.entities()[loc.row]
                };
                self.set_location(swapped, old_arch_id, loc.row);
            }

            let (old_arch, new_arch) = self.get_two_archetypes_mut(old_arch_id, new_arch_id);
            let new_row = new_arch.push_entity(entity_id);

            for new_idx in 0..new_arch.types().len() {
                let t = new_arch.types()[new_idx];
                if let Some(old_idx) = old_arch.column_index(t) {
                    let _ = old_arch.columns_mut()[old_idx]
                        .move_last_to(&mut new_arch.columns_mut()[new_idx]);
                    if let Some(component) = Self::take_last_component_of_type(&mut components, t) {
                        (component.set_fn)(
                            component.value,
                            &mut new_arch.columns_mut()[new_idx],
                            new_row,
                        );
                    }
                    continue;
                }

                let component = Self::take_last_component_of_type(&mut components, t)
                    .unwrap_or_else(|| panic!("Bundle missing component {:?}", t));
                (component.push_fn)(component.value, &mut new_arch.columns_mut()[new_idx]);
            }

            for old_idx in 0..old_arch.types().len() {
                let t = old_arch.types()[old_idx];
                if new_arch.column_index(t).is_none() {
                    let _ = old_arch.columns_mut()[old_idx].drop_last();
                }
            }

            old_arch.pop_entity();
            self.set_location(entity_id, new_arch_id, new_row);
        }
    }

    pub fn spawn_with_components(&mut self, components: Vec<ErasedComponent>) -> Entity {
        let entity_id = self.new_entity();
        if components.is_empty() {
            return entity_id;
        }
        self.add_with_components(entity_id, components);
        entity_id
    }
}

#[cfg(test)]
mod tests {
    use super::Scene;
    use crate::{Component, ErasedComponent};

    struct A;
    impl Component for A {}

    struct Unregistered;
    impl Component for Unregistered {}

    #[test]
    fn deregister_component_is_blocked_while_live_instances_exist() {
        let mut scene = Scene::new();
        scene.register_component::<A>();

        let e1 = scene.new_entity();
        scene.add_component(e1, A);

        scene.deregister_component::<A>();

        let e2 = scene.new_entity();
        scene.add_component(e2, A);

        assert!(scene.get_component::<A>(e1).is_some());
        assert!(scene.get_component::<A>(e2).is_some());
    }

    #[test]
    fn add_with_components_ignores_unregistered_components_without_panicking() {
        let mut scene = Scene::new();

        let entity = scene.new_entity();
        scene.add_with_components(entity, vec![ErasedComponent::new(Unregistered)]);

        assert!(scene.get_component::<Unregistered>(entity).is_none());
    }
}
