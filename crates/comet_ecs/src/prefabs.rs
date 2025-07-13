use comet_structs::FlatMap;

pub type PrefabFactory = fn(&mut crate::Scene) -> usize;

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
            fn prefab_factory(scene: &mut $crate::Scene) -> usize {
                let entity = scene.new_entity() as usize;
                $(
                    scene.add_component(entity, $component);
                )*
                entity
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
