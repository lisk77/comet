use super::*;
use crate::QueryTarget;
use std::ptr;

type TemporalCountCache =
    HashMap<(usize, Arc<[TypeId]>, usize, usize, Tick, TemporalFilterKind), ResolvedQueryFilter>;

fn validate_components(components: &[QueryComponent]) {
    assert!(
        components.len() <= MAX_QUERY_COMPONENTS,
        "query fetches more than {MAX_QUERY_COMPONENTS} components"
    );
    for first in 0..components.len() {
        for second in (first + 1)..components.len() {
            assert!(
                components[first].type_id != components[second].type_id
                    || (!components[first].writes && !components[second].writes),
                "query called with duplicate component types"
            );
        }
    }
}

fn archetype_target_types(
    scene: &Scene,
    arch: &crate::archetypes::Archetype,
    target_types: &[TypeId],
) -> Vec<TypeId> {
    let mut seen = HashSet::new();
    target_types
        .iter()
        .flat_map(|target_type| scene.query_targets(*target_type))
        .filter(|target| arch.column_index(target.component_type).is_some())
        .filter_map(|target| {
            seen.insert(target.component_type)
                .then_some(target.component_type)
        })
        .collect()
}

fn archetype_target_count(
    scene: &Scene,
    arch: &crate::archetypes::Archetype,
    target_types: &[TypeId],
) -> usize {
    target_types
        .iter()
        .flat_map(|target_type| scene.query_targets(*target_type))
        .filter(|target| arch.column_index(target.component_type).is_some())
        .map(|target| target.component_type)
        .collect::<HashSet<_>>()
        .len()
}

fn resolve_amounts(
    scene: &Scene,
    arch: &crate::archetypes::Archetype,
    amounts: &[QueryAmount],
) -> Vec<usize> {
    amounts
        .iter()
        .map(|amount| {
            scene
                .query_targets(amount.type_id)
                .iter()
                .filter(|target| arch.column_index(target.component_type).is_some())
                .count()
        })
        .collect()
}

fn archetype_matches_filters(
    scene: &Scene,
    arch: &crate::archetypes::Archetype,
    state: &QueryFilterState,
) -> bool {
    state
        .expression
        .archetype_match(&|type_ids| archetype_target_count(scene, arch, type_ids))
        != ArchetypeFilterMatch::Never
}

fn selected_archetype_targets(
    scene: &Scene,
    arch: &crate::archetypes::Archetype,
    component: &QueryComponent,
) -> Vec<QueryTarget> {
    let mut targets = scene
        .query_targets(component.type_id)
        .iter()
        .filter(|target| arch.column_index(target.component_type).is_some())
        .cloned()
        .collect::<Vec<_>>();
    if uses_candidate_ranges(scene, component.type_id) {
        for range in &component.ranges {
            targets = range.select(targets);
        }
    }
    targets
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
        let candidates = selected_archetype_targets(scene, arch, component);

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

    resolved.retain(|targets| {
        for first in 0..components.len() {
            let Some(first_target) = targets[first].as_ref() else {
                continue;
            };
            for second in (first + 1)..components.len() {
                let Some(second_target) = targets[second].as_ref() else {
                    continue;
                };
                if first_target.component_type == second_target.component_type
                    && (components[first].writes || components[second].writes)
                {
                    return false;
                }
            }
        }
        true
    });
    resolved
}

fn resolve_change_filter(
    scene: &Scene,
    arch: &crate::archetypes::Archetype,
    components: &[QueryComponent],
    targets: &[Option<QueryTarget>; MAX_QUERY_COMPONENTS],
    target_type: TypeId,
    since_tick: Tick,
) -> ResolvedChangeFilter {
    let selected_slots = components
        .iter()
        .enumerate()
        .filter(|(_, component)| component.type_id == target_type)
        .map(|(slot, _)| slot)
        .collect::<Vec<_>>();
    let component_types = if selected_slots.is_empty() {
        scene
            .query_targets(target_type)
            .iter()
            .filter(|target| arch.column_index(target.component_type).is_some())
            .map(|target| target.component_type)
            .collect()
    } else {
        selected_slots
            .into_iter()
            .filter_map(|slot| targets[slot].as_ref())
            .map(|target| target.component_type)
            .collect()
    };
    ResolvedChangeFilter {
        component_types,
        since_tick,
    }
}

fn resolve_temporal_count_filter(
    scene: &Scene,
    arch_id: usize,
    arch: &crate::archetypes::Archetype,
    target_types: &Arc<[TypeId]>,
    min: usize,
    max: usize,
    since_tick: Tick,
    kind: TemporalFilterKind,
    cache: &mut TemporalCountCache,
) -> ResolvedQueryFilter {
    let key = (arch_id, target_types.clone(), min, max, since_tick, kind);
    if let Some(filter) = cache.get(&key) {
        return filter.clone();
    }

    let component_types = archetype_target_types(scene, arch, target_types);
    let candidate_count = component_types.len();
    let resolved = if min > max || min > candidate_count {
        ResolvedQueryFilter::False
    } else if min == 0 && max >= candidate_count {
        ResolvedQueryFilter::True
    } else {
        let matching_entities = arch
            .entities()
            .iter()
            .copied()
            .filter(|entity| {
                let count = component_types
                    .iter()
                    .filter(|component_type| match kind {
                        TemporalFilterKind::Added => {
                            scene.component_added_since_type(*entity, **component_type, since_tick)
                        }
                        TemporalFilterKind::Changed => scene.component_changed_since_type(
                            *entity,
                            **component_type,
                            since_tick,
                        ),
                        TemporalFilterKind::Modified => {
                            scene.component_changed_since_type(
                                *entity,
                                **component_type,
                                since_tick,
                            ) && !scene.component_added_since_type(
                                *entity,
                                **component_type,
                                since_tick,
                            )
                        }
                    })
                    .count();
                count >= min && count <= max
            })
            .collect::<HashSet<_>>();
        ResolvedQueryFilter::TemporalCount(ResolvedTemporalCountFilter {
            matching_entities: Arc::new(matching_entities),
            component_types: component_types.into(),
            min,
            max,
            since_tick,
            kind,
        })
    };
    cache.insert(key, resolved.clone());
    resolved
}

fn resolve_filter(
    scene: &Scene,
    arch_id: usize,
    arch: &crate::archetypes::Archetype,
    components: &[QueryComponent],
    targets: &[Option<QueryTarget>; MAX_QUERY_COMPONENTS],
    filter: &QueryFilterExpr,
    temporal_count_cache: &mut TemporalCountCache,
) -> ResolvedQueryFilter {
    match filter {
        QueryFilterExpr::True => ResolvedQueryFilter::True,
        QueryFilterExpr::False => ResolvedQueryFilter::False,
        QueryFilterExpr::Count(type_ids, min, max) => {
            let count = archetype_target_count(scene, arch, type_ids);
            if count >= *min && count <= *max {
                ResolvedQueryFilter::True
            } else {
                ResolvedQueryFilter::False
            }
        }
        QueryFilterExpr::TemporalCount(type_ids, min, max, since_tick, kind) => {
            resolve_temporal_count_filter(
                scene,
                arch_id,
                arch,
                type_ids,
                *min,
                *max,
                *since_tick,
                *kind,
                temporal_count_cache,
            )
        }
        QueryFilterExpr::Added(type_id, since_tick) => ResolvedQueryFilter::Added(
            resolve_change_filter(scene, arch, components, targets, *type_id, *since_tick),
        ),
        QueryFilterExpr::Changed(type_id, since_tick) => ResolvedQueryFilter::Changed(
            resolve_change_filter(scene, arch, components, targets, *type_id, *since_tick),
        ),
        QueryFilterExpr::Modified(type_id, since_tick) => ResolvedQueryFilter::Modified(
            resolve_change_filter(scene, arch, components, targets, *type_id, *since_tick),
        ),
        QueryFilterExpr::Spawned(since_tick) => ResolvedQueryFilter::Spawned(*since_tick),
        QueryFilterExpr::And(filters) => ResolvedQueryFilter::and(
            filters
                .iter()
                .map(|filter| {
                    resolve_filter(
                        scene,
                        arch_id,
                        arch,
                        components,
                        targets,
                        filter,
                        temporal_count_cache,
                    )
                })
                .collect(),
        ),
        QueryFilterExpr::Or(filters) => ResolvedQueryFilter::or(
            filters
                .iter()
                .map(|filter| {
                    resolve_filter(
                        scene,
                        arch_id,
                        arch,
                        components,
                        targets,
                        filter,
                        temporal_count_cache,
                    )
                })
                .collect(),
        ),
        QueryFilterExpr::Not(filter) => ResolvedQueryFilter::not(resolve_filter(
            scene,
            arch_id,
            arch,
            components,
            targets,
            filter,
            temporal_count_cache,
        )),
    }
}

fn resolved_accesses(
    scene: &Scene,
    components: &[QueryComponent],
    state: &QueryFilterState,
    diagnostics: &mut QueryDiagnostics,
) -> Vec<(usize, [Option<QueryTarget>; MAX_QUERY_COMPONENTS])> {
    if components.is_empty() {
        return scene
            .archetypes()
            .iter()
            .enumerate()
            .inspect(|_| diagnostics.inspect_archetype())
            .filter(|(_, archetype)| archetype_matches_filters(scene, archetype, state))
            .map(|(archetype, _)| (archetype, std::array::from_fn(|_| None)))
            .collect();
    }

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
    if dynamic_slots.len() > 1 {
        assert!(
            !components.iter().any(|component| component.writes),
            "multiple flattened trait fetches must be immutable"
        );
    } else if let Some(&dynamic_slot) = dynamic_slots.first() {
        assert!(
            !components
                .iter()
                .enumerate()
                .any(|(slot, component)| slot != dynamic_slot && component.writes),
            "a flattened trait query cannot repeat mutable concrete query data"
        );
    }

    let mut resolved = Vec::new();
    if components[0].required
        && !components[0].ranges.is_empty()
        && uses_candidate_ranges(scene, components[0].type_id)
    {
        for (arch_id, arch) in scene.archetypes().iter().enumerate() {
            diagnostics.inspect_archetype();
            if !archetype_matches_filters(scene, arch, state) {
                continue;
            }
            for anchor in selected_archetype_targets(scene, arch, &components[0]) {
                for targets in resolve_archetype_targets(scene, arch, components, Some(anchor)) {
                    resolved.push((arch_id, targets));
                }
            }
        }
        return resolved;
    }

    if components[0].required {
        let scan_archetypes = state.simple.is_none() || state.has_trait_cardinality_filter(scene);
        for anchor in anchor_targets {
            if scan_archetypes {
                for (arch_id, arch) in scene.archetypes().iter().enumerate() {
                    diagnostics.inspect_archetype();
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
                let simple = state.simple.as_ref().unwrap();
                let (matches, cache_hit, inspected) = scene.cached_single_plan(
                    anchor.component_type,
                    &simple.with_components,
                    &simple.without_components,
                    &simple.with_any_components,
                    &simple.without_any_components,
                );
                if cache_hit {
                    diagnostics.cache_hit();
                } else {
                    diagnostics.cache_miss();
                }
                for _ in 0..inspected {
                    diagnostics.inspect_archetype();
                }
                for (arch_id, _) in matches {
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
        diagnostics.inspect_archetype();
        if !archetype_matches_filters(scene, arch, state) {
            continue;
        }
        let present = selected_archetype_targets(scene, arch, &components[0]);
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

pub(crate) fn build_query_accesses(
    scene: &Scene,
    state: &QueryFilterState,
    layout: &QueryLayout,
    diagnostics: &mut QueryDiagnostics,
) -> Vec<QueryAccess> {
    validate_components(&layout.components);
    let resolved = resolved_accesses(scene, &layout.components, state, diagnostics);
    diagnostics.planned_combinations(resolved.len());
    let change_state = scene.query_change_state() as *const _ as *mut _;
    let spawn_ticks = scene.query_spawn_ticks() as *const _;
    let component_event_tick = scene.component_event_tick();
    let mut accesses = Vec::with_capacity(resolved.len());
    let mut temporal_count_cache = TemporalCountCache::new();
    let mut amount_cache: HashMap<usize, Arc<[usize]>> = HashMap::new();

    for (arch_id, targets) in resolved {
        let arch = scene.archetypes().get(arch_id);
        let filter = resolve_filter(
            scene,
            arch_id,
            arch,
            &layout.components,
            &targets,
            &state.expression,
            &mut temporal_count_cache,
        );
        if filter.is_false() {
            diagnostics.filtered_access();
            continue;
        }
        let amounts = (!layout.amounts.is_empty()).then(|| {
            amount_cache
                .entry(arch_id)
                .or_insert_with(|| resolve_amounts(scene, arch, &layout.amounts).into())
                .clone()
        });
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
            amounts,
            change_state,
            spawn_ticks,
            component_event_tick,
            filter,
            len: arch.len(),
            row: 0,
        });
    }

    accesses
}

pub(crate) fn build_query_accesses_mut(
    scene: &mut Scene,
    state: &QueryFilterState,
    layout: &QueryLayout,
    diagnostics: &mut QueryDiagnostics,
) -> Vec<QueryAccess> {
    validate_components(&layout.components);
    let mut temporal_count_cache = TemporalCountCache::new();
    let mut amount_cache: HashMap<usize, Arc<[usize]>> = HashMap::new();
    let resolved_accesses = resolved_accesses(scene, &layout.components, state, diagnostics);
    diagnostics.planned_combinations(resolved_accesses.len());
    let resolved = resolved_accesses
        .into_iter()
        .filter_map(|(arch_id, targets)| {
            let arch = scene.archetypes().get(arch_id);
            let filter = resolve_filter(
                scene,
                arch_id,
                arch,
                &layout.components,
                &targets,
                &state.expression,
                &mut temporal_count_cache,
            );
            if filter.is_false() {
                diagnostics.filtered_access();
                return None;
            }
            let amounts = (!layout.amounts.is_empty()).then(|| {
                amount_cache
                    .entry(arch_id)
                    .or_insert_with(|| resolve_amounts(scene, arch, &layout.amounts).into())
                    .clone()
            });
            Some((arch_id, targets, filter, amounts))
        })
        .collect::<Vec<_>>();
    let (archetypes, change_state, spawn_ticks, component_event_tick) = scene.query_parts_mut();
    let change_state = change_state as *mut _;
    let spawn_ticks = spawn_ticks as *const _;
    let mut accesses = Vec::with_capacity(resolved.len());

    for (arch_id, targets, filter, amounts) in resolved {
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
            amounts,
            change_state,
            spawn_ticks,
            component_event_tick,
            filter,
            len,
            row: 0,
        });
    }

    accesses
}
