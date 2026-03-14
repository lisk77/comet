use super::*;
use crate::query::tuple_types::*;

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
            impl_query_state_methods_scene_ref!($first_ty);

            pub fn iter(self) -> $iter<'a, $first_ty $(, $ty)*> {
                let mut accesses = Vec::new();
                let required = [$first_ty::type_id(), $($ty::type_id()),+];
                for_each_matching_archetype(self.scene, &self.state, $first_ty::type_id(), &required, |scene_ref, arch_id, first_idx| {
                    let arch = scene_ref.archetypes().get(arch_id);
                    $(let Some($idx) = arch.column_index($ty::type_id()) else {
                        return;
                    };)+
                    let cols = arch.columns();
                    let $first_col = &cols[first_idx] as *const _;
                    $(let $col = &cols[$idx] as *const _;)+
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
                    added_filter: self.state.added_filter,
                    changed_filter: self.state.changed_filter,
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
            impl_query_state_methods_scene_ref!($first_ty);

            pub fn iter(self) -> $iter_mut<'a, $first_ty, $($ty),+> {
                let mut accesses = Vec::new();
                let required = [$first_ty::type_id(), $($ty::type_id()),+];
                for_each_matching_archetype_mut(self.scene, &self.state, $first_ty::type_id(), &required, |scene_ref, arch_id, first_idx| {
                    let arch = scene_ref.archetypes_mut().get_mut(arch_id);
                    $(let Some($idx) = arch.column_index($ty::type_id()) else {
                        return;
                    };)+
                    let len = arch.len();
                    let cols = arch.columns_mut();
                    let $first_col = &mut cols[first_idx] as *mut _;
                    $(let $col = &mut cols[$idx] as *mut _;)+
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
                    added_filter: self.state.added_filter,
                    changed_filter: self.state.changed_filter,
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
            impl_query_state_methods_scene_ref!($first_ty);

            pub fn iter(self) -> $iter<'a, $first_ty $(, $ty)*> {
                let mut accesses = Vec::new();
                let required = [$first_ty::type_id() $(, $ty::type_id())*];
                for_each_matching_archetype(self.scene, &self.state, $first_ty::type_id(), &required, |scene_ref, arch_id, first_idx| {
                    let arch = scene_ref.archetypes().get(arch_id);
                    $(let Some($idx) = arch.column_index($ty::type_id()) else {
                        return;
                    };)*
                    let cols = arch.columns();
                    let $first_col = &cols[first_idx] as *const _;
                    $(let $col = &cols[$idx] as *const _;)*
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
                    added_filter: self.state.added_filter,
                    changed_filter: self.state.changed_filter,
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
            impl_query_state_methods_scene_ref!($first_ty);

            pub fn iter(self) -> $iter_mut<'a, $first_ty $(, $ty)*> {
                let mut accesses = Vec::new();
                let required = [$first_ty::type_id() $(, $ty::type_id())*];
                for_each_matching_archetype_mut(self.scene, &self.state, $first_ty::type_id(), &required, |scene_ref, arch_id, first_idx| {
                    let arch = scene_ref.archetypes_mut().get_mut(arch_id);
                    $(let Some($idx) = arch.column_index($ty::type_id()) else {
                        return;
                    };)*
                    let len = arch.len();
                    let cols = arch.columns_mut();
                    let $first_col = &mut cols[first_idx] as *mut _;
                    $(let $col = &mut cols[$idx] as *mut _;)*
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
                    added_filter: self.state.added_filter,
                    changed_filter: self.state.changed_filter,
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
