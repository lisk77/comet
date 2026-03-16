use super::*;

pub(super) trait RowAccess {
    fn len(&self) -> usize;
    fn row_mut(&mut self) -> &mut usize;
}

impl RowAccess for QueryAccess {
    fn len(&self) -> usize {
        self.len
    }

    fn row_mut(&mut self) -> &mut usize {
        &mut self.row
    }
}

impl RowAccess for QueryMutAccess {
    fn len(&self) -> usize {
        self.len
    }

    fn row_mut(&mut self) -> &mut usize {
        &mut self.row
    }
}

pub(super) fn next_access_row<'a, A: RowAccess>(
    accesses: &'a mut [A],
    idx: &mut usize,
) -> Option<(&'a mut A, usize)> {
    if *idx >= accesses.len() {
        return None;
    }

    let should_advance = {
        let access = &mut accesses[*idx];
        *access.row_mut() >= access.len()
    };
    if should_advance {
        *idx += 1;
        return next_access_row(accesses, idx);
    }

    let access = &mut accesses[*idx];
    let row = *access.row_mut();
    *access.row_mut() += 1;
    Some((access, row))
}

#[inline(always)]
pub(super) unsafe fn fetch_entity(
    entities: *const Entity,
    len: usize,
    row: usize,
) -> Option<Entity> {
    unsafe { <Entity as EntityFetch>::get(entities, len, row) }
}

#[inline(always)]
pub(super) fn has_change_filters(
    added_since_filters: &[(TypeId, Tick)],
    changed_since_filters: &[(TypeId, Tick)],
) -> bool {
    !added_since_filters.is_empty() || !changed_since_filters.is_empty()
}

#[inline(always)]
pub(super) unsafe fn matches_change_filters(
    scene: *const Scene,
    entity: Entity,
    added_since_filters: &[(TypeId, Tick)],
    changed_since_filters: &[(TypeId, Tick)],
) -> bool {
    let scene = unsafe { &*scene };
    for (type_id, tick) in added_since_filters {
        if !scene.component_added_since_type(entity, *type_id, *tick) {
            return false;
        }
    }
    for (type_id, tick) in changed_since_filters {
        if !scene.component_changed_since_type(entity, *type_id, *tick) {
            return false;
        }
    }
    true
}

mod single;
mod tuples;
