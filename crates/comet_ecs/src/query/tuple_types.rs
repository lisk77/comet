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
            pub(super) $first_col: *const comet_structs::Column,
            $(pub(super) $col: *const comet_structs::Column,)+
            pub(super) len: usize,
            pub(super) row: usize,
        }

        pub(super) struct $access_mut {
            pub(super) $first_col: *mut comet_structs::Column,
            $(pub(super) $col: *mut comet_structs::Column,)+
            pub(super) len: usize,
            pub(super) row: usize,
        }

        pub struct $iter<'a, $first_ty: Component, $($ty: Component),+> {
            pub(super) accesses: Vec<$access>,
            pub(super) idx: usize,
            pub(super) _marker: PhantomData<(&'a $first_ty, $(&'a $ty),+)>,
        }

        pub struct $iter_mut<'a, $first_ty: Component, $($ty: Component),+> {
            pub(super) accesses: Vec<$access_mut>,
            pub(super) idx: usize,
            pub(super) _marker: PhantomData<(&'a mut $first_ty, $(&'a mut $ty),+)>,
        }

        pub struct $builder<'a, $first_ty: Component, $($ty: Component),+> {
            pub(super) scene: &'a Scene,
            pub(super) tags: Vec<TypeId>,
            pub(super) _marker: PhantomData<(&'a $first_ty, $(&'a $ty),+)>,
        }

        pub struct $builder_mut<'a, $first_ty: Component, $($ty: Component),+> {
            pub(super) scene: &'a mut Scene,
            pub(super) tags: Vec<TypeId>,
            pub(super) _marker: PhantomData<(&'a mut $first_ty, $(&'a mut $ty),+)>,
        }
    };
}

for_each_tuple_arity!(define_tuple_types_arity);
