use super::*;
use crate::QueryTarget;
use std::ptr;

fn validate_components(components: &[QueryComponent]) {
    assert!(
        !components.is_empty(),
        "query must fetch at least one component"
    );
    assert!(
        components.len() <= MAX_QUERY_COMPONENTS,
        "query fetches more than {MAX_QUERY_COMPONENTS} components"
    );
}

fn archetype_has_target(
    scene: &Scene,
    arch: &crate::archetypes::Archetype,
    target_type: TypeId,
) -> bool {
    scene
        .query_targets(target_type)
        .iter()
        .any(|target| arch.column_index(target.component_type).is_some())
}

fn archetype_matches_filters(
    scene: &Scene,
    arch: &crate::archetypes::Archetype,
    state: &QueryFilterState,
) -> bool {
    state
        .with_components
        .iter()
        .all(|type_id| archetype_has_target(scene, arch, *type_id))
        && state
            .without_components
            .iter()
            .all(|type_id| !archetype_has_target(scene, arch, *type_id))
        && (state.with_any_components.is_empty()
            || state
                .with_any_components
                .iter()
                .any(|type_id| archetype_has_target(scene, arch, *type_id)))
        && state
            .without_any_components
            .iter()
            .all(|type_id| !archetype_has_target(scene, arch, *type_id))
}

fn has_trait_filters(scene: &Scene, state: &QueryFilterState) -> bool {
    state
        .with_components
        .iter()
        .chain(&state.without_components)
        .chain(&state.with_any_components)
        .chain(&state.without_any_components)
        .any(|target_type| {
            scene
                .query_targets(*target_type)
                .iter()
                .any(|target| target.component_type != *target_type)
        })
}

fn resolve_archetype_targets(
    scene: &Scene,
    arch: &crate::archetypes::Archetype,
    components: &[QueryComponent],
    anchor: Option<QueryTarget>,
) -> Vec<[Option<QueryTarget>; MAX_QUERY_COMPONENTS]> {
    let mut initial: [Option<QueryTarget>; MAX_QUERY_COMPONENTS] = std::array::from_fn(|_| None);
    initial[0] = anchor;
    let mut resolved = vec![initial];

    for (slot, component) in components.iter().enumerate().skip(1) {
        let candidates = scene
            .query_targets(component.type_id)
            .iter()
            .filter(|target| arch.column_index(target.component_type).is_some())
            .cloned()
            .collect::<Vec<_>>();

        if candidates.is_empty() {
            if component.required {
                return Vec::new();
            }
            continue;
        }

        let mut expanded = Vec::with_capacity(resolved.len() * candidates.len());
        for targets in resolved {
            for candidate in &candidates {
                let mut targets = targets.clone();
                targets[slot] = Some(candidate.clone());
                expanded.push(targets);
            }
        }
        resolved = expanded;
    }

    for targets in &resolved {
        let component_types = targets
            .iter()
            .flatten()
            .map(|target| target.component_type)
            .collect::<Vec<_>>();
        assert!(
            !has_duplicate_type_ids(&component_types),
            "query called with duplicate component types"
        );
    }
    resolved
}

fn resolve_change_filters(
    scene: &Scene,
    arch: &crate::archetypes::Archetype,
    targets: &[Option<QueryTarget>; MAX_QUERY_COMPONENTS],
    filters: &[(TypeId, Tick)],
) -> Vec<ResolvedChangeFilter> {
    filters
        .iter()
        .map(|(target_type, since_tick)| {
            let selected_types = targets
                .iter()
                .flatten()
                .filter(|target| target.target_type == *target_type)
                .map(|target| target.component_type)
                .collect::<Vec<_>>();
            let component_types = if selected_types.is_empty() {
                scene
                    .query_targets(*target_type)
                    .iter()
                    .filter(|target| arch.column_index(target.component_type).is_some())
                    .map(|target| target.component_type)
                    .collect()
            } else {
                selected_types
            };
            ResolvedChangeFilter {
                component_types,
                since_tick: *since_tick,
            }
        })
        .collect()
}

fn resolved_accesses(
    scene: &Scene,
    components: &[QueryComponent],
    state: &QueryFilterState,
) -> Vec<(usize, [Option<QueryTarget>; MAX_QUERY_COMPONENTS])> {
    let anchor_targets = scene.query_targets(components[0].type_id).to_vec();
    let dynamic_slots = components
        .iter()
        .enumerate()
        .filter(|(_, component)| {
            scene
                .query_targets(component.type_id)
                .iter()
                .any(|target| target.component_type != component.type_id)
        })
        .map(|(slot, _)| slot)
        .collect::<Vec<_>>();
    assert!(
        dynamic_slots.len() <= 1,
        "multiple flattened trait fetches require explicit grouping semantics"
    );
    if let Some(&dynamic_slot) = dynamic_slots.first() {
        assert!(
            !components
                .iter()
                .enumerate()
                .any(|(slot, component)| slot != dynamic_slot && component.writes),
            "a flattened trait query cannot repeat mutable concrete query data"
        );
    }

    let mut resolved = Vec::new();
    if components[0].required {
        let trait_filters = has_trait_filters(scene, state);
        for anchor in anchor_targets {
            if trait_filters {
                for (arch_id, arch) in scene.archetypes().iter().enumerate() {
                    if arch.column_index(anchor.component_type).is_none()
                        || !archetype_matches_filters(scene, arch, state)
                    {
                        continue;
                    }
                    for targets in
                        resolve_archetype_targets(scene, arch, components, Some(anchor.clone()))
                    {
                        resolved.push((arch_id, targets));
                    }
                }
            } else {
                for (arch_id, _) in scene.cached_single_plan(
                    anchor.component_type,
                    &state.with_components,
                    &state.without_components,
                    &state.with_any_components,
                    &state.without_any_components,
                ) {
                    let arch = scene.archetypes().get(arch_id);
                    for targets in
                        resolve_archetype_targets(scene, arch, components, Some(anchor.clone()))
                    {
                        resolved.push((arch_id, targets));
                    }
                }
            }
        }
        return resolved;
    }

    for (arch_id, arch) in scene.archetypes().iter().enumerate() {
        if !archetype_matches_filters(scene, arch, state) {
            continue;
        }
        let present = anchor_targets
            .iter()
            .filter(|target| arch.column_index(target.component_type).is_some())
            .cloned()
            .collect::<Vec<_>>();
        if present.is_empty() {
            for targets in resolve_archetype_targets(scene, arch, components, None) {
                resolved.push((arch_id, targets));
            }
        } else {
            for anchor in present {
                for targets in resolve_archetype_targets(scene, arch, components, Some(anchor)) {
                    resolved.push((arch_id, targets));
                }
            }
        }
    }
    resolved
}

pub(crate) fn build_query_accesses<'a, Data: QueryData<'a>>(
    scene: &'a Scene,
    state: &QueryFilterState,
) -> Vec<QueryAccess> {
    let components = Data::components();
    validate_components(&components);
    let resolved = resolved_accesses(scene, &components, state);
    let change_state = scene.query_change_state() as *const _ as *mut _;
    let component_event_tick = scene.component_event_tick();
    let mut accesses = Vec::with_capacity(resolved.len());

    for (arch_id, targets) in resolved {
        let arch = scene.archetypes().get(arch_id);
        let added_since_filters =
            resolve_change_filters(scene, arch, &targets, &state.added_since_filters);
        let changed_since_filters =
            resolve_change_filters(scene, arch, &targets, &state.changed_since_filters);
        let mut columns = [ptr::null_mut(); MAX_QUERY_COMPONENTS];
        let mut component_types = [None; MAX_QUERY_COMPONENTS];
        let mut casters: [Option<crate::QueryCaster>; MAX_QUERY_COMPONENTS] =
            std::array::from_fn(|_| None);

        for (slot, target) in targets.into_iter().enumerate() {
            let Some(target) = target else {
                continue;
            };
            let column_index = arch
                .column_index(target.component_type)
                .expect("resolved query target is missing its archetype column");
            columns[slot] = &arch.columns()[column_index] as *const comet_structs::Column as *mut _;
            component_types[slot] = Some(target.component_type);
            casters[slot] = Some(target.caster);
        }

        accesses.push(QueryAccess {
            entities: arch.entities().as_ptr(),
            columns,
            component_types,
            casters,
            change_state,
            component_event_tick,
            added_since_filters,
            changed_since_filters,
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
    validate_components(&components);
    let resolved = resolved_accesses(scene, &components, state)
        .into_iter()
        .map(|(arch_id, targets)| {
            let arch = scene.archetypes().get(arch_id);
            let added_since_filters =
                resolve_change_filters(scene, arch, &targets, &state.added_since_filters);
            let changed_since_filters =
                resolve_change_filters(scene, arch, &targets, &state.changed_since_filters);
            (arch_id, targets, added_since_filters, changed_since_filters)
        })
        .collect::<Vec<_>>();
    let (archetypes, change_state, component_event_tick) = scene.query_parts_mut();
    let change_state = change_state as *mut _;
    let mut accesses = Vec::with_capacity(resolved.len());

    for (arch_id, targets, added_since_filters, changed_since_filters) in resolved {
        let arch = archetypes.get_mut(arch_id);
        let entities = arch.entities().as_ptr();
        let len = arch.len();
        let columns_ptr = arch.columns_mut().as_mut_ptr();
        let mut columns = [ptr::null_mut(); MAX_QUERY_COMPONENTS];
        let mut component_types = [None; MAX_QUERY_COMPONENTS];
        let mut casters: [Option<crate::QueryCaster>; MAX_QUERY_COMPONENTS] =
            std::array::from_fn(|_| None);

        for (slot, target) in targets.into_iter().enumerate() {
            let Some(target) = target else {
                continue;
            };
            let column_index = arch
                .column_index(target.component_type)
                .expect("resolved query target is missing its archetype column");
            columns[slot] = unsafe { columns_ptr.add(column_index) };
            component_types[slot] = Some(target.component_type);
            casters[slot] = Some(target.caster);
        }

        accesses.push(QueryAccess {
            entities,
            columns,
            component_types,
            casters,
            change_state,
            component_event_tick,
            added_since_filters,
            changed_since_filters,
            len,
            row: 0,
        });
    }

    accesses
}
