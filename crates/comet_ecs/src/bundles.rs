use std::any::TypeId;

use comet_structs::Column;

use crate::{EcsError, ErasedComponent, Scene};
pub trait Bundle: 'static {
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

#[doc(hidden)]
pub trait BundleInfo: Bundle {
    fn component_type_ids() -> Vec<TypeId>;
}

#[macro_export]
macro_rules! bundle {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        pub struct $name {
            $(pub $field: $ty,)*
        }

        impl $crate::Bundle for $name {
            fn into_components(self) -> Vec<$crate::ErasedComponent> {
                let mut components = Vec::new();
                $(components.extend($crate::Bundle::into_components(self.$field));)*
                components
            }

            fn type_ids(&self) -> Vec<std::any::TypeId> {
                let mut type_ids = Vec::new();
                $(type_ids.extend($crate::Bundle::type_ids(&self.$field));)*
                type_ids
            }

            fn ensure_registered(&self, scene: &mut $crate::Scene) {
                $($crate::Bundle::ensure_registered(&self.$field, scene);)*
            }

            fn try_spawn(
                self,
                scene: &mut $crate::Scene,
            ) -> Result<$crate::Entity, $crate::EcsError> {
                self.ensure_registered(scene);
                let component_types = self.type_ids();
                if scene.__bundle_has_required_components(&component_types) {
                    return scene.try_spawn_with_components(self.into_components());
                }
                scene.__try_spawn_bundle_typed(
                    std::any::TypeId::of::<$name>(),
                    &component_types,
                    move |columns, column_indices, row| {
                        $crate::Bundle::write_components(self, columns, column_indices, row);
                    },
                )
            }

            fn write_components(
                self,
                columns: &mut [$crate::__private::Column],
                column_indices: &[usize],
                row: usize,
            ) {
                let mut offset = 0usize;
                $(
                    let width = $crate::Bundle::type_ids(&self.$field).len();
                    let end = offset + width;
                    $crate::Bundle::write_components(
                        self.$field,
                        columns,
                        &column_indices[offset..end],
                        row,
                    );
                    offset = end;
                )*
            }

            fn write_components_reserved(
                self,
                columns: &mut [$crate::__private::Column],
                column_indices: &[usize],
                row: usize,
            ) {
                let mut offset = 0usize;
                $(
                    let width = $crate::Bundle::type_ids(&self.$field).len();
                    let end = offset + width;
                    $crate::Bundle::write_components_reserved(
                        self.$field,
                        columns,
                        &column_indices[offset..end],
                        row,
                    );
                    offset = end;
                )*
            }
        }

        impl $crate::__private::BundleInfo for $name {
            fn component_type_ids() -> Vec<std::any::TypeId> {
                let mut type_ids = Vec::new();
                $(type_ids.extend(<$ty as $crate::__private::BundleInfo>::component_type_ids());)*
                type_ids
            }
        }
    };
}
