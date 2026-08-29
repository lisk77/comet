use crate::{Bundle, Component, EcsError, ErasedComponent, Scene};
use comet_structs::Column;
use std::any::TypeId;

pub trait ComponentTuple {
    fn type_ids() -> Vec<TypeId>;
    fn ensure_all(scene: &mut Scene);
}

impl ComponentTuple for () {
    fn type_ids() -> Vec<TypeId> {
        Vec::new()
    }

    fn ensure_all(_scene: &mut Scene) {}
}

impl<C: Component> ComponentTuple for C {
    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<C>()]
    }

    fn ensure_all(scene: &mut Scene) {
        scene.ensure_component::<C>();
    }
}

impl Bundle for () {
    fn into_components(self) -> Vec<ErasedComponent> {
        Vec::new()
    }

    fn try_spawn(self, scene: &mut Scene) -> Result<crate::Entity, EcsError> {
        Ok(scene.new_entity_immediate())
    }

    fn type_ids(&self) -> Vec<TypeId> {
        Vec::new()
    }

    fn ensure_registered(&self, _scene: &mut Scene) {}

    fn write_components(self, _columns: &mut [Column], _column_indices: &[usize], _row: usize) {}
    fn write_components_reserved(
        self,
        _columns: &mut [Column],
        _column_indices: &[usize],
        _row: usize,
    ) {
    }
}

impl<C: Component> Bundle for C {
    fn into_components(self) -> Vec<ErasedComponent> {
        vec![ErasedComponent::new(self)]
    }

    fn type_ids(&self) -> Vec<TypeId> {
        vec![TypeId::of::<C>()]
    }

    fn ensure_registered(&self, scene: &mut Scene) {
        scene.ensure_component::<C>();
    }

    fn write_components(self, columns: &mut [Column], column_indices: &[usize], _row: usize) {
        let col_idx = column_indices[0];
        unsafe {
            columns[col_idx].push_unchecked::<C>(self);
        }
    }

    fn write_components_reserved(
        self,
        columns: &mut [Column],
        column_indices: &[usize],
        _row: usize,
    ) {
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
                vec![$(TypeId::of::<$name>()),+]
            }

            fn ensure_all(scene: &mut Scene) {
                $(scene.ensure_component::<$name>();)+
            }
        }

        impl<$($name: Component),+> Bundle for ($($name,)+) {
            #[allow(non_snake_case)]
            fn into_components(self) -> Vec<ErasedComponent> {
                let ($($name,)+) = self;
                vec![$(ErasedComponent::new($name)),+]
            }

            fn try_spawn(
                self,
                scene: &mut Scene,
            ) -> Result<crate::Entity, EcsError> {
                self.ensure_registered(scene);
                let component_types = [$(std::any::TypeId::of::<$name>()),+];
                if scene.__bundle_has_required_components(&component_types) {
                    return scene.try_spawn_with_components(self.into_components());
                }
                scene.__try_spawn_bundle_typed(
                    std::any::TypeId::of::<($($name,)+)>(),
                    &component_types,
                    move |columns, column_indices, row| {
                        self.write_components(columns, column_indices, row);
                    },
                )
            }


            fn type_ids(&self) -> Vec<TypeId> {
                vec![$(std::any::TypeId::of::<$name>()),+]
            }

            fn ensure_registered(&self, scene: &mut Scene) {
                $(scene.ensure_component::<$name>();)+
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
