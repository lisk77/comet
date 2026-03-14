use crate::{Component, ComponentTuple, Entity, Scene, Tick};
use std::any::TypeId;
use std::marker::PhantomData;

#[macro_use]
mod arities;
mod builders;
mod fetch;
mod filters;
mod iterators;
mod query_types;
mod scene_query;
mod tuple_types;

pub(crate) use arities::has_duplicate_type_ids;
pub(crate) use fetch::{EntityFetch, QueryAccess, QueryMutAccess, ReadFetch, WriteFetch};
pub(crate) use filters::{typed_filters, QueryFilterSet, QueryFilterState};
pub use filters::{Added, Changed, QueryParam, With, WithAny, Without, WithoutAny};
pub use query_types::Query;
pub(crate) use query_types::{
    QueryBuilder, QueryBuilderFiltered, QueryFiltered, QueryIter, QueryIterFiltered, QueryIterMut,
    QueryIterMutFiltered,
};

pub trait QuerySpec<'a> {
    type Builder;
    fn build(scene: &'a Scene) -> Self::Builder;
}

pub trait QuerySpecMut<'a> {
    type Builder;
    fn build(scene: &'a mut Scene) -> Self::Builder;
}
