use super::*;

pub struct QueryParam<Data, Filters = ()>(PhantomData<(Data, Filters)>);

pub struct With<C: Component>(PhantomData<C>);
pub struct Without<C: Component>(PhantomData<C>);
pub struct WithAny<Cs: ComponentTuple>(PhantomData<Cs>);
pub struct WithoutAny<Cs: ComponentTuple>(PhantomData<Cs>);
pub struct Added<C: Component>(PhantomData<C>);
pub struct Changed<C: Component>(PhantomData<C>);

#[derive(Default)]
pub(crate) struct QueryFilterState {
    pub(crate) with_components: Vec<TypeId>,
    pub(crate) without_components: Vec<TypeId>,
    pub(crate) with_any_components: Vec<TypeId>,
    pub(crate) without_any_components: Vec<TypeId>,
    pub(crate) added_filter: Option<(TypeId, Tick)>,
    pub(crate) changed_filter: Option<(TypeId, Tick)>,
}

impl QueryFilterState {
    pub(crate) fn empty() -> Self {
        Self::default()
    }
}

pub(crate) trait QueryFilterSet {
    fn apply(scene: &Scene, state: &mut QueryFilterState);
}

impl QueryFilterSet for () {
    fn apply(_scene: &Scene, _state: &mut QueryFilterState) {}
}

impl<C: Component> QueryFilterSet for With<C> {
    fn apply(_scene: &Scene, state: &mut QueryFilterState) {
        state.with_components.push(C::type_id());
    }
}

impl<C: Component> QueryFilterSet for Without<C> {
    fn apply(_scene: &Scene, state: &mut QueryFilterState) {
        state.without_components.push(C::type_id());
    }
}

impl<Cs: ComponentTuple> QueryFilterSet for WithAny<Cs> {
    fn apply(_scene: &Scene, state: &mut QueryFilterState) {
        state.with_any_components.extend(Cs::type_ids());
    }
}

impl<Cs: ComponentTuple> QueryFilterSet for WithoutAny<Cs> {
    fn apply(_scene: &Scene, state: &mut QueryFilterState) {
        state.without_any_components.extend(Cs::type_ids());
    }
}

impl<C: Component> QueryFilterSet for Added<C> {
    fn apply(scene: &Scene, state: &mut QueryFilterState) {
        state.added_filter = Some((C::type_id(), scene.query_default_tick()));
    }
}

impl<C: Component> QueryFilterSet for Changed<C> {
    fn apply(scene: &Scene, state: &mut QueryFilterState) {
        state.changed_filter = Some((C::type_id(), scene.query_default_tick()));
    }
}

macro_rules! impl_query_filter_set_tuple {
    ($($name:ident),+) => {
        impl<$($name: QueryFilterSet),+> QueryFilterSet for ($($name,)+) {
            fn apply(scene: &Scene, state: &mut QueryFilterState) {
                $(
                    $name::apply(scene, state);
                )+
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
    let mut state = QueryFilterState::empty();
    Filters::apply(scene, &mut state);
    state
}
