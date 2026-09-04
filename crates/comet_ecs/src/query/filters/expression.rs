use super::*;

#[derive(Clone)]
pub(crate) struct ResolvedChangeFilter {
    pub(crate) component_types: Vec<TypeId>,
    pub(crate) since_tick: Tick,
}

#[derive(Clone)]
pub(crate) struct ResolvedTemporalCountFilter {
    pub(crate) matching_entities: Arc<HashSet<Entity>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TemporalFilterKind {
    Added,
    Changed,
    Modified,
}

#[derive(Clone)]
pub(crate) enum QueryFilterExpr {
    True,
    False,
    Count(Arc<[TypeId]>, usize, usize),
    TemporalCount(Arc<[TypeId]>, usize, usize, Tick, TemporalFilterKind),
    Added(TypeId, Tick),
    Changed(TypeId, Tick),
    Modified(TypeId, Tick),
    Spawned(Tick),
    And(Vec<Self>),
    Or(Vec<Self>),
    Not(Box<Self>),
}

impl QueryFilterExpr {
    pub(super) fn count(type_ids: Arc<[TypeId]>, min: usize, max: usize) -> Self {
        if min > max || (type_ids.is_empty() && min > 0) {
            Self::False
        } else if type_ids.is_empty() {
            Self::True
        } else {
            Self::Count(type_ids, min, max)
        }
    }

    pub(super) fn temporal_count(
        type_ids: Arc<[TypeId]>,
        min: usize,
        max: usize,
        since_tick: Tick,
        kind: TemporalFilterKind,
    ) -> Self {
        if min > max || (type_ids.is_empty() && min > 0) {
            Self::False
        } else if type_ids.is_empty() {
            Self::True
        } else {
            Self::TemporalCount(type_ids, min, max, since_tick, kind)
        }
    }

    pub(super) fn and(filters: Vec<Self>) -> Self {
        let mut flattened = Vec::new();
        for filter in filters {
            match filter {
                Self::True => {}
                Self::False => return Self::False,
                Self::And(filters) => flattened.extend(filters),
                filter => flattened.push(filter),
            }
        }
        match flattened.len() {
            0 => Self::True,
            1 => flattened.pop().unwrap(),
            _ => Self::And(flattened),
        }
    }

    pub(super) fn or(filters: Vec<Self>) -> Self {
        let mut flattened = Vec::new();
        for filter in filters {
            match filter {
                Self::True => return Self::True,
                Self::False => {}
                Self::Or(filters) => flattened.extend(filters),
                filter => flattened.push(filter),
            }
        }
        match flattened.len() {
            0 => Self::False,
            1 => flattened.pop().unwrap(),
            _ => Self::Or(flattened),
        }
    }

    pub(super) fn not(filter: Self) -> Self {
        match filter {
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Not(filter) => *filter,
            filter => Self::Not(Box::new(filter)),
        }
    }

    pub(crate) fn archetype_match(
        &self,
        target_count: &impl Fn(&[TypeId]) -> usize,
    ) -> ArchetypeFilterMatch {
        match self {
            Self::True => ArchetypeFilterMatch::Always,
            Self::False => ArchetypeFilterMatch::Never,
            Self::Count(type_ids, min, max) => {
                let count = target_count(type_ids);
                ArchetypeFilterMatch::from_bool(count >= *min && count <= *max)
            }
            Self::TemporalCount(type_ids, min, max, _, _) => {
                let count = target_count(type_ids);
                if min > max || *min > count {
                    ArchetypeFilterMatch::Never
                } else if *min == 0 && *max >= count {
                    ArchetypeFilterMatch::Always
                } else {
                    ArchetypeFilterMatch::Dynamic
                }
            }
            Self::Added(type_id, _) | Self::Changed(type_id, _) | Self::Modified(type_id, _) => {
                if target_count(std::slice::from_ref(type_id)) > 0 {
                    ArchetypeFilterMatch::Dynamic
                } else {
                    ArchetypeFilterMatch::Never
                }
            }
            Self::Spawned(_) => ArchetypeFilterMatch::Dynamic,
            Self::And(filters) => filters
                .iter()
                .fold(ArchetypeFilterMatch::Always, |result, filter| {
                    result.and(filter.archetype_match(target_count))
                }),
            Self::Or(filters) => filters
                .iter()
                .fold(ArchetypeFilterMatch::Never, |result, filter| {
                    result.or(filter.archetype_match(target_count))
                }),
            Self::Not(filter) => filter.archetype_match(target_count).not(),
        }
    }

    fn has_trait_cardinality_filter(&self, scene: &Scene) -> bool {
        match self {
            Self::Count(type_ids, _, _) | Self::TemporalCount(type_ids, _, _, _, _) => {
                type_ids.iter().any(|type_id| {
                    scene
                        .query_targets(*type_id)
                        .iter()
                        .any(|target| target.component_type != *type_id)
                })
            }
            Self::And(filters) | Self::Or(filters) => filters
                .iter()
                .any(|filter| filter.has_trait_cardinality_filter(scene)),
            Self::Not(filter) => filter.has_trait_cardinality_filter(scene),
            Self::True
            | Self::False
            | Self::Added(_, _)
            | Self::Changed(_, _)
            | Self::Modified(_, _)
            | Self::Spawned(_) => false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArchetypeFilterMatch {
    Always,
    Never,
    Dynamic,
}

impl ArchetypeFilterMatch {
    fn from_bool(value: bool) -> Self {
        if value {
            Self::Always
        } else {
            Self::Never
        }
    }

    fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::Never, _) | (_, Self::Never) => Self::Never,
            (Self::Always, Self::Always) => Self::Always,
            _ => Self::Dynamic,
        }
    }

    fn or(self, other: Self) -> Self {
        match (self, other) {
            (Self::Always, _) | (_, Self::Always) => Self::Always,
            (Self::Never, Self::Never) => Self::Never,
            _ => Self::Dynamic,
        }
    }

    fn not(self) -> Self {
        match self {
            Self::Always => Self::Never,
            Self::Never => Self::Always,
            Self::Dynamic => Self::Dynamic,
        }
    }
}

#[derive(Clone)]
pub(crate) enum ResolvedQueryFilter {
    True,
    False,
    Added(ResolvedChangeFilter),
    Changed(ResolvedChangeFilter),
    Modified(ResolvedChangeFilter),
    TemporalCount(ResolvedTemporalCountFilter),
    Spawned(Tick),
    And(Vec<Self>),
    Or(Vec<Self>),
    Not(Box<Self>),
}

impl ResolvedQueryFilter {
    pub(crate) fn and(filters: Vec<Self>) -> Self {
        let mut flattened = Vec::new();
        for filter in filters {
            match filter {
                Self::True => {}
                Self::False => return Self::False,
                Self::And(filters) => flattened.extend(filters),
                filter => flattened.push(filter),
            }
        }
        match flattened.len() {
            0 => Self::True,
            1 => flattened.pop().unwrap(),
            _ => Self::And(flattened),
        }
    }

    pub(crate) fn or(filters: Vec<Self>) -> Self {
        let mut flattened = Vec::new();
        for filter in filters {
            match filter {
                Self::True => return Self::True,
                Self::False => {}
                Self::Or(filters) => flattened.extend(filters),
                filter => flattened.push(filter),
            }
        }
        match flattened.len() {
            0 => Self::False,
            1 => flattened.pop().unwrap(),
            _ => Self::Or(flattened),
        }
    }

    pub(crate) fn not(filter: Self) -> Self {
        match filter {
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Not(filter) => *filter,
            filter => Self::Not(Box::new(filter)),
        }
    }

    pub(crate) fn is_false(&self) -> bool {
        matches!(self, Self::False)
    }

    pub(crate) unsafe fn matches(
        &self,
        change_state: *const HashMap<(u32, TypeId), ComponentChangeState>,
        spawn_ticks: *const HashMap<Entity, Tick>,
        entity: Entity,
    ) -> bool {
        match self {
            Self::True => true,
            Self::False => false,
            Self::Added(filter) => unsafe {
                matches_change_filter(change_state, entity, filter, |state| state.added_tick)
            },
            Self::Changed(filter) => unsafe {
                matches_change_filter(change_state, entity, filter, |state| state.changed_tick)
            },
            Self::Modified(filter) => unsafe {
                matches_modified_filter(change_state, entity, filter)
            },
            Self::TemporalCount(filter) => filter.matching_entities.contains(&entity),
            Self::Spawned(since_tick) => unsafe {
                (&*spawn_ticks)
                    .get(&entity)
                    .is_some_and(|tick| tick_is_newer_than(*tick, *since_tick))
            },
            Self::And(filters) => filters
                .iter()
                .all(|filter| unsafe { filter.matches(change_state, spawn_ticks, entity) }),
            Self::Or(filters) => filters
                .iter()
                .any(|filter| unsafe { filter.matches(change_state, spawn_ticks, entity) }),
            Self::Not(filter) => unsafe { !filter.matches(change_state, spawn_ticks, entity) },
        }
    }
}

unsafe fn matches_change_filter(
    change_state: *const HashMap<(u32, TypeId), ComponentChangeState>,
    entity: Entity,
    filter: &ResolvedChangeFilter,
    tick: impl Fn(&ComponentChangeState) -> Tick,
) -> bool {
    let change_state = unsafe { &*change_state };
    filter.component_types.iter().any(|type_id| {
        change_state
            .get(&(entity.index, *type_id))
            .is_some_and(|state| tick_is_newer_than(tick(state), filter.since_tick))
    })
}

unsafe fn matches_modified_filter(
    change_state: *const HashMap<(u32, TypeId), ComponentChangeState>,
    entity: Entity,
    filter: &ResolvedChangeFilter,
) -> bool {
    let change_state = unsafe { &*change_state };
    filter.component_types.iter().any(|type_id| {
        change_state
            .get(&(entity.index, *type_id))
            .is_some_and(|state| {
                tick_is_newer_than(state.changed_tick, filter.since_tick)
                    && !tick_is_newer_than(state.added_tick, filter.since_tick)
            })
    })
}

fn tick_is_newer_than(tick: Tick, last_seen_tick: Tick) -> bool {
    tick != last_seen_tick && tick.wrapping_sub(last_seen_tick) <= (u32::MAX / 2)
}

#[derive(Default)]
pub(crate) struct SimpleQueryFilters {
    pub(crate) with_components: Vec<TypeId>,
    pub(crate) without_components: Vec<TypeId>,
    pub(crate) with_any_components: Vec<TypeId>,
    pub(crate) without_any_components: Vec<TypeId>,
}

impl SimpleQueryFilters {
    fn merge(&mut self, mut other: Self) -> bool {
        if !self.with_any_components.is_empty() && !other.with_any_components.is_empty() {
            return false;
        }
        self.with_components.append(&mut other.with_components);
        self.without_components
            .append(&mut other.without_components);
        if self.with_any_components.is_empty() {
            self.with_any_components = other.with_any_components;
        }
        self.without_any_components
            .append(&mut other.without_any_components);
        true
    }
}

pub(crate) struct QueryFilterState {
    pub(crate) expression: QueryFilterExpr,
    pub(crate) simple: Option<SimpleQueryFilters>,
}

impl QueryFilterState {
    pub(crate) fn has_trait_cardinality_filter(&self, scene: &Scene) -> bool {
        self.expression.has_trait_cardinality_filter(scene)
    }
}

fn with_type_id(expression: &QueryFilterExpr) -> Option<TypeId> {
    match expression {
        QueryFilterExpr::Count(type_ids, 1, usize::MAX) if type_ids.len() == 1 => {
            type_ids.first().copied()
        }
        _ => None,
    }
}

pub(super) fn simple_filters(expression: &QueryFilterExpr) -> Option<SimpleQueryFilters> {
    match expression {
        QueryFilterExpr::True
        | QueryFilterExpr::TemporalCount(_, _, _, _, _)
        | QueryFilterExpr::Added(_, _)
        | QueryFilterExpr::Changed(_, _)
        | QueryFilterExpr::Modified(_, _)
        | QueryFilterExpr::Spawned(_) => Some(SimpleQueryFilters::default()),
        QueryFilterExpr::False => None,
        QueryFilterExpr::Count(type_ids, 1, usize::MAX) if type_ids.len() == 1 => {
            Some(SimpleQueryFilters {
                with_components: type_ids.to_vec(),
                ..Default::default()
            })
        }
        QueryFilterExpr::Count(type_ids, 1, usize::MAX) => Some(SimpleQueryFilters {
            with_any_components: type_ids.to_vec(),
            ..Default::default()
        }),
        QueryFilterExpr::Count(type_ids, 0, 0) if type_ids.len() == 1 => Some(SimpleQueryFilters {
            without_components: type_ids.to_vec(),
            ..Default::default()
        }),
        QueryFilterExpr::Count(type_ids, 0, 0) => Some(SimpleQueryFilters {
            without_any_components: type_ids.to_vec(),
            ..Default::default()
        }),
        QueryFilterExpr::Count(_, _, _) => None,
        QueryFilterExpr::Not(filter) => match filter.as_ref() {
            QueryFilterExpr::Count(type_ids, 1, usize::MAX) if type_ids.len() == 1 => {
                Some(SimpleQueryFilters {
                    without_components: type_ids.to_vec(),
                    ..Default::default()
                })
            }
            QueryFilterExpr::Count(type_ids, 1, usize::MAX) => Some(SimpleQueryFilters {
                without_any_components: type_ids.to_vec(),
                ..Default::default()
            }),
            QueryFilterExpr::Count(type_ids, 0, 0) if type_ids.len() == 1 => {
                Some(SimpleQueryFilters {
                    with_components: type_ids.to_vec(),
                    ..Default::default()
                })
            }
            QueryFilterExpr::Count(type_ids, 0, 0) => Some(SimpleQueryFilters {
                with_any_components: type_ids.to_vec(),
                ..Default::default()
            }),
            QueryFilterExpr::Or(filters)
                if filters.iter().all(|filter| with_type_id(filter).is_some()) =>
            {
                Some(SimpleQueryFilters {
                    without_any_components: filters.iter().filter_map(with_type_id).collect(),
                    ..Default::default()
                })
            }
            _ => None,
        },
        QueryFilterExpr::Or(filters)
            if filters.iter().all(|filter| with_type_id(filter).is_some()) =>
        {
            Some(SimpleQueryFilters {
                with_any_components: filters.iter().filter_map(with_type_id).collect(),
                ..Default::default()
            })
        }
        QueryFilterExpr::And(filters) => {
            let mut simple = SimpleQueryFilters::default();
            for filter in filters {
                if !simple.merge(simple_filters(filter)?) {
                    return None;
                }
            }
            Some(simple)
        }
        QueryFilterExpr::Or(_) => None,
    }
}
