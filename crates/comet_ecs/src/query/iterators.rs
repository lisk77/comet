use super::tuple_types::*;
use super::*;

impl<'a, P: ReadFetch<'a>> Iterator for QueryIter<'a, P> {
    type Item = P::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let access = self.accesses.get_mut(self.idx)?;
            if access.row >= access.len {
                self.idx += 1;
                continue;
            }
            let row = access.row;
            access.row += 1;
            if self.added_tick_filter.is_some() || self.changed_tick_filter.is_some() {
                let Some(entity) = (unsafe { <Entity as EntityFetch>::get(access.entities, access.len, row) }) else {
                    continue;
                };
                let scene = unsafe { &*access.scene };
                if let Some(tick) = self.added_tick_filter {
                    if !scene.component_added_since_type(entity, P::type_id(), tick) {
                        continue;
                    }
                }
                if let Some(tick) = self.changed_tick_filter {
                    if !scene.component_changed_since_type(entity, P::type_id(), tick) {
                        continue;
                    }
                }
            }
            unsafe { return P::get(access.col, row); }
        }
    }
}

impl<'a, P: WriteFetch<'a>> Iterator for QueryIterMut<'a, P> {
    type Item = P::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let access = self.accesses.get_mut(self.idx)?;
            if access.row >= access.len {
                self.idx += 1;
                continue;
            }
            let row = access.row;
            access.row += 1;
            unsafe {
                let entity = <Entity as EntityFetch>::get(access.entities, access.len, row)?;
                if let Some(tick) = self.added_tick_filter {
                    if !(&*access.scene).component_added_since_type(entity, P::type_id(), tick) {
                        continue;
                    }
                }
                if let Some(tick) = self.changed_tick_filter {
                    if !(&*access.scene).component_changed_since_type(entity, P::type_id(), tick) {
                        continue;
                    }
                }
                let item = P::get(access.col, row)?;
                if P::writes() {
                    (&mut *access.scene).mark_component_changed_for_query(entity, P::type_id());
                }
                return Some(item);
            }
        }
    }
}

impl<'a, P: ReadFetch<'a>, F> Iterator for QueryIterFiltered<'a, P, F>
where
    F: Fn(&P::Component) -> bool + 'a,
{
    type Item = P::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.inner.next()?;
            if (self.filter)(P::as_ref(&item)) {
                return Some(item);
            }
        }
    }
}

impl<'a, P: WriteFetch<'a>, F> Iterator for QueryIterMutFiltered<'a, P, F>
where
    F: Fn(&P::Component) -> bool + 'a,
{
    type Item = P::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.inner.next()?;
            if (self.filter)(P::as_ref(&item)) {
                return Some(item);
            }
        }
    }
}

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
                    let access = self.accesses.get_mut(self.idx)?;
                    if access.row >= access.len {
                        self.idx += 1;
                        continue;
                    }
                    let row = access.row;
                    access.row += 1;
                    unsafe {
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
                    let access = self.accesses.get_mut(self.idx)?;
                    if access.row >= access.len {
                        self.idx += 1;
                        continue;
                    }
                    let row = access.row;
                    access.row += 1;
                    unsafe {
                        let first_item = $first_ty::get(access.$first_col, row)?;
                        $(let $col = $ty::get(access.$col, row)?;)+
                        if $first_ty::writes() || $($ty::writes())||+ {
                            let entity = <Entity as EntityFetch>::get(access.entities, access.len, row)?;
                            if $first_ty::writes() {
                                (&mut *access.scene).mark_component_changed_for_query(entity, $first_ty::type_id());
                            }
                            $(
                                if $ty::writes() {
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
                    let access = self.accesses.get_mut(self.idx)?;
                    if access.row >= access.len {
                        self.idx += 1;
                        continue;
                    }
                    let row = access.row;
                    access.row += 1;
                    unsafe {
                        return Some((
                            <Entity as EntityFetch>::get(access.entities, access.len, row)?,
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
                    let access = self.accesses.get_mut(self.idx)?;
                    if access.row >= access.len {
                        self.idx += 1;
                        continue;
                    }
                    let row = access.row;
                    access.row += 1;
                    unsafe {
                        let entity = <Entity as EntityFetch>::get(access.entities, access.len, row)?;
                        let first_item = $first_ty::get(access.$first_col, row)?;
                        $(let $col = $ty::get(access.$col, row)?;)*
                        if $first_ty::writes() {
                            (&mut *access.scene).mark_component_changed_for_query(entity, $first_ty::type_id());
                        }
                        $(
                            if $ty::writes() {
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
