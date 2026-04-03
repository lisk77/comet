use comet_structs::{Column, FlatMap};
use std::any::{Any, TypeId};

pub struct ErasedComponent {
    pub(crate) type_id: TypeId,
    pub(crate) push_fn: fn(Box<dyn Any + Send>, &mut Column),
    pub(crate) set_fn: fn(Box<dyn Any + Send>, &mut Column, usize),
    pub(crate) value: Box<dyn Any + Send>,
}

impl ErasedComponent {
    pub fn new<C: crate::Component + 'static>(value: C) -> Self {
        fn push<C: crate::Component + 'static>(value: Box<dyn Any + Send>, column: &mut Column) {
            let value = *value
                .downcast::<C>()
                .expect("ErasedComponent type mismatch");
            column.push::<C>(value);
        }

        fn set<C: crate::Component + 'static>(
            value: Box<dyn Any + Send>,
            column: &mut Column,
            row: usize,
        ) {
            let value = *value
                .downcast::<C>()
                .expect("ErasedComponent type mismatch");
            let _ = column.set::<C>(row, value);
        }

        Self {
            type_id: C::type_id(),
            push_fn: push::<C>,
            set_fn: set::<C>,
            value: Box::new(value),
        }
    }
}

pub type PrefabFactory = fn(&mut crate::Scene) -> crate::Entity;

pub(crate) struct PrefabManager {
    pub(crate) prefabs: FlatMap<String, PrefabFactory>,
}

impl PrefabManager {
    pub fn new() -> Self {
        Self {
            prefabs: FlatMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, factory: PrefabFactory) {
        self.prefabs.insert(name.to_string(), factory);
    }

    pub fn has_prefab(&self, name: &str) -> bool {
        self.prefabs.contains(&name.to_string())
    }
}

#[macro_export]
macro_rules! register_prefab {
    ($scene:expr, $name:expr, $($component:expr),* $(,)?) => {
        {
            fn prefab_factory(scene: &mut $crate::Scene) -> $crate::Entity {
                scene.spawn_with_components(vec![
                    $(
                        $crate::prefabs::ErasedComponent::new($component),
                    )*
                ])
            }
            $scene.register_prefab($name, prefab_factory);
        }
    };
}

#[macro_export]
macro_rules! spawn_prefab {
    ($scene:expr, $name:expr) => {
        $scene.spawn_prefab($name)
    };
}
