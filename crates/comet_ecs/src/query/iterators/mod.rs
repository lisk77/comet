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

impl<'a, Data: QueryData<'a>, Filters> Iterator for Query<'a, Data, Filters> {
    type Item = Data;

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
                Data::mark_changed(access.scene, entity, &access.columns);
                return Data::fetch(entity, &access.columns, row);
            }
        }
    }
}
