use super::super::tuple_types::*;
use super::*;

macro_rules! impl_tuple_iterators_arity {
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
        impl<'a, $first_ty: ReadFetch<'a>, $($ty: ReadFetch<'a>),+> Iterator for $iter<'a, $first_ty, $($ty),+> {
            type Item = ($first_ty::Item, $($ty::Item),+);

            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    let (access, row) = next_access_row(&mut self.accesses, &mut self.idx)?;
                    unsafe {
                        let entity = fetch_entity(access.entities, access.len, row)?;
                        if !matches_change_filters(
                            access.scene,
                            entity,
                            &self.added_since_filters,
                            &self.changed_since_filters,
                        ) {
                            continue;
                        }
                        return Some((
                            $first_ty::get(access.$first_col, row)?,
                            $($ty::get(access.$col, row)?,)+
                        ));
                    }
                }
            }
        }

        impl<'a, $first_ty: WriteFetch<'a>, $($ty: WriteFetch<'a>),+> Iterator for $iter_mut<'a, $first_ty, $($ty),+> {
            type Item = ($first_ty::Item, $($ty::Item),+);

            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    let (access, row) = next_access_row(&mut self.accesses, &mut self.idx)?;
                    unsafe {
                        let entity = fetch_entity(access.entities, access.len, row)?;
                        if !matches_change_filters(
                            access.scene,
                            entity,
                            &self.added_since_filters,
                            &self.changed_since_filters,
                        ) {
                            continue;
                        }
                        let first_item = $first_ty::get(access.$first_col, row)?;
                        $(let $col = $ty::get(access.$col, row)?;)+
                        if $first_ty::writes() || $($ty::writes())||+ {
                            if $first_ty::writes() && $first_ty::is_present(&first_item) {
                                (&mut *access.scene).mark_component_changed_for_query(entity, $first_ty::type_id());
                            }
                            $(
                                if $ty::writes() && $ty::is_present(&$col) {
                                    (&mut *access.scene).mark_component_changed_for_query(entity, $ty::type_id());
                                }
                            )+
                        }
                        return Some((first_item, $($col,)+));
                    }
                }
            }
        }
    };
}

macro_rules! impl_entity_tuple_iterators_arity {
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
        impl<'a, $first_ty: ReadFetch<'a> $(, $ty: ReadFetch<'a>)*> Iterator for $iter<'a, $first_ty $(, $ty)*> {
            type Item = (Entity, $first_ty::Item $(, $ty::Item)*);

            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    let (access, row) = next_access_row(&mut self.accesses, &mut self.idx)?;
                    unsafe {
                        let entity = fetch_entity(access.entities, access.len, row)?;
                        if !matches_change_filters(
                            access.scene,
                            entity,
                            &self.added_since_filters,
                            &self.changed_since_filters,
                        ) {
                            continue;
                        }
                        return Some((
                            entity,
                            $first_ty::get(access.$first_col, row)?,
                            $($ty::get(access.$col, row)?,)*
                        ));
                    }
                }
            }
        }

        impl<'a, $first_ty: WriteFetch<'a> $(, $ty: WriteFetch<'a>)*> Iterator for $iter_mut<'a, $first_ty $(, $ty)*> {
            type Item = (Entity, $first_ty::Item $(, $ty::Item)*);

            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    let (access, row) = next_access_row(&mut self.accesses, &mut self.idx)?;
                    unsafe {
                        let entity = fetch_entity(access.entities, access.len, row)?;
                        if !matches_change_filters(
                            access.scene,
                            entity,
                            &self.added_since_filters,
                            &self.changed_since_filters,
                        ) {
                            continue;
                        }
                        let first_item = $first_ty::get(access.$first_col, row)?;
                        $(let $col = $ty::get(access.$col, row)?;)*
                        if $first_ty::writes() && $first_ty::is_present(&first_item) {
                            (&mut *access.scene).mark_component_changed_for_query(entity, $first_ty::type_id());
                        }
                        $(
                            if $ty::writes() && $ty::is_present(&$col) {
                                (&mut *access.scene).mark_component_changed_for_query(entity, $ty::type_id());
                            }
                        )*
                        return Some((entity, first_item $(, $col)*));
                    }
                }
            }
        }
    };
}

for_each_tuple_arity!(impl_tuple_iterators_arity);
for_each_entity_tuple_arity!(impl_entity_tuple_iterators_arity);
