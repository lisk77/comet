use comet_structs::{Column, FlatMap};
use std::any::TypeId;

trait ErasedComponentValue: Send {
    fn push(self: Box<Self>, column: &mut Column);
    fn set(self: Box<Self>, column: &mut Column, row: usize);
}

struct ComponentValue<C: crate::Component>(C);

impl<C: crate::Component> ErasedComponentValue for ComponentValue<C> {
    fn push(self: Box<Self>, column: &mut Column) {
        column.push::<C>(self.0);
    }

    fn set(self: Box<Self>, column: &mut Column, row: usize) {
        let _ = column.set::<C>(row, self.0);
    }
}

pub struct ErasedComponent {
    pub(crate) type_id: TypeId,
    pub(crate) register_fn: fn(&mut crate::Scene),
    value: Box<dyn ErasedComponentValue>,
}

impl ErasedComponent {
    pub fn new<C: crate::Component>(value: C) -> Self {
        fn register<C: crate::Component>(scene: &mut crate::Scene) {
            scene.ensure_component::<C>();
        }

        Self {
            type_id: TypeId::of::<C>(),
            register_fn: register::<C>,
            value: Box::new(ComponentValue(value)),
        }
    }

    pub(crate) fn push(self, column: &mut Column) {
        self.value.push(column);
    }

    pub(crate) fn set(self, column: &mut Column, row: usize) {
        self.value.set(column, row);
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
