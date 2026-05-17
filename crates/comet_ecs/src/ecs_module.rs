use std::any::TypeId;
use comet_app::{App, Module};
use crate::{
    Bundle, Camera, Component, ComponentTuple, Entity, PrefabFactory, QueryParam, QuerySpecMut, Collider, Sprite, Scene, Text, Transform
};

pub struct EcsModule {
    pub scene: Scene,
}

impl EcsModule {
    pub fn new() -> Self {
        Self { scene: Scene::new() }
    }

    pub fn preset_2d() -> Self {
        let mut m = Self::new();
        m.scene.register_components::<(Camera, Transform, Collider, Sprite, Text)>();
        m
    }

    pub fn preset_3d() -> Self {
        let mut m = Self::new();
        m.scene.register_components::<(Transform, Collider, Text)>();
        m
    }
}

impl Module for EcsModule {
    fn build(&mut self, app: &mut App) {
        app.add_pre_tick_hook(|app| {
            let m = app.get_module_mut::<EcsModule>();
            let tick = m.scene.component_event_tick().wrapping_sub(1);
            m.scene.set_default_query_since_tick(tick);
        });
        app.add_post_tick_hook(|app| {
            let m = app.get_module_mut::<EcsModule>();
            m.scene.apply_commands();
            let _ = m.scene.advance_component_event_tick();
        });
    }
}

pub trait EcsModuleExt {
    fn scene(&self) -> &Scene;
    fn scene_mut(&mut self) -> &mut Scene;

    fn spawn<B: Bundle + 'static>(&mut self, bundle: B) -> Entity;
    fn spawn_batch<B: Bundle + 'static>(&mut self, bundles: Vec<B>) -> Vec<Entity>;
    
    fn deferred_spawn_empty(&mut self);
    fn deferred_spawn<B: Bundle>(&mut self, bundle: B);
    fn deferred_spawn_batch<B: Bundle>(&mut self, batch: Vec<B>);
    fn deferred_delete_entity(&mut self, entity: Entity);
    fn deferred_register_component<C: Component>(&mut self);
    fn deferred_register_components<T: ComponentTuple>(&mut self);
    fn deferred_deregister_component<C: Component>(&mut self);
    fn deferred_add_component<C: Component>(&mut self, entity: Entity, component: C);
    fn deferred_add_components<B: Bundle>(&mut self, entity: Entity, components: B);
    fn deferred_remove_component<C: Component>(&mut self, entity: Entity);
    fn deferred_remove_components<T: ComponentTuple>(&mut self, entity: Entity);
    fn deferred_delete_entities_with(&mut self, components: Vec<TypeId>);
    fn deferred_register_prefab(&mut self, name: impl Into<String>, factory: PrefabFactory);
    fn deferred_spawn_prefab(&mut self, name: impl Into<String>);
    
    fn apply_deferred_commands(&mut self);
    fn queued_deferred_command_count(&self) -> usize;

    fn query<'a, Data, Filters>(&'a self) -> <QueryParam<Data, Filters> as QuerySpecMut<'a>>::Builder
    where
        QueryParam<Data, Filters>: QuerySpecMut<'a>;

    fn new_entity(&mut self) -> Entity;
    fn delete_entity(&mut self, entity_id: Entity);
    fn get_entity(&self, entity_id: Entity) -> Option<&Entity>;

    fn register_component<C: Component>(&mut self);
    fn register_components<T: ComponentTuple>(&mut self);
    fn deregister_component<C: Component>(&mut self);
    fn add_component<C: Component>(&mut self, entity_id: Entity, component: C);
    fn add_components<B: Bundle>(&mut self, entity_id: Entity, components: B);
    fn remove_component<C: Component>(&mut self, entity_id: Entity);
    fn remove_components<T: ComponentTuple>(&mut self, entity_id: Entity);
    fn get_component<C: Component>(&self, entity_id: Entity) -> Option<&C>;
    fn get_component_mut<C: Component>(&mut self, entity_id: Entity) -> Option<&mut C>;
    fn delete_entities_with(&mut self, components: Vec<TypeId>);
    fn has<C: Component>(&self, entity_id: Entity) -> bool;

    fn register_prefab(&mut self, name: &str, factory: PrefabFactory);
    fn spawn_prefab(&mut self, name: &str) -> Option<Entity>;
    fn has_prefab(&self, name: &str) -> bool;
}

impl EcsModuleExt for App {
    fn scene(&self) -> &Scene {
        &self.get_module::<EcsModule>().scene
    }

    fn scene_mut(&mut self) -> &mut Scene {
        &mut self.get_module_mut::<EcsModule>().scene
    }

    fn spawn<B: Bundle + 'static>(&mut self, bundle: B) -> Entity {
        self.get_module_mut::<EcsModule>().scene.spawn_bundle(bundle)
    }

    fn spawn_batch<B: Bundle + 'static>(&mut self, bundles: Vec<B>) -> Vec<Entity> {
        self.get_module_mut::<EcsModule>().scene.spawn_bundle_batch(bundles)
    }

    fn deferred_spawn_empty(&mut self) {
        self.get_module_mut::<EcsModule>().scene.deferred_spawn_empty();
    }

    fn deferred_spawn<B: Bundle>(&mut self, bundle: B) {
        self.get_module_mut::<EcsModule>().scene.deferred_spawn(bundle);
    }

    fn deferred_spawn_batch<B: Bundle>(&mut self, batch: Vec<B>) {
        self.get_module_mut::<EcsModule>().scene.deferred_spawn_batch(batch);
    }

    fn deferred_delete_entity(&mut self, entity: Entity) {
        self.get_module_mut::<EcsModule>().scene.deferred_delete_entity(entity);
    }

    fn deferred_register_component<C: Component>(&mut self) {
        self.get_module_mut::<EcsModule>().scene.deferred_register_component::<C>();
    }

    fn deferred_register_components<T: ComponentTuple>(&mut self) {
        self.get_module_mut::<EcsModule>().scene.deferred_register_components::<T>();
    }

    fn deferred_deregister_component<C: Component>(&mut self) {
        self.get_module_mut::<EcsModule>().scene.deferred_deregister_component::<C>();
    }

    fn deferred_add_component<C: Component>(&mut self, entity: Entity, component: C) {
        self.get_module_mut::<EcsModule>().scene.deferred_add_component::<C>(entity, component);
    }

    fn deferred_add_components<B: Bundle>(&mut self, entity: Entity, components: B) {
        self.get_module_mut::<EcsModule>().scene.deferred_add_components(entity, components);
    }

    fn deferred_remove_component<C: Component>(&mut self, entity: Entity) {
        self.get_module_mut::<EcsModule>().scene.deferred_remove_component::<C>(entity);
    }

    fn deferred_remove_components<T: ComponentTuple>(&mut self, entity: Entity) {
        self.get_module_mut::<EcsModule>().scene.deferred_remove_components::<T>(entity);
    }

    fn deferred_delete_entities_with(&mut self, components: Vec<TypeId>) {
        self.get_module_mut::<EcsModule>().scene.deferred_delete_entities_with(components);
    }

    fn deferred_register_prefab(&mut self, name: impl Into<String>, factory: PrefabFactory) {
        self.get_module_mut::<EcsModule>().scene.deferred_register_prefab(name, factory);
    }

    fn deferred_spawn_prefab(&mut self, name: impl Into<String>) {
        self.get_module_mut::<EcsModule>().scene.deferred_spawn_prefab(name);
    }

    fn apply_deferred_commands(&mut self) {
        self.get_module_mut::<EcsModule>().scene.apply_commands();
    }

    fn queued_deferred_command_count(&self) -> usize {
        self.get_module::<EcsModule>().scene.queued_command_count()
    }

    fn query<'a, Data, Filters>(&'a self) -> <QueryParam<Data, Filters> as QuerySpecMut<'a>>::Builder
    where
        QueryParam<Data, Filters>: QuerySpecMut<'a>,
    {
        self.get_module::<EcsModule>().scene.query_mut::<Data, Filters>()
    }

    fn new_entity(&mut self) -> Entity {
        self.get_module_mut::<EcsModule>().scene.new_entity()
    }

    fn delete_entity(&mut self, entity_id: Entity) {
        self.get_module_mut::<EcsModule>().scene.delete_entity(entity_id);
    }

    fn get_entity(&self, entity_id: Entity) -> Option<&Entity> {
        self.get_module::<EcsModule>().scene.get_entity(entity_id)
    }

    fn register_component<C: Component>(&mut self) {
        self.get_module_mut::<EcsModule>().scene.register_component::<C>();
    }

    fn register_components<T: ComponentTuple>(&mut self) {
        self.get_module_mut::<EcsModule>().scene.register_components::<T>();
    }

    fn deregister_component<C: Component>(&mut self) {
        self.get_module_mut::<EcsModule>().scene.deregister_component::<C>();
    }

    fn add_component<C: Component>(&mut self, entity_id: Entity, component: C) {
        self.get_module_mut::<EcsModule>().scene.add_component(entity_id, component);
    }

    fn add_components<B: Bundle>(&mut self, entity_id: Entity, components: B) {
        self.get_module_mut::<EcsModule>().scene.add_components(entity_id, components);
    }

    fn remove_component<C: Component>(&mut self, entity_id: Entity) {
        self.get_module_mut::<EcsModule>().scene.remove_component::<C>(entity_id);
    }

    fn remove_components<T: ComponentTuple>(&mut self, entity_id: Entity) {
        self.get_module_mut::<EcsModule>().scene.remove_components::<T>(entity_id);
    }

    fn get_component<C: Component>(&self, entity_id: Entity) -> Option<&C> {
        self.get_module::<EcsModule>().scene.get_component::<C>(entity_id)
    }

    fn get_component_mut<C: Component>(&mut self, entity_id: Entity) -> Option<&mut C> {
        self.get_module_mut::<EcsModule>().scene.get_component_mut::<C>(entity_id)
    }

    fn delete_entities_with(&mut self, components: Vec<TypeId>) {
        self.get_module_mut::<EcsModule>().scene.delete_entities_with(components);
    }

    fn has<C: Component>(&self, entity_id: Entity) -> bool {
        self.get_module::<EcsModule>().scene.has::<C>(entity_id)
    }

    fn register_prefab(&mut self, name: &str, factory: PrefabFactory) {
        self.get_module_mut::<EcsModule>().scene.register_prefab(name, factory);
    }

    fn spawn_prefab(&mut self, name: &str) -> Option<Entity> {
        self.get_module_mut::<EcsModule>().scene.spawn_prefab(name)
    }

    fn has_prefab(&self, name: &str) -> bool {
        self.get_module::<EcsModule>().scene.has_prefab(name)
    }
}
