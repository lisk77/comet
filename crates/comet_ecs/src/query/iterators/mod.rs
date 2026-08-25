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
    change_state: *const HashMap<(u32, TypeId), ComponentChangeState>,
    entity: Entity,
    added_since_filters: &[(TypeId, Tick)],
    changed_since_filters: &[(TypeId, Tick)],
) -> bool {
    let change_state = unsafe { &*change_state };
    for (type_id, tick) in added_since_filters {
        if !change_state
            .get(&(entity.index, *type_id))
            .is_some_and(|state| tick_is_newer_than(state.added_tick, *tick))
        {
            return false;
        }
    }
    for (type_id, tick) in changed_since_filters {
        if !change_state
            .get(&(entity.index, *type_id))
            .is_some_and(|state| tick_is_newer_than(state.changed_tick, *tick))
        {
            return false;
        }
    }
    true
}

#[inline(always)]
fn tick_is_newer_than(tick: Tick, last_seen_tick: Tick) -> bool {
    tick != last_seen_tick && tick.wrapping_sub(last_seen_tick) <= (u32::MAX / 2)
}

impl<'a, Data: QueryData<'a>, Filters> Iterator for Query<'a, Data, Filters> {
    type Item = Data;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (access, row) = next_access_row(&mut self.accesses, &mut self.idx)?;
            unsafe {
                let entity = fetch_entity(access.entities, access.len, row)?;
                if !matches_change_filters(
                    access.change_state,
                    entity,
                    &self.added_since_filters,
                    &self.changed_since_filters,
                ) {
                    continue;
                }
                Data::mark_changed(
                    access.change_state,
                    access.component_event_tick,
                    entity,
                    &access.columns,
                );
                return Data::fetch(entity, &access.columns, row);
            }
        }
    }
}
