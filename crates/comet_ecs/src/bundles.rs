use crate::{ErasedComponent, Scene};

pub trait Bundle {
    fn into_components(self) -> Vec<ErasedComponent>;

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
        }
    };
}
