use crate::component_changes::ComponentChangeState;
use crate::{Component, ComponentTuple, Entity, Scene, Tick};
use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;

mod arities;
mod builders;
mod fetch;
mod filters;
mod iterators;
mod query_data;
mod query_types;
mod scene_query;

pub(crate) use arities::has_duplicate_type_ids;
pub(crate) use builders::{build_query_accesses, build_query_accesses_mut};
pub(crate) use fetch::{EntityFetch, QueryAccess, ReadFetch, WriteFetch};
pub(crate) use filters::{typed_filters, QueryFilterSet, QueryFilterState, ResolvedChangeFilter};
pub use filters::{Added, Changed, QueryParam, With, WithAny, Without, WithoutAny};
pub(crate) use query_data::{QueryComponent, QueryData, MAX_QUERY_COMPONENTS};
pub use query_types::Query;

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
