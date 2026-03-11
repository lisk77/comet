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
                    &component_types,
                    move |columns, column_indices, _row| {
                        let mut __bundle_col_i = 0usize;
                        $(
                            {
                                let col_idx = column_indices[__bundle_col_i];
                                __bundle_col_i += 1;
                                columns[col_idx].push::<$ty>(self.$field);
                            }
                        )*
                    },
                )
            }
        }
    };
}
