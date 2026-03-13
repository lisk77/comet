use crate::{Bundle, Component, ComponentTuple, Entity, ErasedComponent, PrefabFactory, Scene};
use std::any::TypeId;

/// A deferred operation that can be applied to a [`Scene`].
pub enum SceneCommand {
    SpawnEntity,
    DeleteEntity(Entity),
    RegisterComponent {
        type_id: TypeId,
        register_fn: fn(&mut Scene),
    },
    DeregisterComponent {
        type_id: TypeId,
        deregister_fn: fn(&mut Scene),
    },
    AddComponent {
        entity: Entity,
        component: ErasedComponent,
    },
    RemoveComponent {
        entity: Entity,
        type_id: TypeId,
        remove_fn: fn(&mut Scene, Entity),
    },
    DeleteEntitiesWith(Vec<TypeId>),
    RegisterPrefab {
        name: String,
        factory: PrefabFactory,
    },
    SpawnPrefab(String),
    SpawnBundle {
        components: Vec<ErasedComponent>,
    },
    SpawnBundleBatch {
        bundles: Vec<Vec<ErasedComponent>>,
    },
    AddBundle {
        entity: Entity,
        components: Vec<ErasedComponent>,
    },
}

#[derive(Default)]
/// Queue of deferred [`SceneCommand`] values.
pub struct SceneCommands {
    queue: Vec<SceneCommand>,
}

impl SceneCommands {
    /// Creates an empty command queue.
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    /// Returns the amount of queued commands.
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Returns `true` if there are no queued commands.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Clears all queued commands without applying them.
    pub fn clear(&mut self) {
        self.queue.clear();
    }

    /// Queues a raw [`SceneCommand`].
    pub fn push(&mut self, command: SceneCommand) {
        self.queue.push(command);
    }

    /// Queues spawning of an empty entity.
    pub fn spawn_empty(&mut self) {
        self.push(SceneCommand::SpawnEntity);
    }

    /// Queues deleting an entity.
    pub fn delete_entity(&mut self, entity: Entity) {
        self.push(SceneCommand::DeleteEntity(entity));
    }

    /// Queues component type registration.
    pub fn register_component<C: Component>(&mut self) {
        self.push(SceneCommand::RegisterComponent {
            type_id: C::type_id(),
            register_fn: register_component_impl::<C>,
        });
    }

    /// Queues component type deregistration.
    pub fn deregister_component<C: Component>(&mut self) {
        self.push(SceneCommand::DeregisterComponent {
            type_id: C::type_id(),
            deregister_fn: deregister_component_impl::<C>,
        });
    }

    /// Queues adding or setting a component on an entity.
    pub fn add_component<C: Component>(&mut self, entity: Entity, component: C) {
        self.push(SceneCommand::AddComponent {
            entity,
            component: ErasedComponent::new(component),
        });
    }

    /// Queues removing a component from an entity.
    pub fn remove_component<C: Component>(&mut self, entity: Entity) {
        self.push(SceneCommand::RemoveComponent {
            entity,
            type_id: C::type_id(),
            remove_fn: remove_component_impl::<C>,
        });
    }

    /// Queues deletion of all entities matching a component tuple.
    pub fn delete_entities_with<Cs: ComponentTuple>(&mut self) {
        self.push(SceneCommand::DeleteEntitiesWith(Cs::type_ids()));
    }

    /// Queues prefab registration.
    pub fn register_prefab(&mut self, name: impl Into<String>, factory: PrefabFactory) {
        self.push(SceneCommand::RegisterPrefab {
            name: name.into(),
            factory,
        });
    }

    /// Queues prefab spawning by name.
    pub fn spawn_prefab(&mut self, name: impl Into<String>) {
        self.push(SceneCommand::SpawnPrefab(name.into()));
    }

    /// Queues spawning a single bundle.
    pub fn spawn_bundle<B: Bundle>(&mut self, bundle: B) {
        self.push(SceneCommand::SpawnBundle {
            components: bundle.into_components(),
        });
    }

    /// Queues batch spawning of bundles.
    pub fn spawn_bundle_batch<B: Bundle>(&mut self, bundles: Vec<B>) {
        let bundles = bundles
            .into_iter()
            .map(Bundle::into_components)
            .collect::<Vec<_>>();
        self.push(SceneCommand::SpawnBundleBatch { bundles });
    }

    /// Queues adding a bundle to an entity.
    pub fn add_bundle<B: Bundle>(&mut self, entity: Entity, bundle: B) {
        self.push(SceneCommand::AddBundle {
            entity,
            components: bundle.into_components(),
        });
    }

    /// Applies all queued commands in FIFO order.
    pub fn apply(&mut self, scene: &mut Scene) {
        let queued = std::mem::take(&mut self.queue);
        for command in queued {
            Self::apply_command(scene, command);
        }
    }

    /// Applies a single command immediately.
    pub fn apply_command(scene: &mut Scene, command: SceneCommand) {
        match command {
            SceneCommand::SpawnEntity => {
                let _ = scene.new_entity_immediate();
            }
            SceneCommand::DeleteEntity(entity) => scene.delete_entity_immediate(entity),
            SceneCommand::RegisterComponent {
                type_id: _type_id,
                register_fn,
            } => register_fn(scene),
            SceneCommand::DeregisterComponent {
                type_id: _type_id,
                deregister_fn,
            } => deregister_fn(scene),
            SceneCommand::AddComponent { entity, component } => {
                scene.add_with_components_immediate(entity, vec![component]);
            }
            SceneCommand::RemoveComponent {
                entity,
                type_id: _type_id,
                remove_fn,
            } => remove_fn(scene, entity),
            SceneCommand::DeleteEntitiesWith(type_ids) => scene.delete_entities_with_immediate(type_ids),
            SceneCommand::RegisterPrefab { name, factory } => {
                scene.register_prefab_immediate(&name, factory)
            }
            SceneCommand::SpawnPrefab(name) => {
                let _ = scene.spawn_prefab_immediate(&name);
            }
            SceneCommand::SpawnBundle { components } => {
                let _ = scene.spawn_with_components_immediate(components);
            }
            SceneCommand::SpawnBundleBatch { bundles } => {
                for components in bundles {
                    let _ = scene.spawn_with_components_immediate(components);
                }
            }
            SceneCommand::AddBundle { entity, components } => {
                scene.add_with_components_immediate(entity, components);
            }
        }
    }
}

fn register_component_impl<C: Component>(scene: &mut Scene) {
    scene.register_component_immediate::<C>();
}

fn deregister_component_impl<C: Component>(scene: &mut Scene) {
    scene.deregister_component_immediate::<C>();
}

fn remove_component_impl<C: Component>(scene: &mut Scene, entity: Entity) {
    scene.remove_component_immediate::<C>(entity);
}
