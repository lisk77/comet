use super::*;

mod expression;

use expression::simple_filters;
pub(crate) use expression::{
    ArchetypeFilterMatch, QueryFilterExpr, QueryFilterState, ResolvedChangeFilter,
    ResolvedQueryFilter, ResolvedTemporalCountFilter, TemporalFilterKind,
};

pub struct QueryParam<Data, Filters = ()>(PhantomData<(Data, Filters)>);

pub struct Count<C: ?Sized + Component, const MIN: usize, const MAX: usize, Condition = ()>(
    PhantomData<C>,
    PhantomData<Condition>,
);
pub struct CountOf<Components, const MIN: usize, const MAX: usize, Condition = ()>(
    PhantomData<Components>,
    PhantomData<Condition>,
);
pub struct Added<C: ?Sized>(PhantomData<C>);
pub struct Changed<C: ?Sized>(PhantomData<C>);
pub struct Spawned;
pub struct Or<Filters>(PhantomData<Filters>);
pub struct Not<Filter>(PhantomData<Filter>);

#[doc(hidden)]
pub trait FilterTuple {
    type With;
    type Added;
    type Changed;
}

pub(crate) trait FilterTupleInfo {
    fn type_ids() -> Vec<TypeId>;
}

impl FilterTuple for () {
    type With = ();
    type Added = ();
    type Changed = ();
}

impl FilterTupleInfo for () {
    fn type_ids() -> Vec<TypeId> {
        Vec::new()
    }
}

macro_rules! impl_filter_tuple {
    ($($name:ident),+) => {
        impl<$($name: Component),+> FilterTuple for ($($name,)+) {
            type With = ($(With<$name>,)+);
            type Added = ($(Added<$name>,)+);
            type Changed = ($(Changed<$name>,)+);
        }

        impl<$($name: Component),+> FilterTupleInfo for ($($name,)+) {
            fn type_ids() -> Vec<TypeId> {
                vec![$(TypeId::of::<$name>()),+]
            }
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

pub type Exactly<C, const COUNT: usize> = Count<C, COUNT, COUNT>;
pub type AtLeast<C, const COUNT: usize> = Count<C, COUNT, { usize::MAX }>;
pub type AtMost<C, const COUNT: usize> = Count<C, 0, COUNT>;

pub type ExactlyOf<Components, const COUNT: usize> = CountOf<Components, COUNT, COUNT>;
pub type AtLeastOf<Components, const COUNT: usize> = CountOf<Components, COUNT, { usize::MAX }>;
pub type AtMostOf<Components, const COUNT: usize> = CountOf<Components, 0, COUNT>;
pub type With<C> = AtLeast<C, 1>;
pub type Without<C> = Exactly<C, 0>;
pub type WithAll<Cs> = <Cs as FilterTuple>::With;
pub type WithAny<Cs> = Or<<Cs as FilterTuple>::With>;
pub type WithoutAll<Cs> = Not<<Cs as FilterTuple>::With>;
pub type WithoutAny<Cs> = Not<Or<<Cs as FilterTuple>::With>>;
pub type AddedAll<Cs> = <Cs as FilterTuple>::Added;
pub type AddedAny<Cs> = Or<<Cs as FilterTuple>::Added>;
pub type ChangedAll<Cs> = <Cs as FilterTuple>::Changed;
pub type ChangedAny<Cs> = Or<<Cs as FilterTuple>::Changed>;
pub type AddedExactly<C, const COUNT: usize> = Count<C, COUNT, COUNT, Added<C>>;
pub type AddedAtLeast<C, const COUNT: usize> = Count<C, COUNT, { usize::MAX }, Added<C>>;
pub type AddedAtMost<C, const COUNT: usize> = Count<C, 0, COUNT, Added<C>>;
pub type AddedExactlyOf<Components, const COUNT: usize> =
    CountOf<Components, COUNT, COUNT, Added<Components>>;
pub type AddedAtLeastOf<Components, const COUNT: usize> =
    CountOf<Components, COUNT, { usize::MAX }, Added<Components>>;
pub type AddedAtMostOf<Components, const COUNT: usize> =
    CountOf<Components, 0, COUNT, Added<Components>>;
pub type ChangedExactly<C, const COUNT: usize> = Count<C, COUNT, COUNT, Changed<C>>;
pub type ChangedAtLeast<C, const COUNT: usize> = Count<C, COUNT, { usize::MAX }, Changed<C>>;
pub type ChangedAtMost<C, const COUNT: usize> = Count<C, 0, COUNT, Changed<C>>;
pub type ChangedExactlyOf<Components, const COUNT: usize> =
    CountOf<Components, COUNT, COUNT, Changed<Components>>;
pub type ChangedAtLeastOf<Components, const COUNT: usize> =
    CountOf<Components, COUNT, { usize::MAX }, Changed<Components>>;
pub type ChangedAtMostOf<Components, const COUNT: usize> =
    CountOf<Components, 0, COUNT, Changed<Components>>;

pub(crate) trait QueryFilterSet {
    fn expression(scene: &Scene) -> QueryFilterExpr;
}

impl QueryFilterSet for () {
    fn expression(_scene: &Scene) -> QueryFilterExpr {
        QueryFilterExpr::True
    }
}

impl<C: ?Sized + Component, const MIN: usize, const MAX: usize> QueryFilterSet
    for Count<C, MIN, MAX>
{
    fn expression(_scene: &Scene) -> QueryFilterExpr {
        QueryFilterExpr::count(vec![TypeId::of::<C>()].into(), MIN, MAX)
    }
}

impl<Components: FilterTupleInfo, const MIN: usize, const MAX: usize> QueryFilterSet
    for CountOf<Components, MIN, MAX>
{
    fn expression(_scene: &Scene) -> QueryFilterExpr {
        QueryFilterExpr::count(Components::type_ids().into(), MIN, MAX)
    }
}

impl<Components: FilterTupleInfo, const MIN: usize, const MAX: usize> QueryFilterSet
    for CountOf<Components, MIN, MAX, Added<Components>>
{
    fn expression(scene: &Scene) -> QueryFilterExpr {
        QueryFilterExpr::temporal_count(
            Components::type_ids().into(),
            MIN,
            MAX,
            scene.default_query_since_tick(),
            TemporalFilterKind::Added,
        )
    }
}

impl<Components: FilterTupleInfo, const MIN: usize, const MAX: usize> QueryFilterSet
    for CountOf<Components, MIN, MAX, Changed<Components>>
{
    fn expression(scene: &Scene) -> QueryFilterExpr {
        QueryFilterExpr::temporal_count(
            Components::type_ids().into(),
            MIN,
            MAX,
            scene.default_query_since_tick(),
            TemporalFilterKind::Changed,
        )
    }
}

impl<C: ?Sized + Component, const MIN: usize, const MAX: usize> QueryFilterSet
    for Count<C, MIN, MAX, Added<C>>
{
    fn expression(scene: &Scene) -> QueryFilterExpr {
        QueryFilterExpr::temporal_count(
            vec![TypeId::of::<C>()].into(),
            MIN,
            MAX,
            scene.default_query_since_tick(),
            TemporalFilterKind::Added,
        )
    }
}

impl<C: ?Sized + Component, const MIN: usize, const MAX: usize> QueryFilterSet
    for Count<C, MIN, MAX, Changed<C>>
{
    fn expression(scene: &Scene) -> QueryFilterExpr {
        QueryFilterExpr::temporal_count(
            vec![TypeId::of::<C>()].into(),
            MIN,
            MAX,
            scene.default_query_since_tick(),
            TemporalFilterKind::Changed,
        )
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

impl QueryFilterSet for Spawned {
    fn expression(scene: &Scene) -> QueryFilterExpr {
        QueryFilterExpr::Spawned(scene.default_query_since_tick())
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
