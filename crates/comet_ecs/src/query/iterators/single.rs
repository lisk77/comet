use super::*;

impl<'a, P: ReadFetch<'a>> Iterator for QueryIter<'a, P> {
    type Item = P::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (access, row) = next_access_row(&mut self.accesses, &mut self.idx)?;
            if has_change_filters(&self.added_since_filters, &self.changed_since_filters) {
                let Some(entity) = (unsafe { fetch_entity(access.entities, access.len, row) }) else {
                    continue;
                };
                if unsafe {
                    !matches_change_filters(
                        access.scene,
                        entity,
                        &self.added_since_filters,
                        &self.changed_since_filters,
                    )
                } {
                    continue;
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
