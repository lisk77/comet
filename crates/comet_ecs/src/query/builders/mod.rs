use super::*;
use std::ptr;

pub(crate) fn build_query_accesses<'a, Data: QueryData<'a>>(
    scene: &'a Scene,
    state: &QueryFilterState,
) -> Vec<QueryAccess> {
    let components = Data::components();
    assert!(
        !components.is_empty(),
        "query must fetch at least one component"
    );
    assert!(
        components.len() <= MAX_QUERY_COMPONENTS,
        "query fetches more than {MAX_QUERY_COMPONENTS} components"
    );
    assert!(
        components[0].required,
        "the first query fetch cannot be optional"
    );

    let component_types = components
        .iter()
        .map(|component| component.type_id)
        .collect::<Vec<_>>();
    assert!(
        !has_duplicate_type_ids(&component_types),
        "query called with duplicate component types"
    );

    let change_state = scene.query_change_state() as *const _ as *mut _;
    let component_event_tick = scene.component_event_tick();
    let mut accesses = Vec::new();
    for (arch_id, _) in scene.cached_single_plan(
        components[0].type_id,
        &state.with_components,
        &state.without_components,
        &state.with_any_components,
        &state.without_any_components,
    ) {
        let arch = scene.archetypes().get(arch_id);
        let mut columns: [*mut comet_structs::Column; MAX_QUERY_COMPONENTS] =
            [ptr::null_mut(); MAX_QUERY_COMPONENTS];

        for (slot, component) in components.iter().enumerate() {
            columns[slot] = match arch.column_index(component.type_id) {
                Some(column_index) => {
                    &arch.columns()[column_index] as *const comet_structs::Column as *mut _
                }
                None if !component.required => ptr::null_mut(),
                None => continue,
            };
        }

        if components
            .iter()
            .enumerate()
            .any(|(slot, component)| component.required && columns[slot].is_null())
        {
            continue;
        }

        accesses.push(QueryAccess {
            entities: arch.entities().as_ptr(),
            columns,
            change_state,
            component_event_tick,
            len: arch.len(),
            row: 0,
        });
    }

    accesses
}

pub(crate) fn build_query_accesses_mut<'a, Data: QueryData<'a>>(
    scene: &'a mut Scene,
    state: &QueryFilterState,
) -> Vec<QueryAccess> {
    let components = Data::components();
    assert!(
        !components.is_empty(),
        "query must fetch at least one component"
    );
    assert!(
        components.len() <= MAX_QUERY_COMPONENTS,
        "query fetches more than {MAX_QUERY_COMPONENTS} components"
    );
    assert!(
        components[0].required,
        "the first query fetch cannot be optional"
    );

    let component_types = components
        .iter()
        .map(|component| component.type_id)
        .collect::<Vec<_>>();
    assert!(
        !has_duplicate_type_ids(&component_types),
        "query called with duplicate component types"
    );

    let plan = scene.cached_single_plan(
        components[0].type_id,
        &state.with_components,
        &state.without_components,
        &state.with_any_components,
        &state.without_any_components,
    );
    let (archetypes, change_state, component_event_tick) = scene.query_parts_mut();
    let change_state = change_state as *mut _;
    let mut accesses = Vec::new();

    for (arch_id, _) in plan {
        let arch = archetypes.get_mut(arch_id);
        let mut column_indices = [None; MAX_QUERY_COMPONENTS];

        for (slot, component) in components.iter().enumerate() {
            column_indices[slot] = arch.column_index(component.type_id);
        }

        if components
            .iter()
            .enumerate()
            .any(|(slot, component)| component.required && column_indices[slot].is_none())
        {
            continue;
        }

        let entities = arch.entities().as_ptr();
        let len = arch.len();
        let columns_ptr = arch.columns_mut().as_mut_ptr();
        let mut columns: [*mut comet_structs::Column; MAX_QUERY_COMPONENTS] =
            [ptr::null_mut(); MAX_QUERY_COMPONENTS];

        for (slot, column_index) in column_indices.into_iter().enumerate() {
            if let Some(column_index) = column_index {
                columns[slot] = unsafe { columns_ptr.add(column_index) };
            }
        }

        accesses.push(QueryAccess {
            entities,
            columns,
            change_state,
            component_event_tick,
            len,
            row: 0,
        });
    }

    accesses
}
