use crate::component_changes::ComponentChangeState;
use crate::{Component, Entity, Scene, Tick};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::sync::Arc;

mod arities;
mod builders;
mod fetch;
mod filters;
mod iterators;
mod query_data;
mod query_types;
mod scene_query;
mod selectors;

pub(crate) use arities::has_duplicate_type_ids;
pub(crate) use builders::{build_query_accesses, build_query_accesses_mut};
pub(crate) use fetch::{EntityFetch, QueryAccess, ReadFetch, WriteFetch};
pub(crate) use filters::{
    typed_filters, ArchetypeFilterMatch, QueryFilterExpr, QueryFilterSet, QueryFilterState,
    ResolvedChangeFilter, ResolvedQueryFilter, ResolvedTemporalCountFilter, TemporalFilterKind,
};
pub use filters::{
    Added, AddedAll, AddedAny, AddedAtLeast, AddedAtMost, AddedExactly, AtLeast, AtMost, Changed,
    ChangedAll, ChangedAny, ChangedAtLeast, ChangedAtMost, ChangedExactly, Count, Exactly, Not, Or,
    QueryParam, Spawned, With, WithAll, WithAny, Without, WithoutAll, WithoutAny,
};
pub use query_data::QueryItem;
pub(crate) use query_data::{
    QueryComponent, QueryData, QueryElement, QueryElementInfo, ReadQueryElement,
    MAX_QUERY_COMPONENTS,
};
pub use query_types::Query;
pub(crate) use selectors::QueryRange;
pub use selectors::{First, Last, Range, Skip, Take};

pub(crate) fn uses_candidate_ranges(scene: &Scene, type_id: TypeId) -> bool {
    let targets = scene.query_targets(type_id);
    targets.is_empty()
        || targets
            .iter()
            .any(|target| target.component_type != type_id)
}

pub(crate) fn concrete_row_ranges(scene: &Scene, components: &[QueryComponent]) -> Vec<QueryRange> {
    components
        .iter()
        .filter(|component| !uses_candidate_ranges(scene, component.type_id))
        .flat_map(|component| component.ranges.iter().copied())
        .collect()
}

/// Describes data that can be fetched by a read-only query.
pub trait QuerySpec<'a> {
    type Data;
    type Filters;

    fn build(scene: &'a Scene) -> Query<'a, Self::Data, Self::Filters>;
}

/// Describes data that can be fetched by a query.
pub trait QuerySpecMut<'a> {
    type Data;
    type Filters;

    fn build(scene: &'a mut Scene) -> Query<'a, Self::Data, Self::Filters>;
}
