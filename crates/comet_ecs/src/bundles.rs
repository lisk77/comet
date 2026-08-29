use std::any::TypeId;

use comet_structs::Column;

use crate::{EcsError, ErasedComponent, Scene};
pub trait Bundle {
    fn into_components(self) -> Vec<ErasedComponent>;

    fn ensure_registered(&self, scene: &mut Scene);

    fn try_spawn(self, scene: &mut Scene) -> Result<crate::Entity, EcsError>
    where
        Self: Sized,
    {
        self.ensure_registered(scene);
        scene.try_spawn_with_components(self.into_components())
    }

    fn spawn(self, scene: &mut Scene) -> crate::Entity
    where
        Self: Sized,
    {
        self.try_spawn(scene)
            .unwrap_or_else(|error| comet_log::fatal!("{}", error))
    }

    fn insert(self, scene: &mut Scene, entity: crate::Entity)
    where
        Self: Sized,
    {
        scene.add_with_components(entity, self.into_components());
    }

    fn type_ids(&self) -> Vec<TypeId>;

    fn write_components(self, columns: &mut [Column], column_indices: &[usize], _row: usize)
    where
        Self: Sized,
    {
        for (i, component) in self.into_components().into_iter().enumerate() {
            component.push(&mut columns[column_indices[i]]);
        }
    }

    fn write_components_reserved(self, columns: &mut [Column], column_indices: &[usize], row: usize)
    where
        Self: Sized,
    {
        self.write_components(columns, column_indices, row);
    }
}

#[macro_export]
macro_rules! bundle {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        pub struct $name {
            $(pub $field: $ty,)*
        }

        impl $crate::Bundle for $name {
            fn into_components(self) -> Vec<$crate::ErasedComponent> {
                vec![
                    $(
                        $crate::ErasedComponent::new(self.$field),
                    )*
                ]
            }

            fn type_ids(&self) -> Vec<std::any::TypeId> {
                vec![$(std::any::TypeId::of::<$ty>()),*]
            }

            fn ensure_registered(&self, scene: &mut $crate::Scene) {
                $(scene.ensure_component::<$ty>();)*
            }

            fn try_spawn(
                self,
                scene: &mut $crate::Scene,
            ) -> Result<$crate::Entity, $crate::EcsError> {
                self.ensure_registered(scene);
                let component_types = [
                    $(
                        std::any::TypeId::of::<$ty>(),
                    )*
                ];
                if scene.__bundle_has_required_components(&component_types) {
                    return scene.try_spawn_with_components(self.into_components());
                }
                scene.__try_spawn_bundle_typed(
                    std::any::TypeId::of::<$name>(),
                    &component_types,
                    move |columns, column_indices, _row| {
                        let mut __bundle_col_i = 0usize;
                        $(
                            {
                                let col_idx = column_indices[__bundle_col_i];
                                __bundle_col_i += 1;
                                unsafe {
                                    columns[col_idx].push_unchecked::<$ty>(self.$field);
                                }
                            }
                        )*
                    },
                )
            }

        }
    };
}
