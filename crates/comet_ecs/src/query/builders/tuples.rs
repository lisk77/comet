use super::*;
use crate::query::tuple_types::*;
use std::ptr;

macro_rules! impl_tuple_builders_arity {
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
        impl<'a, $first_ty: ReadFetch<'a> + 'a, $($ty: ReadFetch<'a> + 'a),+, Filters: QueryFilterSet> QuerySpec<'a> for crate::query::QueryParam<($first_ty, $($ty,)+), Filters> {
            type Builder = $builder<'a, $first_ty, $($ty),+, Filters>;

            fn build(scene: &'a Scene) -> Self::Builder {
                $builder::from_state(scene, typed_filters::<Filters>(scene))
            }
        }

        impl<'a, $first_ty: ReadFetch<'a> + 'a, $($ty: ReadFetch<'a> + 'a),+> QuerySpec<'a> for ($first_ty, $($ty,)+) {
            type Builder = $builder<'a, $first_ty, $($ty),+, ()>;

            fn build(scene: &'a Scene) -> Self::Builder {
                $builder::from_state(scene, QueryFilterState::empty())
            }
        }

        impl<'a, $first_ty: WriteFetch<'a> + 'a, $($ty: WriteFetch<'a> + 'a),+, Filters: QueryFilterSet> QuerySpecMut<'a> for crate::query::QueryParam<($first_ty, $($ty,)+), Filters> {
            type Builder = $builder_mut<'a, $first_ty, $($ty),+, Filters>;

            fn build(scene: &'a mut Scene) -> Self::Builder {
                $builder_mut::from_state(scene, typed_filters::<Filters>(scene))
            }
        }

        impl<'a, $first_ty: WriteFetch<'a> + 'a, $($ty: WriteFetch<'a> + 'a),+> QuerySpecMut<'a> for ($first_ty, $($ty,)+) {
            type Builder = $builder_mut<'a, $first_ty, $($ty),+, ()>;

            fn build(scene: &'a mut Scene) -> Self::Builder {
                $builder_mut::from_state(scene, QueryFilterState::empty())
            }
        }

        impl<'a, $first_ty: ReadFetch<'a> + 'a, $($ty: ReadFetch<'a> + 'a),+, Filters> $builder<'a, $first_ty, $($ty),+, Filters> {
            impl_query_state_methods_scene_ref!();

            pub fn iter(self) -> $iter<'a, $first_ty $(, $ty)*> {
                assert!(
                    $first_ty::required(),
                    "the first tuple query fetch cannot be optional"
                );
                let mut accesses = Vec::new();
                #[allow(unused_mut)]
                let mut required = vec![$first_ty::type_id()];
                $(
                    if $ty::required() {
                        required.push($ty::type_id());
                    }
                )+
                for_each_matching_archetype(self.scene, &self.state, $first_ty::type_id(), &required, |scene_ref, arch_id, first_idx| {
                    let arch = scene_ref.archetypes().get(arch_id);
                    let cols = arch.columns();
                    let $first_col = &cols[first_idx] as *const _;
                    $(
                        let $col = match arch.column_index($ty::type_id()) {
                            Some($idx) => &cols[$idx] as *const _,
                            None if !$ty::required() => ptr::null(),
                            None => return,
                        };
                    )+
                    let entities = arch.entities().as_ptr();
                    let scene = scene_ref as *const Scene;
                    accesses.push($access {
                        entities,
                        scene,
                        $first_col,
                        $($col,)+
                        len: arch.len(),
                        row: 0,
                    });
                });

                $iter {
                    accesses,
                    idx: 0,
                    added_since_filters: self.state.added_since_filters,
                    changed_since_filters: self.state.changed_since_filters,
                    _marker: PhantomData,
                }
            }

            pub fn for_each(self, mut f: impl FnMut(($first_ty::Item, $($ty::Item),+))) {
                let mut iter = self.iter();
                while let Some(item) = iter.next() {
                    f(item);
                }
            }
        }

        impl<'a, $first_ty: WriteFetch<'a> + 'a, $($ty: WriteFetch<'a> + 'a),+, Filters> $builder_mut<'a, $first_ty, $($ty),+, Filters> {
            impl_query_state_methods_scene_mut!();

            pub fn iter(self) -> $iter_mut<'a, $first_ty, $($ty),+> {
                assert!(
                    $first_ty::required(),
                    "the first tuple query fetch cannot be optional"
                );
                let mut accesses = Vec::new();
                #[allow(unused_mut)]
                let mut required = vec![$first_ty::type_id()];
                $(
                    if $ty::required() {
                        required.push($ty::type_id());
                    }
                )+
                for_each_matching_archetype_mut(self.scene, &self.state, $first_ty::type_id(), &required, |scene_ref, arch_id, first_idx| {
                    let arch = scene_ref.archetypes_mut().get_mut(arch_id);
                    $(
                        let $idx = match arch.column_index($ty::type_id()) {
                            Some(idx) => Some(idx),
                            None if !$ty::required() => None,
                            None => return,
                        };
                    )+
                    let len = arch.len();
                    let cols = arch.columns_mut();
                    let $first_col = &mut cols[first_idx] as *mut _;
                    $(
                        let $col = match $idx {
                            Some(idx) => &mut cols[idx] as *mut _,
                            None => ptr::null_mut(),
                        };
                    )+
                    let entities = arch.entities().as_ptr();
                    let scene = scene_ref as *mut Scene;
                    accesses.push($access_mut {
                        entities,
                        $first_col,
                        $($col,)+
                        scene,
                        len,
                        row: 0,
                    });
                });

                $iter_mut {
                    accesses,
                    idx: 0,
                    added_since_filters: self.state.added_since_filters,
                    changed_since_filters: self.state.changed_since_filters,
                    _marker: PhantomData,
                }
            }

            pub fn for_each(self, mut f: impl FnMut(($first_ty::Item, $($ty::Item),+))) {
                let mut iter = self.iter();
                while let Some(item) = iter.next() {
                    f(item);
                }
            }
        }
    };
}

macro_rules! impl_entity_tuple_builders_arity {
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
        impl<'a, $first_ty: ReadFetch<'a> + 'a $(, $ty: ReadFetch<'a> + 'a)*, Filters: QueryFilterSet> QuerySpec<'a> for crate::query::QueryParam<(Entity, $first_ty $(, $ty)*), Filters> {
            type Builder = $builder<'a, $first_ty $(, $ty)*, Filters>;

            fn build(scene: &'a Scene) -> Self::Builder {
                $builder::from_state(scene, typed_filters::<Filters>(scene))
            }
        }

        impl<'a, $first_ty: ReadFetch<'a> + 'a $(, $ty: ReadFetch<'a> + 'a)*> QuerySpec<'a> for (Entity, $first_ty $(, $ty)*) {
            type Builder = $builder<'a, $first_ty $(, $ty)*, ()>;

            fn build(scene: &'a Scene) -> Self::Builder {
                $builder::from_state(scene, QueryFilterState::empty())
            }
        }

        impl<'a, $first_ty: WriteFetch<'a> + 'a $(, $ty: WriteFetch<'a> + 'a)*, Filters: QueryFilterSet> QuerySpecMut<'a> for crate::query::QueryParam<(Entity, $first_ty $(, $ty)*), Filters> {
            type Builder = $builder_mut<'a, $first_ty $(, $ty)*, Filters>;

            fn build(scene: &'a mut Scene) -> Self::Builder {
                $builder_mut::from_state(scene, typed_filters::<Filters>(scene))
            }
        }

        impl<'a, $first_ty: WriteFetch<'a> + 'a $(, $ty: WriteFetch<'a> + 'a)*> QuerySpecMut<'a> for (Entity, $first_ty $(, $ty)*) {
            type Builder = $builder_mut<'a, $first_ty $(, $ty)*, ()>;

            fn build(scene: &'a mut Scene) -> Self::Builder {
                $builder_mut::from_state(scene, QueryFilterState::empty())
            }
        }

        impl<'a, $first_ty: ReadFetch<'a> + 'a $(, $ty: ReadFetch<'a> + 'a)*, Filters> $builder<'a, $first_ty $(, $ty)*, Filters> {
            impl_query_state_methods_scene_ref!();

            pub fn iter(self) -> $iter<'a, $first_ty $(, $ty)*> {
                assert!(
                    $first_ty::required(),
                    "the first entity-tuple query fetch cannot be optional"
                );
                let mut accesses = Vec::new();
                #[allow(unused_mut)]
                let mut required = vec![$first_ty::type_id()];
                $(
                    if $ty::required() {
                        required.push($ty::type_id());
                    }
                )*
                for_each_matching_archetype(self.scene, &self.state, $first_ty::type_id(), &required, |scene_ref, arch_id, first_idx| {
                    let arch = scene_ref.archetypes().get(arch_id);
                    let cols = arch.columns();
                    let $first_col = &cols[first_idx] as *const _;
                    $(
                        let $col = match arch.column_index($ty::type_id()) {
                            Some($idx) => &cols[$idx] as *const _,
                            None if !$ty::required() => ptr::null(),
                            None => return,
                        };
                    )*
                    let entities = arch.entities().as_ptr();
                    let scene = scene_ref as *const Scene;
                    accesses.push($access {
                        entities,
                        scene,
                        $first_col,
                        $($col,)*
                        len: arch.len(),
                        row: 0,
                    });
                });

                $iter {
                    accesses,
                    idx: 0,
                    added_since_filters: self.state.added_since_filters,
                    changed_since_filters: self.state.changed_since_filters,
                    _marker: PhantomData,
                }
            }

            pub fn for_each(self, mut f: impl FnMut((Entity, $first_ty::Item $(, $ty::Item)*))) {
                let mut iter = self.iter();
                while let Some(item) = iter.next() {
                    f(item);
                }
            }
        }

        impl<'a, $first_ty: WriteFetch<'a> + 'a $(, $ty: WriteFetch<'a> + 'a)*, Filters> $builder_mut<'a, $first_ty $(, $ty)*, Filters> {
            impl_query_state_methods_scene_mut!();

            pub fn iter(self) -> $iter_mut<'a, $first_ty $(, $ty)*> {
                assert!(
                    $first_ty::required(),
                    "the first entity-tuple query fetch cannot be optional"
                );
                let mut accesses = Vec::new();
                #[allow(unused_mut)]
                let mut required = vec![$first_ty::type_id()];
                $(
                    if $ty::required() {
                        required.push($ty::type_id());
                    }
                )*
                for_each_matching_archetype_mut(self.scene, &self.state, $first_ty::type_id(), &required, |scene_ref, arch_id, first_idx| {
                    let arch = scene_ref.archetypes_mut().get_mut(arch_id);
                    $(
                        let $idx = match arch.column_index($ty::type_id()) {
                            Some(idx) => Some(idx),
                            None if !$ty::required() => None,
                            None => return,
                        };
                    )*
                    let len = arch.len();
                    let cols = arch.columns_mut();
                    let $first_col = &mut cols[first_idx] as *mut _;
                    $(
                        let $col = match $idx {
                            Some(idx) => &mut cols[idx] as *mut _,
                            None => ptr::null_mut(),
                        };
                    )*
                    let entities = arch.entities().as_ptr();
                    let scene = scene_ref as *mut Scene;
                    accesses.push($access_mut {
                        entities,
                        $first_col,
                        $($col,)*
                        scene,
                        len,
                        row: 0,
                    });
                });

                $iter_mut {
                    accesses,
                    idx: 0,
                    added_since_filters: self.state.added_since_filters,
                    changed_since_filters: self.state.changed_since_filters,
                    _marker: PhantomData,
                }
            }

            pub fn for_each(self, mut f: impl FnMut((Entity, $first_ty::Item $(, $ty::Item)*))) {
                let mut iter = self.iter();
                while let Some(item) = iter.next() {
                    f(item);
                }
            }
        }
    };
}

for_each_tuple_arity!(impl_tuple_builders_arity);
for_each_entity_tuple_arity!(impl_entity_tuple_builders_arity);
