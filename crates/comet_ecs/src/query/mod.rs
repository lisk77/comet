use crate::component_changes::ComponentChangeState;
use crate::{Component, Entity, Scene, Tick};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::sync::Arc;

mod aggregates;
mod arities;
mod builders;
mod fetch;
mod filters;
mod iterators;
mod query_data;
mod query_types;
mod scene_query;
mod selectors;

pub use aggregates::Amount;
pub(crate) use arities::has_duplicate_type_ids;
pub(crate) use builders::{build_query_accesses, build_query_accesses_mut};
pub(crate) use fetch::{EntityFetch, QueryAccess, ReadFetch, WriteFetch};
pub(crate) use filters::{
    typed_filters, ArchetypeFilterMatch, QueryFilterExpr, QueryFilterSet, QueryFilterState,
    ResolvedChangeFilter, ResolvedQueryFilter, ResolvedTemporalCountFilter, TemporalFilterKind,
};
pub use filters::{
    Added, AddedAll, AddedAny, AddedAtLeast, AddedAtLeastOf, AddedAtMost, AddedAtMostOf,
    AddedExactly, AddedExactlyOf, AtLeast, AtLeastOf, AtMost, AtMostOf, Changed, ChangedAll,
    ChangedAny, ChangedAtLeast, ChangedAtLeastOf, ChangedAtMost, ChangedAtMostOf, ChangedExactly,
    ChangedExactlyOf, Count, CountOf, Exactly, ExactlyOf, Modified, ModifiedAll, ModifiedAny,
    ModifiedAtLeast, ModifiedAtLeastOf, ModifiedAtMost, ModifiedAtMostOf, ModifiedExactly,
    ModifiedExactlyOf, Not, Or, QueryParam, Spawned, With, WithAll, WithAny, Without, WithoutAll,
    WithoutAny,
};
pub use query_data::QueryItem;
pub(crate) use query_data::{
    QueryAmount, QueryComponent, QueryData, QueryElement, QueryElementInfo, QueryLayout,
    ReadQueryElement, MAX_QUERY_COMPONENTS,
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

pub(crate) fn concrete_row_ranges(
    scene: &Scene,
    components: &[QueryComponent],
    amounts: &[QueryAmount],
) -> Vec<QueryRange> {
    let mut ranges = components
        .iter()
        .filter(|component| {
            !component.ranges.is_empty() && !uses_candidate_ranges(scene, component.type_id)
        })
        .map(|component| (component.row_order, component.ranges.as_slice()))
        .chain(
            amounts
                .iter()
                .filter(|amount| !amount.ranges.is_empty())
                .map(|amount| (amount.row_order, amount.ranges.as_slice())),
        )
        .collect::<Vec<_>>();
    if ranges.len() > 1 {
        ranges.sort_unstable_by_key(|(order, _)| *order);
    }
    ranges
        .into_iter()
        .flat_map(|(_, ranges)| ranges.iter().copied())
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
