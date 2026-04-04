use super::iterators::RowAccess;
use super::*;

macro_rules! define_tuple_types_arity {
    (
        $builder:ident,
        $iter:ident,
        $access:ident,
        $builder_mut:ident,
        $iter_mut:ident,
        $access_mut:ident,
        $first_ty:ident,
        $first_col:ident,
        $($ty:ident, $idx:ident, $col:ident),+
    ) => {
        pub(super) struct $access {
            pub(super) entities: *const Entity,
            pub(super) scene: *const Scene,
            pub(super) $first_col: *const comet_structs::Column,
            $(pub(super) $col: *const comet_structs::Column,)+
            pub(super) len: usize,
            pub(super) row: usize,
        }

        impl RowAccess for $access {
            fn len(&self) -> usize {
                self.len
            }

            fn row_mut(&mut self) -> &mut usize {
                &mut self.row
            }
        }

        pub(super) struct $access_mut {
            pub(super) entities: *const Entity,
            pub(super) $first_col: *mut comet_structs::Column,
            $(pub(super) $col: *mut comet_structs::Column,)+
            pub(super) scene: *mut Scene,
            pub(super) len: usize,
            pub(super) row: usize,
        }

        impl RowAccess for $access_mut {
            fn len(&self) -> usize {
                self.len
            }

            fn row_mut(&mut self) -> &mut usize {
                &mut self.row
            }
        }

        pub struct $iter<'a, $first_ty, $($ty),+> {
            pub(super) accesses: Vec<$access>,
            pub(super) idx: usize,
            pub(super) added_since_filters: Vec<(TypeId, Tick)>,
            pub(super) changed_since_filters: Vec<(TypeId, Tick)>,
            pub(super) _marker: PhantomData<(&'a (), $first_ty, $($ty),+)>,
        }

        pub struct $iter_mut<'a, $first_ty, $($ty),+> {
            pub(super) accesses: Vec<$access_mut>,
            pub(super) idx: usize,
            pub(super) added_since_filters: Vec<(TypeId, Tick)>,
            pub(super) changed_since_filters: Vec<(TypeId, Tick)>,
            pub(super) _marker: PhantomData<(&'a (), $first_ty, $($ty),+)>,
        }

        pub struct $builder<'a, $first_ty, $($ty),+, Filters = ()> {
            pub(super) scene: &'a Scene,
            pub(super) state: QueryFilterState,
            pub(super) _marker: PhantomData<($first_ty, $($ty),+, Filters)>,
        }

        pub struct $builder_mut<'a, $first_ty, $($ty),+, Filters = ()> {
            pub(super) scene: &'a Scene,
            pub(super) state: QueryFilterState,
            pub(super) _marker: PhantomData<($first_ty, $($ty),+, Filters)>,
        }

        impl<'a, $first_ty, $($ty),+, Filters> $builder<'a, $first_ty, $($ty),+, Filters> {
            pub(super) fn from_state(scene: &'a Scene, state: QueryFilterState) -> Self {
                Self {
                    scene,
                    state,
                    _marker: PhantomData,
                }
            }
        }

        impl<'a, $first_ty, $($ty),+, Filters> $builder_mut<'a, $first_ty, $($ty),+, Filters> {
            pub(super) fn from_state(scene: &'a Scene, state: QueryFilterState) -> Self {
                Self {
                    scene,
                    state,
                    _marker: PhantomData,
                }
            }
        }
    };
}

macro_rules! define_entity_tuple_types_arity {
    (
        $builder:ident,
        $iter:ident,
        $access:ident,
        $builder_mut:ident,
        $iter_mut:ident,
        $access_mut:ident,
        $first_ty:ident,
        $first_col:ident
        $(,
            $ty:ident,
            $idx:ident,
            $col:ident
        )*
    ) => {
        pub(super) struct $access {
            pub(super) entities: *const Entity,
            pub(super) scene: *const Scene,
            pub(super) $first_col: *const comet_structs::Column,
            $(pub(super) $col: *const comet_structs::Column,)*
            pub(super) len: usize,
            pub(super) row: usize,
        }

        impl RowAccess for $access {
            fn len(&self) -> usize {
                self.len
            }

            fn row_mut(&mut self) -> &mut usize {
                &mut self.row
            }
        }

        pub(super) struct $access_mut {
            pub(super) entities: *const Entity,
            pub(super) $first_col: *mut comet_structs::Column,
            $(pub(super) $col: *mut comet_structs::Column,)*
            pub(super) scene: *mut Scene,
            pub(super) len: usize,
            pub(super) row: usize,
        }

        impl RowAccess for $access_mut {
            fn len(&self) -> usize {
                self.len
            }

            fn row_mut(&mut self) -> &mut usize {
                &mut self.row
            }
        }

        pub struct $iter<'a, $first_ty $(, $ty)*> {
            pub(super) accesses: Vec<$access>,
            pub(super) idx: usize,
            pub(super) added_since_filters: Vec<(TypeId, Tick)>,
            pub(super) changed_since_filters: Vec<(TypeId, Tick)>,
            pub(super) _marker: PhantomData<(&'a (), $first_ty $(, $ty)*)>,
        }

        pub struct $iter_mut<'a, $first_ty $(, $ty)*> {
            pub(super) accesses: Vec<$access_mut>,
            pub(super) idx: usize,
            pub(super) added_since_filters: Vec<(TypeId, Tick)>,
            pub(super) changed_since_filters: Vec<(TypeId, Tick)>,
            pub(super) _marker: PhantomData<(&'a (), $first_ty $(, $ty)*)>,
        }

        pub struct $builder<'a, $first_ty $(, $ty)*, Filters = ()> {
            pub(super) scene: &'a Scene,
            pub(super) state: QueryFilterState,
            pub(super) _marker: PhantomData<($first_ty $(, $ty)*, Filters)>,
        }

        pub struct $builder_mut<'a, $first_ty $(, $ty)*, Filters = ()> {
            pub(super) scene: &'a Scene,
            pub(super) state: QueryFilterState,
            pub(super) _marker: PhantomData<($first_ty $(, $ty)*, Filters)>,
        }

        impl<'a, $first_ty $(, $ty)*, Filters> $builder<'a, $first_ty $(, $ty)*, Filters> {
            pub(super) fn from_state(scene: &'a Scene, state: QueryFilterState) -> Self {
                Self {
                    scene,
                    state,
                    _marker: PhantomData,
                }
            }
        }

        impl<'a, $first_ty $(, $ty)*, Filters> $builder_mut<'a, $first_ty $(, $ty)*, Filters> {
            pub(super) fn from_state(scene: &'a Scene, state: QueryFilterState) -> Self {
                Self {
                    scene,
                    state,
                    _marker: PhantomData,
                }
            }
        }
    };
}

for_each_tuple_arity!(define_tuple_types_arity);
for_each_entity_tuple_arity!(define_entity_tuple_types_arity);
