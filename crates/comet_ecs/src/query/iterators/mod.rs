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
) -> Option<(usize, &'a mut A, usize)> {
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

    let access_index = *idx;
    let access = &mut accesses[access_index];
    let row = *access.row_mut();
    *access.row_mut() += 1;
    Some((access_index, access, row))
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
unsafe fn row_matches(access: &QueryAccess, entity: Entity) -> bool {
    unsafe {
        access
            .filter
            .matches(access.change_state, access.spawn_ticks, entity)
    }
}

impl<'a, Data, Filters> Query<'a, Data, Filters> {
    fn prepare_entity_selection(&mut self)
    where
        Data: QueryData<'a>,
    {
        if self.entity_ranges.is_empty() || self.selected_entities.is_some() {
            return;
        }

        let mut seen = HashSet::new();
        let mut entities = Vec::new();
        for access in &self.accesses {
            for row in 0..access.len {
                self.diagnostics.entity_selection_row();
                let Some(entity) = (unsafe { fetch_entity(access.entities, access.len, row) })
                else {
                    continue;
                };
                if unsafe { row_matches(access, entity) } && seen.insert(entity) {
                    entities.push(entity);
                }
            }
        }

        for range in &self.entity_ranges {
            entities = range.select(entities);
        }
        self.selected_entities = Some(entities.into_iter().collect());
    }

    fn prepare_row_selection(&mut self)
    where
        Data: QueryData<'a>,
    {
        if self.row_ranges.is_empty() || self.selected_rows.is_some() {
            return;
        }

        let mut rows = Vec::new();
        for (access_index, access) in self.accesses.iter().enumerate() {
            for row in 0..access.len {
                self.diagnostics.row_selection_row();
                let Some(entity) = (unsafe { fetch_entity(access.entities, access.len, row) })
                else {
                    continue;
                };
                if unsafe { row_matches(access, entity) }
                    && self
                        .selected_entities
                        .as_ref()
                        .is_none_or(|entities| entities.contains(&entity))
                {
                    rows.push((access_index, row));
                }
            }
        }

        for range in &self.row_ranges {
            rows = range.select(rows);
        }
        self.selected_rows = Some(rows.into_iter().collect());
    }
}

impl<'a, Data: QueryData<'a>, Filters> Iterator for Query<'a, Data, Filters> {
    type Item = Data::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.prepare_entity_selection();
        self.prepare_row_selection();
        loop {
            let (access_index, access, row) = next_access_row(&mut self.accesses, &mut self.idx)?;
            self.diagnostics.row_considered();
            unsafe {
                let entity = fetch_entity(access.entities, access.len, row)?;
                if !row_matches(access, entity)
                    || self
                        .selected_entities
                        .as_ref()
                        .is_some_and(|entities| !entities.contains(&entity))
                    || self
                        .selected_rows
                        .as_ref()
                        .is_some_and(|rows| !rows.contains(&(access_index, row)))
                {
                    self.diagnostics.row_rejected();
                    continue;
                }
                Data::mark_changed(
                    access.change_state,
                    access.component_event_tick,
                    entity,
                    &access.columns,
                    &access.component_types,
                );
                let item = Data::fetch(
                    entity,
                    &access.columns,
                    &access.casters,
                    access.amounts.as_deref().unwrap_or_default(),
                    row,
                );
                if item.is_some() {
                    self.diagnostics.row_yielded();
                }
                return item;
            }
        }
    }
}
