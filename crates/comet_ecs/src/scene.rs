use crate::archetypes::{Archetypes, ComponentInfo};
use crate::bundles::Bundle;
use crate::prefabs::{ErasedComponent, PrefabManager};
use crate::query_plan_cache::QueryPlanCache;
use crate::scene_commands::{SceneCommand, SceneCommands};
use crate::scene_internals::{BundleSpawnPlan, ComponentChangeState};
use crate::{
    Component, ComponentTuple, Entity, EntityLocation, IdQueue, Tick,
};
use comet_log::*;
use comet_structs::{Column, ComponentSet};
use std::alloc::Layout;
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ptr;
use std::slice;
use std::sync::Arc;

const DEFAULT_ENTITY_STORAGE_CAPACITY: usize = 256;

pub struct Scene {
    component_event_tick: Tick,
    default_query_since_tick: Tick,
    id_queue: IdQueue,
    next_id: u32,
    generations: Vec<u32>,
    entities: Vec<Option<Entity>>,
    active_entities: u32,
    component_registry: Vec<Option<TypeId>>,
    component_index: HashMap<TypeId, usize>,
    component_info: HashMap<TypeId, ComponentInfo>,
    entity_locations: Vec<Option<EntityLocation>>,
    archetypes: Archetypes,
    archetype_version: usize,
    query_plan_cache: RefCell<QueryPlanCache>,
    bundle_spawn_cache: HashMap<TypeId, BundleSpawnPlan>,
    component_change_state: HashMap<(u32, TypeId), ComponentChangeState>,
    removed_component_events: HashMap<TypeId, Vec<(Entity, Tick)>>,
    prefabs: PrefabManager,
    commands: SceneCommands,
}

impl Scene {
    /// Creates a new empty scene.
    pub fn new() -> Self {
        let mut scene = Self {
            component_event_tick: 0,
            default_query_since_tick: 0,
            id_queue: IdQueue::new(),
            next_id: 0,
            generations: Vec::with_capacity(DEFAULT_ENTITY_STORAGE_CAPACITY),
            entities: Vec::with_capacity(DEFAULT_ENTITY_STORAGE_CAPACITY),
            active_entities: 0,
            component_registry: Vec::new(),
            component_index: HashMap::new(),
            component_info: HashMap::new(),
            entity_locations: Vec::with_capacity(DEFAULT_ENTITY_STORAGE_CAPACITY),
            archetypes: Archetypes::new(),
            archetype_version: 0,
            query_plan_cache: RefCell::new(QueryPlanCache::default()),
            bundle_spawn_cache: HashMap::new(),
            component_change_state: HashMap::new(),
            removed_component_events: HashMap::new(),
            prefabs: PrefabManager::new(),
            commands: SceneCommands::new(),
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
        self.active_entities
    }

    /// Returns the tick used to stamp component add/change/remove events.
    pub fn component_event_tick(&self) -> Tick {
        self.component_event_tick
    }

    /// Sets the current logical tick used to stamp component add/change/remove events.
    pub fn set_component_event_tick(&mut self, tick: Tick) {
        self.component_event_tick = tick;
    }

    /// Returns the default baseline used by temporal query filters like `Added<T>` and `Changed<T>`.
    pub fn default_query_since_tick(&self) -> Tick {
        self.default_query_since_tick
    }

    /// Sets the default baseline used by temporal query filters like `Added<T>` and `Changed<T>`.
    pub fn set_default_query_since_tick(&mut self, tick: Tick) {
        self.default_query_since_tick = tick;
    }

    /// Advances the component-event tick by one and returns it.
    pub fn advance_component_event_tick(&mut self) -> Tick {
        self.component_event_tick = self.component_event_tick.wrapping_add(1);
        self.component_event_tick
    }

    /// Queues spawning an empty entity.
    /// The command executes on [`Scene::apply_commands`] or at app tick-end.
    pub fn deferred_spawn_empty(&mut self) {
        self.commands.spawn_empty();
    }

    /// Queues spawning an entity with the given components.
    /// The command executes on [`Scene::apply_commands`] or at app tick-end.
    pub fn deferred_spawn<B: Bundle>(&mut self, bundle: B) {
        self.commands.spawn(bundle.into_components());
    }

    /// Queues spawning a batch of entities with the given components.
    /// The command executes on [`Scene::apply_commands`] or at app tick-end.
    pub fn deferred_spawn_batch<B: Bundle>(&mut self, batch: Vec<B>) {
        let entities = batch
            .into_iter()
            .map(|bundle| bundle.into_components())
            .collect();
        self.commands.spawn_batch(entities);
    }

    /// Queues deleting an entity.
    pub fn deferred_delete_entity(&mut self, entity: Entity) {
        self.commands.delete_entity(entity);
    }

    /// Queues component registration.
    pub fn deferred_register_component<C: Component + 'static>(&mut self) {
        self.commands.register_component::<C>();
    }

    /// Queues registration of a tuple of component types.
    pub fn deferred_register_components<T: ComponentTuple>(&mut self) {
        T::deferred_register_all(&mut self.commands);
    }

    /// Queues component deregistration.
    pub fn deferred_deregister_component<C: Component + 'static>(&mut self) {
        self.commands.deregister_component::<C>();
    }

    /// Queues adding or setting a component on an entity.
    pub fn deferred_add_component<C: Component + 'static>(&mut self, entity: Entity, component: C) {
        self.commands.add_component(entity, component);
    }

    /// Queues adding or setting multiple components on an entity.
    pub fn deferred_add_components<B: Bundle>(
        &mut self,
        entity: Entity,
        bundle: B,
    ) {
        self.commands
            .add_components(entity, bundle.into_components());
    }

    /// Queues removing a component from an entity.
    pub fn deferred_remove_component<C: Component + 'static>(&mut self, entity: Entity) {
        self.commands.remove_component::<C>(entity);
    }

    /// Queues removing multiple components from an entity.
    pub fn deferred_remove_components<T: ComponentTuple>(&mut self, entity: Entity) {
        self.commands.remove_components(entity, T::type_ids());
    }

    /// Queues deleting all entities that contain the given component set.
    pub fn deferred_delete_entities_with(&mut self, components: Vec<TypeId>) {
        self.commands
            .push(SceneCommand::DeleteEntitiesWith(components));
    }

    /// Queues prefab registration.
    pub fn deferred_register_prefab(
        &mut self,
        name: impl Into<String>,
        factory: crate::prefabs::PrefabFactory,
    ) {
        self.commands.register_prefab(name, factory);
    }

    /// Queues prefab spawning by name.
    pub fn deferred_spawn_prefab(&mut self, name: impl Into<String>) {
        self.commands.spawn_prefab(name);
    }

    /// Returns the amount of currently queued deferred commands.
    pub fn queued_command_count(&self) -> usize {
        self.commands.len()
    }

    /// Applies all queued deferred commands in FIFO order.
    pub fn apply_commands(&mut self) {
        let mut commands = std::mem::take(&mut self.commands);
        commands.apply(self);
        self.commands = commands;
    }

    /// Applies a single command immediately.
    pub fn apply_command(&mut self, command: SceneCommand) {
        SceneCommands::apply_command(self, command);
    }

    #[inline(always)]
    fn mark_component_added_and_changed(&mut self, entity: Entity, type_id: TypeId) {
        self.component_change_state.insert(
            (entity.index, type_id),
            ComponentChangeState {
                added_tick: self.component_event_tick,
                changed_tick: self.component_event_tick,
            },
        );
    }

    #[inline(always)]
    fn mark_component_changed(&mut self, entity: Entity, type_id: TypeId) {
        let key = (entity.index, type_id);
        if let Some(state) = self.component_change_state.get_mut(&key) {
            state.changed_tick = self.component_event_tick;
            return;
        }
        self.component_change_state.insert(
            key,
            ComponentChangeState {
                added_tick: self.component_event_tick,
                changed_tick: self.component_event_tick,
            },
        );
    }

    pub(crate) fn mark_component_changed_for_query(&mut self, entity: Entity, type_id: TypeId) {
        self.mark_component_changed(entity, type_id);
    }

    #[inline(always)]
    fn mark_component_removed(&mut self, entity: Entity, type_id: TypeId) {
        self.component_change_state.remove(&(entity.index, type_id));
        self.removed_component_events
            .entry(type_id)
            .or_default()
            .push((entity, self.component_event_tick));
    }

    #[inline(always)]
    fn tick_is_newer_than(tick: Tick, last_seen_tick: Tick) -> bool {
        tick != last_seen_tick && tick.wrapping_sub(last_seen_tick) <= (u32::MAX / 2)
    }

    /// Returns whether a specific component has been added to the entity since the given tick
    pub fn component_added_since<C: Component + 'static>(
        &self,
        entity: Entity,
        last_seen_tick: Tick,
    ) -> bool {
        self.component_added_since_type(entity, C::type_id(), last_seen_tick)
    }

    pub(crate) fn component_added_since_type(
        &self,
        entity: Entity,
        type_id: TypeId,
        last_seen_tick: Tick,
    ) -> bool {
        self.component_change_state
            .get(&(entity.index, type_id))
            .is_some_and(|state| Self::tick_is_newer_than(state.added_tick, last_seen_tick))
    }

    /// Returns whether a specific component of an entity has been changed since the given tick
    pub fn component_changed_since<C: Component + 'static>(
        &self,
        entity: Entity,
        last_seen_tick: Tick,
    ) -> bool {
        self.component_changed_since_type(entity, C::type_id(), last_seen_tick)
    }

    pub(crate) fn component_changed_since_type(
        &self,
        entity: Entity,
        type_id: TypeId,
        last_seen_tick: Tick,
    ) -> bool {
        self.component_change_state
            .get(&(entity.index, type_id))
            .is_some_and(|state| Self::tick_is_newer_than(state.changed_tick, last_seen_tick))
    }

    /// Returns entities where `C` was removed since the given tick.
    pub fn removed_since<C: Component + 'static>(&self, last_seen_tick: Tick) -> Vec<Entity> {
        self.removed_component_events
            .get(&C::type_id())
            .map(|events| {
                events
                    .iter()
                    .filter_map(|(entity, tick)| {
                        Self::tick_is_newer_than(*tick, last_seen_tick).then_some(*entity)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    #[inline(always)]
    fn get_next_id(&mut self) {
        self.next_id = self
            .id_queue
            .dequeue()
            .unwrap_or_else(|| self.entities.len() as u32);
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

    /// Returns sparse entity storage indexed by entity index.
    pub fn entities(&self) -> &Vec<Option<Entity>> {
        &self.entities
    }

    #[inline(always)]
    fn allocate_entity_slot(&mut self) -> Entity {
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

        self.active_entities += 1;
        self.get_next_id();
        id
    }

    #[inline(always)]
    fn place_entity_in_archetype(&mut self, entity: Entity, archetype: usize) -> usize {
        let row = self.archetypes.get_mut(archetype).push_entity(entity);
        self.set_location(entity, archetype, row);
        row
    }

    /// Creates a new entity and returns its ID.
    pub fn new_entity(&mut self) -> Entity {
        self.new_entity_immediate()
    }

    pub(crate) fn new_entity_immediate(&mut self) -> Entity {
        let id = self.allocate_entity_slot();
        let empty_set = ComponentSet::new();
        let empty_arch = self.archetypes.get_or_create(
            empty_set,
            &self.component_info,
            &self.component_registry,
        );
        let _ = self.place_entity_in_archetype(id, empty_arch);
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

    /// Deletes an entity by its ID.
    pub fn delete_entity(&mut self, entity_id: Entity) {
        self.delete_entity_immediate(entity_id);
    }

    pub(crate) fn delete_entity_immediate(&mut self, entity_id: Entity) {
        if !self.is_alive(entity_id) {
            return;
        }

        let idx = entity_id.index as usize;
        if let Some(loc) = self.entity_locations.get(idx).and_then(|l| *l) {
            let removed_types = self.archetypes.get(loc.archetype).types().to_vec();
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

            for type_id in removed_types {
                self.mark_component_removed(entity_id, type_id);
            }
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
        self.active_entities -= 1;
    }

    fn get_location(&self, entity_id: Entity) -> Option<EntityLocation> {
        self.entity_locations
            .get(entity_id.index as usize)
            .and_then(|l| *l)
            .filter(|l| l.gen == entity_id.gen)
    }

    #[inline(always)]
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

    fn normalized_components(with_components: &[TypeId]) -> Vec<TypeId> {
        let mut normalized = with_components.to_vec();
        normalized.sort_unstable();
        normalized.dedup();
        normalized
    }

    fn normalized_component_filters(
        with_components: &[TypeId],
        without_components: &[TypeId],
        with_any_components: &[TypeId],
        without_any_components: &[TypeId],
    ) -> Option<(Vec<TypeId>, Vec<TypeId>, Vec<TypeId>, Vec<TypeId>)> {
        let with_components = Self::normalized_components(with_components);
        let without_components = Self::normalized_components(without_components);
        let with_any_components = Self::normalized_components(with_any_components);
        let without_any_components = Self::normalized_components(without_any_components);

        let mut include_components = with_components.clone();
        include_components.extend(with_any_components.iter().copied());
        include_components.sort_unstable();
        include_components.dedup();

        let mut exclude_components = without_components.clone();
        exclude_components.extend(without_any_components.iter().copied());
        exclude_components.sort_unstable();
        exclude_components.dedup();

        if include_components
            .iter()
            .any(|component_type| exclude_components.binary_search(component_type).is_ok())
        {
            return None;
        }
        Some((
            with_components,
            without_components,
            with_any_components,
            without_any_components,
        ))
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
        with_components: &[TypeId],
        without_components: &[TypeId],
        with_any_components: &[TypeId],
        without_any_components: &[TypeId],
    ) -> Vec<(usize, usize)> {
        let Some((
            with_components,
            without_components,
            with_any_components,
            without_any_components,
        )) = Self::normalized_component_filters(
            with_components,
            without_components,
            with_any_components,
            without_any_components,
        )
        else {
            return Vec::new();
        };

        {
            let mut cache = self.query_plan_cache.borrow_mut();
            cache.sync_version(self.archetype_version);
            if let Some(matches) = cache.get_single_cloned(
                component,
                &with_components,
                &without_components,
                &with_any_components,
                &without_any_components,
            ) {
                return matches;
            }
        }

        let mut matches = Vec::new();
        for (arch_id, arch) in self.archetypes.iter().enumerate() {
            if let Some(col_idx) = arch.column_index(component) {
                if with_components
                    .iter()
                    .all(|t| arch.column_index(*t).is_some())
                    && without_components
                        .iter()
                        .all(|t| arch.column_index(*t).is_none())
                    && (with_any_components.is_empty()
                        || with_any_components
                            .iter()
                            .any(|t| arch.column_index(*t).is_some()))
                    && without_any_components
                        .iter()
                        .all(|t| arch.column_index(*t).is_none())
                {
                    matches.push((arch_id, col_idx));
                }
            }
        }

        let mut cache = self.query_plan_cache.borrow_mut();
        cache.sync_version(self.archetype_version);
        cache.insert_single(
            component,
            &with_components,
            &without_components,
            &with_any_components,
            &without_any_components,
            matches.clone(),
        );
        matches
    }

    fn has_live_component_instances(&self, type_id: TypeId) -> bool {
        self.archetypes
            .iter()
            .any(|arch| arch.column_index(type_id).is_some() && !arch.is_empty())
    }

    /// Registers a new component in the scene.
    pub fn register_component<C: Component + 'static>(&mut self) {
        self.register_component_immediate::<C>();
    }

    /// Registers a tuple of component types in the scene.
    pub fn register_components<T: ComponentTuple>(&mut self) {
        T::register_all(self);
    }

    pub(crate) fn register_component_immediate<C: Component + 'static>(&mut self) {
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
        self.bundle_spawn_cache.clear();

        info!("Registered component: {}", C::type_name());
    }

    /// Deregisters a component from the scene.
    pub fn deregister_component<C: Component + 'static>(&mut self) {
        self.deregister_component_immediate::<C>();
    }

    pub(crate) fn deregister_component_immediate<C: Component + 'static>(&mut self) {
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
        self.bundle_spawn_cache.clear();
        self.component_change_state
            .retain(|(_, tracked_type_id), _| *tracked_type_id != type_id);
        self.removed_component_events.remove(&type_id);

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

    fn validate_type_ids_registered(&self, component_types: &[TypeId]) -> bool {
        for component_type in component_types {
            if !self.component_info.contains_key(component_type) {
                error!(
                    "Component TypeId {:?} not registered, cannot add bundle/components",
                    component_type
                );
                return false;
            }
        }
        true
    }

    /// Adds a component to an entity by its ID and an instance of the component.
    /// Overwrites the previous component if another component of the same type is added.
    pub fn add_component<C: Component + 'static>(&mut self, entity_id: Entity, component: C) {
        self.add_component_immediate(entity_id, component);
    }

    pub fn add_components<B: Bundle>(&mut self, entity_id: Entity, components: B) {
        self.add_with_components_immediate(entity_id, components.into_components());
    }

    pub(crate) fn add_component_immediate<C: Component + 'static>(
        &mut self,
        entity_id: Entity,
        component: C,
    ) {
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
            self.mark_component_changed(entity_id, type_id);
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
        self.mark_component_added_and_changed(entity_id, type_id);

        info!(
            "Added component {} to entity {}!",
            C::type_name(),
            entity_id.index
        );
    }

    pub fn remove_component<C: Component + 'static>(&mut self, entity_id: Entity) {
        self.remove_component_immediate::<C>(entity_id);
    }

    pub fn remove_components<T: ComponentTuple>(&mut self, entity_id: Entity) {
        self.remove_with_components_immediate(entity_id, T::type_ids());
    }

    pub(crate) fn remove_with_components_immediate(
        &mut self,
        entity_id: Entity,
        component_types: Vec<TypeId>,
    ) {
        if !self.is_alive(entity_id) || component_types.is_empty() {
            return;
        }

        let loc = match self.get_location(entity_id) {
            Some(loc) => loc,
            None => return,
        };
        let old_arch_id = loc.archetype;
        let old_arch = self.archetypes.get(old_arch_id);

        let mut removed_types = Vec::new();
        let mut removed_indices = Vec::new();
        for type_id in component_types {
            if removed_types.contains(&type_id) {
                continue;
            }
            let Some(index) = self.component_index.get(&type_id).copied() else {
                continue;
            };
            if old_arch.column_index(type_id).is_none() {
                continue;
            }
            removed_types.push(type_id);
            removed_indices.push(index);
        }

        if removed_types.is_empty() {
            return;
        }

        let mut component_set = old_arch.set().clone();
        for index in removed_indices {
            component_set.remove(index);
        }

        let new_arch_id = self.ensure_archetype(component_set);
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
            let type_id = new_arch.types()[new_idx];
            if let Some(old_idx) = old_arch.column_index(type_id) {
                let _ = old_arch.columns_mut()[old_idx]
                    .move_last_to(&mut new_arch.columns_mut()[new_idx]);
            }
        }

        for old_idx in 0..old_arch.types().len() {
            let type_id = old_arch.types()[old_idx];
            if new_arch.column_index(type_id).is_none() {
                let _ = old_arch.columns_mut()[old_idx].drop_last();
            }
        }

        old_arch.pop_entity();
        self.set_location(entity_id, new_arch_id, new_row);

        for type_id in removed_types {
            self.mark_component_removed(entity_id, type_id);
        }
    }

    pub(crate) fn remove_component_immediate<C: Component + 'static>(&mut self, entity_id: Entity) {
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
        self.mark_component_removed(entity_id, type_id);

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
        self.mark_component_changed(entity_id, C::type_id());
        let loc = self.get_location(entity_id)?;
        let arch = self.archetypes.get_mut(loc.archetype);
        let col_idx = arch.column_index(C::type_id())?;
        arch.columns_mut().get_mut(col_idx)?.get_mut::<C>(loc.row)
    }

    /// Returns whether the entity currently has component `C`.
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
            self.delete_entity_immediate(entity);
        }
    }

    /// Deletes all entities that have the given components.
    pub fn delete_entities_with(&mut self, components: Vec<TypeId>) {
        self.delete_entities_with_immediate(components);
    }

    pub(crate) fn delete_entities_with_immediate(&mut self, components: Vec<TypeId>) {
        let Some(indices) = self.component_indices_from_type_ids(&components) else {
            return;
        };
        self.delete_entities_with_indices(&indices);
    }

    /// Registers a prefab with the given name and factory function.
    pub fn register_prefab(&mut self, name: &str, factory: crate::prefabs::PrefabFactory) {
        self.register_prefab_immediate(name, factory);
    }

    pub(crate) fn register_prefab_immediate(
        &mut self,
        name: &str,
        factory: crate::prefabs::PrefabFactory,
    ) {
        self.prefabs.register(name, factory);
    }

    /// Spawns a prefab with the given name.
    pub fn spawn_prefab(&mut self, name: &str) -> Option<Entity> {
        self.spawn_prefab_immediate(name)
    }

    pub(crate) fn spawn_prefab_immediate(&mut self, name: &str) -> Option<Entity> {
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

    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Entity {
        bundle.spawn(self)
    }

    pub fn spawn_batch<B: Bundle + 'static>(
        &mut self,
        bundles: Vec<B>,
    ) -> Vec<Entity> {
        if bundles.is_empty() {
            return Vec::new();
        }

        let component_types = bundles[0].type_ids();
        self.__spawn_bundle_typed_batch(
            TypeId::of::<B>(),
            &component_types,
            bundles,
            |columns, column_indices, row, components| {
                components.write_components_reserved(columns, column_indices, row);
            },
        )
    }

    pub fn spawn_bundle<B: Bundle>(&mut self, bundle: B) -> Entity {
        bundle.spawn(self)
    }

    /// Spawns a batch of bundles immediately.
    pub fn spawn_bundle_batch<B: Bundle>(&mut self, bundles: Vec<B>) -> Vec<Entity> {
        B::spawn_batch(self, bundles)
    }

    pub(crate) fn add_with_components(
        &mut self,
        entity_id: Entity,
        components: Vec<ErasedComponent>,
    ) {
        self.add_with_components_immediate(entity_id, components);
    }

    pub(crate) fn add_with_components_immediate(
        &mut self,
        entity_id: Entity,
        mut components: Vec<ErasedComponent>,
    ) {
        if !self.is_alive(entity_id) || components.is_empty() {
            return;
        }
        if !self.validate_components_registered(&components) {
            return;
        }
        let submitted_types: Vec<TypeId> = components
            .iter()
            .map(|component| component.type_id)
            .collect();

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
            for type_id in submitted_types {
                self.mark_component_changed(entity_id, type_id);
            }
        } else {
            let mut existed_before_add = HashMap::with_capacity(submitted_types.len());
            for &type_id in &submitted_types {
                existed_before_add.entry(type_id).or_insert_with(|| {
                    self.archetypes
                        .get(old_arch_id)
                        .column_index(type_id)
                        .is_some()
                });
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

            for type_id in submitted_types {
                if existed_before_add.get(&type_id).copied().unwrap_or(false) {
                    self.mark_component_changed(entity_id, type_id);
                } else {
                    self.mark_component_added_and_changed(entity_id, type_id);
                }
            }
        }
    }

    /// Spawns an entity from erased components immediately.
    pub fn spawn_with_components(&mut self, components: Vec<ErasedComponent>) -> Entity {
        self.spawn_with_components_immediate(components)
    }

    pub(crate) fn spawn_with_components_immediate(
        &mut self,
        components: Vec<ErasedComponent>,
    ) -> Entity {
        if components.is_empty() {
            return self.new_entity_immediate();
        }
        if !self.validate_components_registered(&components) {
            return self.new_entity_immediate();
        }

        let mut component_set = ComponentSet::new();
        for component in &components {
            let index = self
                .component_index
                .get(&component.type_id)
                .copied()
                .unwrap_or_else(|| panic!("Component {:?} missing index", component.type_id));
            component_set.insert(index);
        }
        let archetype = self.ensure_archetype(component_set);
        let entity_id = self.allocate_entity_slot();
        let row = self.place_entity_in_archetype(entity_id, archetype);
        let mut components = components;
        let mut inserted_types = Vec::new();
        let arch = self.archetypes.get_mut(archetype);
        for type_id in arch.types().to_vec() {
            let component = Self::take_last_component_of_type(&mut components, type_id)
                .unwrap_or_else(|| panic!("Bundle missing component {:?}", type_id));
            let col_idx = arch
                .column_index(type_id)
                .unwrap_or_else(|| panic!("Archetype missing column for {:?}", type_id));
            (component.push_fn)(component.value, &mut arch.columns_mut()[col_idx]);
            inserted_types.push(type_id);
        }

        debug_assert!(
            arch.columns().iter().all(|col| col.len() == row + 1),
            "column length mismatch after spawn_with_components"
        );

        for type_id in inserted_types {
            self.mark_component_added_and_changed(entity_id, type_id);
        }

        entity_id
    }

    #[doc(hidden)]
    pub fn __spawn_bundle_typed<F>(
        &mut self,
        bundle_type: TypeId,
        component_types: &[TypeId],
        writer: F,
    ) -> Entity
    where
        F: FnOnce(&mut [Column], &[usize], usize),
    {
        if component_types.is_empty() {
            return self.new_entity_immediate();
        }

        if !self.bundle_spawn_cache.contains_key(&bundle_type) {
            if !self.validate_type_ids_registered(component_types) {
                return self.new_entity_immediate();
            }

            let mut component_set = ComponentSet::new();
            for component_type in component_types {
                let index = self
                    .component_index
                    .get(component_type)
                    .copied()
                    .unwrap_or_else(|| panic!("Component {:?} missing index", component_type));
                component_set.insert(index);
            }

            let archetype = self.ensure_archetype(component_set);
            let arch = self.archetypes.get(archetype);
            let mut column_indices = Vec::with_capacity(component_types.len());
            for component_type in component_types {
                let col_idx = arch
                    .column_index(*component_type)
                    .unwrap_or_else(|| panic!("Archetype missing column for {:?}", component_type));
                column_indices.push(col_idx);
            }

            self.bundle_spawn_cache.insert(
                bundle_type,
                BundleSpawnPlan {
                    archetype,
                    column_indices: Arc::from(column_indices),
                },
            );
        }

        let (archetype, column_indices_ptr, column_indices_len) = {
            let plan = self
                .bundle_spawn_cache
                .get(&bundle_type)
                .unwrap_or_else(|| panic!("bundle spawn plan unexpectedly missing"));
            (
                plan.archetype,
                plan.column_indices.as_ptr(),
                plan.column_indices.len(),
            )
        };

        let entity_id = self.allocate_entity_slot();
        let row = self.place_entity_in_archetype(entity_id, archetype);
        let arch = self.archetypes.get_mut(archetype);
        // SAFETY: bundle_spawn_cache is not mutated between pointer capture and use.
        let column_indices =
            unsafe { slice::from_raw_parts(column_indices_ptr, column_indices_len) };
        writer(arch.columns_mut(), column_indices, row);

        debug_assert!(
            arch.columns().iter().all(|col| col.len() == row + 1),
            "column length mismatch after __spawn_bundle_typed"
        );
        for type_id in component_types {
            self.mark_component_added_and_changed(entity_id, *type_id);
        }

        entity_id
    }

    #[doc(hidden)]
    pub fn __spawn_bundle_typed_batch<T, F>(
        &mut self,
        bundle_type: TypeId,
        component_types: &[TypeId],
        bundles: Vec<T>,
        mut writer: F,
    ) -> Vec<Entity>
    where
        F: FnMut(&mut [Column], &[usize], usize, T),
    {
        if bundles.is_empty() {
            return Vec::new();
        }
        if component_types.is_empty() {
            return bundles
                .into_iter()
                .map(|_| self.new_entity_immediate())
                .collect();
        }

        if !self.bundle_spawn_cache.contains_key(&bundle_type) {
            if !self.validate_type_ids_registered(component_types) {
                return bundles
                    .into_iter()
                    .map(|_| self.new_entity_immediate())
                    .collect();
            }

            let mut component_set = ComponentSet::new();
            for component_type in component_types {
                let index = self
                    .component_index
                    .get(component_type)
                    .copied()
                    .unwrap_or_else(|| panic!("Component {:?} missing index", component_type));
                component_set.insert(index);
            }

            let archetype = self.ensure_archetype(component_set);
            let arch = self.archetypes.get(archetype);
            let mut column_indices = Vec::with_capacity(component_types.len());
            for component_type in component_types {
                let col_idx = arch
                    .column_index(*component_type)
                    .unwrap_or_else(|| panic!("Archetype missing column for {:?}", component_type));
                column_indices.push(col_idx);
            }

            self.bundle_spawn_cache.insert(
                bundle_type,
                BundleSpawnPlan {
                    archetype,
                    column_indices: Arc::from(column_indices),
                },
            );
        }

        let (archetype, column_indices_ptr, column_indices_len) = {
            let plan = self
                .bundle_spawn_cache
                .get(&bundle_type)
                .unwrap_or_else(|| panic!("bundle spawn plan unexpectedly missing"));
            (
                plan.archetype,
                plan.column_indices.as_ptr(),
                plan.column_indices.len(),
            )
        };

        let count = bundles.len();
        self.entities.reserve(count);
        self.generations.reserve(count);
        self.entity_locations.reserve(count);
        self.archetypes.get_mut(archetype).reserve_rows(count);

        let current_next_is_reusable = (self.next_id as usize) < self.entities.len()
            && self.entities[self.next_id as usize].is_none();
        let reusable_available =
            self.id_queue.size() as usize + usize::from(current_next_is_reusable);
        let reuse_count = reusable_available.min(count);

        let fresh_count = count - reuse_count;
        let fresh_start = self.entities.len() as u32;

        self.active_entities = self
            .active_entities
            .checked_add(count as u32)
            .expect("active_entities overflow");

        let fresh_end = fresh_start as usize + fresh_count;
        if fresh_end > self.generations.len() {
            self.generations.resize(fresh_end, 0);
        }
        if fresh_end > self.entity_locations.len() {
            self.entity_locations.resize(fresh_end, None);
        }

        let mut entities = Vec::with_capacity(count);
        let mut bundles = bundles.into_iter();
        // SAFETY: bundle_spawn_cache is not mutated between pointer capture and use.
        let column_indices =
            unsafe { slice::from_raw_parts(column_indices_ptr, column_indices_len) };
        let arch = self.archetypes.get_mut(archetype);

        for _ in 0..reuse_count {
            let bundle = bundles
                .next()
                .unwrap_or_else(|| panic!("bundle iteration unexpectedly short"));
            let index = self.next_id;
            let slot = index as usize;
            let gen = self.generations[slot];
            let entity_id = Entity { index, gen };

            self.entities[slot] = Some(entity_id);
            let row = arch.push_entity(entity_id);
            self.entity_locations[slot] = Some(EntityLocation {
                archetype,
                row,
                gen,
            });
            writer(arch.columns_mut(), column_indices, row, bundle);
            entities.push(entity_id);
            self.next_id = self
                .id_queue
                .dequeue()
                .unwrap_or_else(|| self.entities.len() as u32);
        }

        for i in 0..fresh_count {
            let bundle = bundles
                .next()
                .unwrap_or_else(|| panic!("bundle iteration unexpectedly short"));
            let index = fresh_start + i as u32;
            let slot = index as usize;
            let gen = self.generations[slot];
            let entity_id = Entity { index, gen };

            self.entities.push(Some(entity_id));
            let row = arch.push_entity(entity_id);
            self.entity_locations[slot] = Some(EntityLocation {
                archetype,
                row,
                gen,
            });
            writer(arch.columns_mut(), column_indices, row, bundle);
            entities.push(entity_id);
        }

        if self.id_queue.is_empty() {
            self.next_id = self.entities.len() as u32;
        }

        for &entity in &entities {
            for type_id in component_types {
                self.mark_component_added_and_changed(entity, *type_id);
            }
        }

        entities
    }
}

#[cfg(test)]
mod tests {
    use super::Scene;
    use crate::{Component, ErasedComponent};

    #[derive(Component)]
    struct A;

    #[derive(Component)]
    struct B;

    #[derive(Component, Eq)]
    struct Value(i32);

    #[derive(Component)]
    struct Unregistered;

    #[derive(Component)]
    struct IncludeTag;

    #[derive(Component)]
    struct ExcludeTag;

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

    #[test]
    #[should_panic(expected = "query called with duplicate component types")]
    fn query_mut_pair_rejects_identical_component_types() {
        let mut scene = Scene::new();
        scene.register_component::<A>();

        let entity = scene.new_entity();
        scene.add_component(entity, A);

        let _ = scene.query_mut::<(&mut A, &mut A), ()>().iter();
    }

    #[test]
    fn query_includes_entity_id_for_read_tuples() {
        let mut scene = Scene::new();
        scene.register_component::<Value>();

        let e = scene.new_entity();
        scene.add_component(e, Value(42));

        let mut iter = scene.query::<(crate::Entity, &Value), ()>().iter();
        let (entity, value) = iter.next().expect("expected one result");
        assert_eq!(entity, e);
        assert_eq!(value.0, 42);
        assert!(iter.next().is_none());
    }

    #[test]
    fn query_mut_includes_entity_id_for_write_tuples() {
        let mut scene = Scene::new();
        scene.register_component::<Value>();

        let e = scene.new_entity();
        scene.add_component(e, Value(7));

        let mut iter = scene.query_mut::<(crate::Entity, &mut Value), ()>().iter();
        let (entity, value) = iter.next().expect("expected one result");
        assert_eq!(entity, e);
        value.0 = 11;
        assert!(iter.next().is_none());

        assert_eq!(scene.get_component::<Value>(e).map(|v| v.0), Some(11));
    }

    #[test]
    fn add_component_moves_entity_between_archetypes_and_preserves_swapped_entity_location() {
        let mut scene = Scene::new();
        scene.register_component::<Value>();
        scene.register_component::<B>();

        let e1 = scene.new_entity();
        let e2 = scene.new_entity();
        scene.add_component(e1, Value(10));
        scene.add_component(e2, Value(20));

        scene.add_component(e1, B);

        assert_eq!(scene.get_component::<Value>(e1).map(|v| v.0), Some(10));
        assert!(scene.get_component::<B>(e1).is_some());
        assert_eq!(scene.get_component::<Value>(e2).map(|v| v.0), Some(20));
        assert!(scene.get_component::<B>(e2).is_none());
    }

    #[test]
    fn remove_components_moves_entity_between_archetypes_and_preserves_remaining_components() {
        let mut scene = Scene::new();
        scene.register_component::<A>();
        scene.register_component::<B>();
        scene.register_component::<Value>();

        let entity = scene.new_entity();
        scene.add_component(entity, A);
        scene.add_component(entity, B);
        scene.add_component(entity, Value(10));

        scene.remove_components::<(A, B)>(entity);

        assert!(scene.get_component::<A>(entity).is_none());
        assert!(scene.get_component::<B>(entity).is_none());
        assert_eq!(
            scene.get_component::<Value>(entity).map(|value| value.0),
            Some(10)
        );
    }

    #[test]
    fn normalized_components_are_order_independent_and_deduplicated() {
        let components_abab = Scene::normalized_components(&[
            A::type_id(),
            B::type_id(),
            A::type_id(),
            B::type_id(),
        ]);
        let components_ba = Scene::normalized_components(&[
            B::type_id(),
            A::type_id(),
        ]);

        assert_eq!(components_abab, components_ba);
        assert_eq!(components_abab.len(), 2);
    }

    #[test]
    fn query_with_and_without_filters_entities() {
        let mut scene = Scene::new();
        scene.register_component::<Value>();
        scene.register_component::<IncludeTag>();
        scene.register_component::<ExcludeTag>();

        let keep = scene.new_entity();
        scene.add_component(keep, Value(10));
        scene.add_component(keep, IncludeTag);

        let filtered_out = scene.new_entity();
        scene.add_component(filtered_out, Value(20));
        scene.add_component(filtered_out, IncludeTag);
        scene.add_component(filtered_out, ExcludeTag);

        let values: Vec<i32> = scene
            .query::<&Value, (crate::With<IncludeTag>, crate::Without<ExcludeTag>)>()
            .iter()
            .map(|v| v.0)
            .collect();

        assert_eq!(values, vec![10]);
    }

    #[test]
    fn query_with_all_and_without_all_filters_entities() {
        let mut scene = Scene::new();
        scene.register_component::<Value>();
        scene.register_component::<IncludeTag>();
        scene.register_component::<ExcludeTag>();
        scene.register_component::<B>();

        let keep = scene.new_entity();
        scene.add_component(keep, Value(10));
        scene.add_component(keep, IncludeTag);

        let filtered_out = scene.new_entity();
        scene.add_component(filtered_out, Value(20));
        scene.add_component(filtered_out, IncludeTag);
        scene.add_component(filtered_out, ExcludeTag);
        scene.add_component(filtered_out, B);

        let values: Vec<i32> = scene
            .query::<&Value, (crate::With<IncludeTag>, crate::WithoutAny<(ExcludeTag, B)>)>()
            .iter()
            .map(|v| v.0)
            .collect();

        assert_eq!(values, vec![10]);
    }

    #[test]
    fn query_with_any_filters_entities() {
        let mut scene = Scene::new();
        scene.register_component::<Value>();
        scene.register_component::<IncludeTag>();
        scene.register_component::<B>();

        let include = scene.new_entity();
        scene.add_component(include, Value(10));
        scene.add_component(include, IncludeTag);

        let b_only = scene.new_entity();
        scene.add_component(b_only, Value(20));
        scene.add_component(b_only, B);

        let neither = scene.new_entity();
        scene.add_component(neither, Value(30));

        let values: Vec<i32> = scene
            .query::<&Value, crate::WithAny<(IncludeTag, B)>>()
            .iter()
            .map(|v| v.0)
            .collect();

        assert_eq!(values, vec![10, 20]);
    }

    #[test]
    fn query_without_any_filters_entities() {
        let mut scene = Scene::new();
        scene.register_component::<Value>();
        scene.register_component::<ExcludeTag>();
        scene.register_component::<B>();

        let keep = scene.new_entity();
        scene.add_component(keep, Value(10));

        let exclude_tag = scene.new_entity();
        scene.add_component(exclude_tag, Value(20));
        scene.add_component(exclude_tag, ExcludeTag);

        let exclude_b = scene.new_entity();
        scene.add_component(exclude_b, Value(30));
        scene.add_component(exclude_b, B);

        let values: Vec<i32> = scene
            .query::<&Value, crate::WithoutAny<(ExcludeTag, B)>>()
            .iter()
            .map(|v| v.0)
            .collect();

        assert_eq!(values, vec![10]);
    }

    #[test]
    fn query_with_and_without_same_tag_is_empty() {
        let mut scene = Scene::new();
        scene.register_component::<Value>();
        scene.register_component::<IncludeTag>();

        let entity = scene.new_entity();
        scene.add_component(entity, Value(1));
        scene.add_component(entity, IncludeTag);

        let mut iter = scene
            .query::<&Value, (crate::With<IncludeTag>, crate::Without<IncludeTag>)>()
            .iter();

        assert!(iter.next().is_none());
    }

    #[test]
    fn component_change_tracking_uses_component_event_tick() {
        let mut scene = Scene::new();
        scene.register_component::<Value>();

        scene.set_component_event_tick(10);
        let entity = scene.new_entity();
        scene.add_component(entity, Value(1));
        assert!(scene.component_added_since::<Value>(entity, 9));
        assert!(scene.component_changed_since::<Value>(entity, 9));
        assert!(!scene.component_changed_since::<Value>(entity, 10));

        scene.set_component_event_tick(15);
        if let Some(value) = scene.get_component_mut::<Value>(entity) {
            value.0 = 2;
        }
        assert!(scene.component_changed_since::<Value>(entity, 10));
    }

    #[test]
    fn removed_since_tracks_component_removals() {
        let mut scene = Scene::new();
        scene.register_component::<Value>();

        let entity = scene.new_entity();
        scene.set_component_event_tick(3);
        scene.add_component(entity, Value(1));
        scene.set_component_event_tick(7);
        scene.remove_component::<Value>(entity);

        let removed = scene.removed_since::<Value>(5);
        assert_eq!(removed, vec![entity]);
    }

    #[test]
    fn query_mut_marks_component_changed() {
        let mut scene = Scene::new();
        scene.register_component::<Value>();

        let entity = scene.new_entity();
        scene.set_component_event_tick(10);
        scene.add_component(entity, Value(1));

        scene.set_component_event_tick(20);
        {
            let mut iter = scene.query_mut::<&mut Value, ()>().iter();
            let _ = iter.next();
        }

        assert!(scene.component_changed_since::<Value>(entity, 10));
    }

    #[test]
    fn query_mut_marks_only_mutable_fetches() {
        let mut scene = Scene::new();
        scene.register_component::<Value>();
        scene.register_component::<A>();

        let entity = scene.new_entity();
        scene.set_component_event_tick(10);
        scene.add_component(entity, Value(1));
        scene.add_component(entity, A);

        scene.set_component_event_tick(30);
        {
            let mut iter = scene.query_mut::<(&Value, &mut A), ()>().iter();
            let _ = iter.next();
        }

        assert!(!scene.component_changed_since::<Value>(entity, 10));
        assert!(scene.component_changed_since::<A>(entity, 10));
    }

    #[test]
    fn temporal_query_filters_use_default_query_since_tick() {
        let mut scene = Scene::new();
        scene.register_component::<Value>();

        let entity = scene.new_entity();
        scene.set_component_event_tick(5);
        scene.add_component(entity, Value(1));

        scene.set_default_query_since_tick(4);
        let added_count = scene.query::<&Value, crate::Added<Value>>().iter().count();
        assert_eq!(added_count, 1);

        scene.set_component_event_tick(9);
        if let Some(value) = scene.get_component_mut::<Value>(entity) {
            value.0 = 2;
        }

        let changed_count = scene
            .query::<&Value, crate::Changed<Value>>()
            .iter()
            .count();
        assert_eq!(changed_count, 1);
    }

    #[test]
    fn temporal_query_constraints_can_use_different_since_ticks_per_filter() {
        let mut scene = Scene::new();
        scene.register_component::<Value>();
        scene.register_component::<A>();

        scene.set_component_event_tick(5);
        let entity = scene.new_entity();
        scene.add_component(entity, Value(1));
        scene.add_component(entity, A);

        scene.set_component_event_tick(10);
        if let Some(value) = scene.get_component_mut::<A>(entity) {
            *value = A;
        }

        let matching = scene
            .query::<&Value, (crate::Added<Value>, crate::Changed<A>)>()
            .added_since::<Value>(4)
            .changed_since::<A>(9)
            .iter()
            .count();
        assert_eq!(matching, 1);

        let non_matching = scene
            .query::<&Value, (crate::Added<Value>, crate::Changed<A>)>()
            .added_since::<Value>(6)
            .changed_since::<A>(10)
            .iter()
            .count();
        assert_eq!(non_matching, 0);
    }

    #[test]
    fn query_optional_read_fetches_return_none_when_component_is_missing() {
        let mut scene = Scene::new();
        scene.register_component::<Value>();
        scene.register_component::<A>();

        let with_a = scene.new_entity();
        scene.add_component(with_a, Value(1));
        scene.add_component(with_a, A);

        let without_a = scene.new_entity();
        scene.add_component(without_a, Value(2));

        let results: Vec<(i32, bool)> = scene
            .query::<(&Value, Option<&A>), ()>()
            .iter()
            .map(|(value, maybe_a)| (value.0, maybe_a.is_some()))
            .collect();

        assert_eq!(results.len(), 2);
        assert!(results.contains(&(1, true)));
        assert!(results.contains(&(2, false)));
    }

    #[test]
    fn query_optional_write_fetches_do_not_mark_missing_components_as_changed() {
        let mut scene = Scene::new();
        scene.register_component::<Value>();
        scene.register_component::<A>();

        let with_a = scene.new_entity();
        scene.set_component_event_tick(1);
        scene.add_component(with_a, Value(1));
        scene.add_component(with_a, A);

        let without_a = scene.new_entity();
        scene.add_component(without_a, Value(2));

        scene.set_component_event_tick(10);
        scene
            .query_mut::<(&Value, Option<&mut A>), ()>()
            .for_each(|(_value, maybe_a)| {
                if let Some(a) = maybe_a {
                    *a = A;
                }
            });

        assert!(scene.component_changed_since::<A>(with_a, 1));
        assert!(!scene.component_changed_since::<A>(without_a, 1));
    }
}
