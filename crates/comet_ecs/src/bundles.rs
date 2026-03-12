use crate::{ErasedComponent, Scene};
pub trait Bundle {
    fn into_components(self) -> Vec<ErasedComponent>;

    fn spawn(self, scene: &mut Scene) -> crate::Entity
    where
        Self: Sized,
    {
        scene.spawn_with_components(self.into_components())
    }

    fn insert(self, scene: &mut Scene, entity: crate::Entity)
    where
        Self: Sized,
    {
        scene.add_with_components(entity, self.into_components());
    }

    fn spawn_batch(scene: &mut Scene, bundles: Vec<Self>) -> Vec<crate::Entity>
    where
        Self: Sized,
    {
        bundles.into_iter().map(|bundle| bundle.spawn(scene)).collect()
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

            fn spawn(self, scene: &mut $crate::Scene) -> $crate::Entity {
                let component_types = [
                    $(
                        std::any::TypeId::of::<$ty>(),
                    )*
                ];
                scene.__spawn_bundle_typed(
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

            fn spawn_batch(scene: &mut $crate::Scene, bundles: Vec<Self>) -> Vec<$crate::Entity> {
                let component_types = [
                    $(
                        std::any::TypeId::of::<$ty>(),
                    )*
                ];
                scene.__spawn_bundle_typed_batch(
                    std::any::TypeId::of::<$name>(),
                    &component_types,
                    bundles,
                    |columns, column_indices, _row, bundle| {
                        let mut __bundle_col_i = 0usize;
                        $(
                            {
                                let col_idx = column_indices[__bundle_col_i];
                                __bundle_col_i += 1;
                                unsafe {
                                    columns[col_idx].push_unchecked_reserved::<$ty>(bundle.$field);
                                }
                            }
                        )*
                    },
                )
            }
        }
    };
}
