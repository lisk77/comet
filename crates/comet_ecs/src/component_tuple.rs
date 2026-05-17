use crate::{Bundle, Component, ErasedComponent, Scene, SceneCommands};
use comet_structs::Column;
use std::any::TypeId;

pub trait ComponentTuple {
    fn type_ids() -> Vec<TypeId>;
    fn register_all(scene: &mut Scene);
    fn deferred_register_all(commands: &mut SceneCommands);
}

impl ComponentTuple for () {
    fn type_ids() -> Vec<TypeId> {
        Vec::new()
    }

    fn register_all(_scene: &mut Scene) {}

    fn deferred_register_all(_commands: &mut SceneCommands) {}
}

impl<C: Component> ComponentTuple for C {
    fn type_ids() -> Vec<TypeId> {
        vec![C::type_id()]
    }

    fn register_all(scene: &mut Scene) {
        scene.register_component::<C>();
    }

    fn deferred_register_all(commands: &mut SceneCommands) {
        commands.register_component::<C>();
    }
}

impl Bundle for () {
    fn into_components(self) -> Vec<ErasedComponent> {
        Vec::new()
    }

    fn spawn(self, scene: &mut Scene) -> crate::Entity {
        scene.new_entity_immediate()
    }

    fn spawn_batch(scene: &mut Scene, bundles: Vec<Self>) -> Vec<crate::Entity> {
        bundles.into_iter().map(|_| scene.new_entity_immediate()).collect()
    }

    fn type_ids(&self) -> Vec<TypeId> {
        Vec::new()
    }

    fn write_components(self, _columns: &mut [Column], _column_indices: &[usize], _row: usize) {}
    fn write_components_reserved(self, _columns: &mut [Column], _column_indices: &[usize], _row: usize) {}
}

impl<C: Component> Bundle for C {
    fn into_components(self) -> Vec<ErasedComponent> {
        vec![ErasedComponent::new(self)]
    }

    fn type_ids(&self) -> Vec<TypeId> {
        vec![C::type_id()]
    }

    fn write_components(self, columns: &mut [Column], column_indices: &[usize], _row: usize) {
        let col_idx = column_indices[0];
        unsafe {
            columns[col_idx].push_unchecked::<C>(self);
        }
    }

    fn write_components_reserved(self, columns: &mut [Column], column_indices: &[usize], _row: usize) {
        let col_idx = column_indices[0];
        unsafe {
            columns[col_idx].push_unchecked_reserved::<C>(self);
        }
    }
}

macro_rules! impl_component_tuple {
    ($($name:ident),+ $(,)?) => {
        impl<$($name: Component),+> ComponentTuple for ($($name,)+) {
            fn type_ids() -> Vec<TypeId> {
                vec![$($name::type_id()),+]
            }

            fn register_all(scene: &mut Scene) {
                $(scene.register_component::<$name>();)+
            }

            fn deferred_register_all(commands: &mut SceneCommands) {
                $(commands.register_component::<$name>();)+
            }
        }

        impl<$($name: Component),+> Bundle for ($($name,)+) {
            #[allow(non_snake_case)]
            fn into_components(self) -> Vec<ErasedComponent> {
                let ($($name,)+) = self;
                vec![$(ErasedComponent::new($name)),+]
            }

            fn spawn(self, scene: &mut Scene) -> crate::Entity {
                let component_types = [$(std::any::TypeId::of::<$name>()),+];
                scene.__spawn_bundle_typed(
                    std::any::TypeId::of::<($($name,)+)>(),
                    &component_types,
                    move |columns, column_indices, row| {
                        self.write_components(columns, column_indices, row);
                    },
                )
            }

            fn spawn_batch(scene: &mut Scene, bundles: Vec<Self>) -> Vec<crate::Entity> {
                if bundles.is_empty() {
                    return Vec::new();
                }
                let component_types = [$(std::any::TypeId::of::<$name>()),+];
                scene.__spawn_bundle_typed_batch(
                    std::any::TypeId::of::<($($name,)+)>(),
                    &component_types,
                    bundles,
                    |columns, column_indices, row, bundle| {
                        bundle.write_components_reserved(columns, column_indices, row);
                    },
                )
            }

            fn type_ids(&self) -> Vec<TypeId> {
                vec![$(std::any::TypeId::of::<$name>()),+]
            }

            #[allow(non_snake_case, unused_assignments)]
            fn write_components(self, columns: &mut [Column], column_indices: &[usize], _row: usize) {
                let ($($name,)+) = self;
                let mut col_i = 0usize;
                $(
                    {
                        let col_idx = column_indices[col_i];
                        col_i += 1;
                        unsafe {
                            columns[col_idx].push_unchecked::<$name>($name);
                        }
                    }
                )+
            }

            #[allow(non_snake_case, unused_assignments)]
            fn write_components_reserved(self, columns: &mut [Column], column_indices: &[usize], _row: usize) {
                let ($($name,)+) = self;
                let mut col_i = 0usize;
                $(
                    {
                        let col_idx = column_indices[col_i];
                        col_i += 1;
                        unsafe {
                            columns[col_idx].push_unchecked_reserved::<$name>($name);
                        }
                    }
                )+
            }
        }
    };
}

impl_component_tuple!(A);
impl_component_tuple!(A, B);
impl_component_tuple!(A, B, C);
impl_component_tuple!(A, B, C, D);
impl_component_tuple!(A, B, C, D, E);
impl_component_tuple!(A, B, C, D, E, F);
impl_component_tuple!(A, B, C, D, E, F, G);
impl_component_tuple!(A, B, C, D, E, F, G, H);
