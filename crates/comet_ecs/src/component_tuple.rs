use crate::bundles::BundleInfo;
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

impl BundleInfo for () {
    fn component_type_ids() -> Vec<TypeId> {
        Vec::new()
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

impl<C: Component> BundleInfo for C {
    fn component_type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<C>()]
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

        impl<$($name: Bundle),+> Bundle for ($($name,)+) {
            #[allow(non_snake_case)]
            fn into_components(self) -> Vec<ErasedComponent> {
                let ($($name,)+) = self;
                let mut components = Vec::new();
                $(components.extend($name.into_components());)+
                components
            }

            fn try_spawn(
                self,
                scene: &mut Scene,
            ) -> Result<crate::Entity, EcsError> {
                self.ensure_registered(scene);
                let component_types = self.type_ids();
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


            #[allow(non_snake_case)]
            fn type_ids(&self) -> Vec<TypeId> {
                let ($($name,)+) = self;
                let mut type_ids = Vec::new();
                $(type_ids.extend($name.type_ids());)+
                type_ids
            }

            #[allow(non_snake_case)]
            fn ensure_registered(&self, scene: &mut Scene) {
                let ($($name,)+) = self;
                $($name.ensure_registered(scene);)+
            }

            #[allow(non_snake_case, unused_assignments)]
            fn write_components(self, columns: &mut [Column], column_indices: &[usize], row: usize) {
                let ($($name,)+) = self;
                let mut offset = 0usize;
                $(
                    let width = $name.type_ids().len();
                    let end = offset + width;
                    $name.write_components(columns, &column_indices[offset..end], row);
                    offset = end;
                )+
            }

            #[allow(non_snake_case, unused_assignments)]
            fn write_components_reserved(self, columns: &mut [Column], column_indices: &[usize], row: usize) {
                let ($($name,)+) = self;
                let mut offset = 0usize;
                $(
                    let width = $name.type_ids().len();
                    let end = offset + width;
                    $name.write_components_reserved(columns, &column_indices[offset..end], row);
                    offset = end;
                )+
            }
        }

        impl<$($name: BundleInfo),+> BundleInfo for ($($name,)+) {
            fn component_type_ids() -> Vec<TypeId> {
                let mut type_ids = Vec::new();
                $(type_ids.extend($name::component_type_ids());)+
                type_ids
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
