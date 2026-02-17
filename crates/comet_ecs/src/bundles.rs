use crate::{EntityId, Scene};

pub trait Bundle {
    fn insert(self, scene: &mut Scene, entity: EntityId);
}

#[macro_export]
macro_rules! bundle {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        pub struct $name {
            $(pub $field: $ty,)*
        }

        impl $crate::Bundle for $name {
            fn insert(self, scene: &mut $crate::Scene, entity: $crate::EntityId) {
                $(scene.add_component(entity, self.$field);)*
            }
        }
    };
}
