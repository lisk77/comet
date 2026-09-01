use super::*;

mod expression;

use expression::simple_filters;
pub(crate) use expression::{
    ArchetypeFilterMatch, QueryFilterExpr, QueryFilterState, ResolvedChangeFilter,
    ResolvedQueryFilter,
};

pub struct QueryParam<Data, Filters = ()>(PhantomData<(Data, Filters)>);

pub struct With<C: ?Sized + Component>(PhantomData<C>);
pub struct Added<C: ?Sized + Component>(PhantomData<C>);
pub struct Changed<C: ?Sized + Component>(PhantomData<C>);
pub struct Or<Filters>(PhantomData<Filters>);
pub struct Not<Filter>(PhantomData<Filter>);

#[doc(hidden)]
pub trait WithFilterTuple {
    type Filters;
}

#[doc(hidden)]
pub trait AddedFilterTuple {
    type Filters;
}

#[doc(hidden)]
pub trait ChangedFilterTuple {
    type Filters;
}

impl WithFilterTuple for () {
    type Filters = ();
}

impl AddedFilterTuple for () {
    type Filters = ();
}

impl ChangedFilterTuple for () {
    type Filters = ();
}

macro_rules! impl_filter_tuple {
    ($($name:ident),+) => {
        impl<$($name: Component),+> WithFilterTuple for ($($name,)+) {
            type Filters = ($(With<$name>,)+);
        }

        impl<$($name: Component),+> AddedFilterTuple for ($($name,)+) {
            type Filters = ($(Added<$name>,)+);
        }

        impl<$($name: Component),+> ChangedFilterTuple for ($($name,)+) {
            type Filters = ($(Changed<$name>,)+);
        }
    };
}

impl_filter_tuple!(A);
impl_filter_tuple!(A, B);
impl_filter_tuple!(A, B, C);
impl_filter_tuple!(A, B, C, D);
impl_filter_tuple!(A, B, C, D, E);
impl_filter_tuple!(A, B, C, D, E, F);
impl_filter_tuple!(A, B, C, D, E, F, G);
impl_filter_tuple!(A, B, C, D, E, F, G, H);

pub type Without<C> = Not<With<C>>;
pub type WithAll<Cs> = <Cs as WithFilterTuple>::Filters;
pub type WithAny<Cs> = Or<<Cs as WithFilterTuple>::Filters>;
pub type WithoutAll<Cs> = Not<<Cs as WithFilterTuple>::Filters>;
pub type WithoutAny<Cs> = Not<Or<<Cs as WithFilterTuple>::Filters>>;
pub type AddedAny<Cs> = Or<<Cs as AddedFilterTuple>::Filters>;
pub type ChangedAny<Cs> = Or<<Cs as ChangedFilterTuple>::Filters>;

pub(crate) trait QueryFilterSet {
    fn expression(scene: &Scene) -> QueryFilterExpr;
}

impl QueryFilterSet for () {
    fn expression(_scene: &Scene) -> QueryFilterExpr {
        QueryFilterExpr::True
    }
}

impl<C: ?Sized + Component> QueryFilterSet for With<C> {
    fn expression(_scene: &Scene) -> QueryFilterExpr {
        QueryFilterExpr::With(TypeId::of::<C>())
    }
}

impl<C: ?Sized + Component> QueryFilterSet for Added<C> {
    fn expression(scene: &Scene) -> QueryFilterExpr {
        QueryFilterExpr::Added(TypeId::of::<C>(), scene.default_query_since_tick())
    }
}

impl<C: ?Sized + Component> QueryFilterSet for Changed<C> {
    fn expression(scene: &Scene) -> QueryFilterExpr {
        QueryFilterExpr::Changed(TypeId::of::<C>(), scene.default_query_since_tick())
    }
}

impl<Filters: QueryFilterList> QueryFilterSet for Or<Filters> {
    fn expression(scene: &Scene) -> QueryFilterExpr {
        QueryFilterExpr::or(Filters::expressions(scene))
    }
}

impl<Filter: QueryFilterSet> QueryFilterSet for Not<Filter> {
    fn expression(scene: &Scene) -> QueryFilterExpr {
        QueryFilterExpr::not(Filter::expression(scene))
    }
}

pub(crate) trait QueryFilterList {
    fn expressions(scene: &Scene) -> Vec<QueryFilterExpr>;
}

impl QueryFilterList for () {
    fn expressions(_scene: &Scene) -> Vec<QueryFilterExpr> {
        Vec::new()
    }
}

macro_rules! impl_query_filter_set_tuple {
    ($($name:ident),+) => {
        impl<$($name: QueryFilterSet),+> QueryFilterList for ($($name,)+) {
            fn expressions(scene: &Scene) -> Vec<QueryFilterExpr> {
                vec![$($name::expression(scene)),+]
            }
        }

        impl<$($name: QueryFilterSet),+> QueryFilterSet for ($($name,)+) {
            fn expression(scene: &Scene) -> QueryFilterExpr {
                QueryFilterExpr::and(<Self as QueryFilterList>::expressions(scene))
            }
        }
    };
}

impl_query_filter_set_tuple!(A);
impl_query_filter_set_tuple!(A, B);
impl_query_filter_set_tuple!(A, B, C);
impl_query_filter_set_tuple!(A, B, C, D);
impl_query_filter_set_tuple!(A, B, C, D, E);
impl_query_filter_set_tuple!(A, B, C, D, E, F);
impl_query_filter_set_tuple!(A, B, C, D, E, F, G);
impl_query_filter_set_tuple!(A, B, C, D, E, F, G, H);

pub(crate) fn typed_filters<Filters: QueryFilterSet>(scene: &Scene) -> QueryFilterState {
    let expression = Filters::expression(scene);
    let simple = simple_filters(&expression);
    QueryFilterState { expression, simple }
}
