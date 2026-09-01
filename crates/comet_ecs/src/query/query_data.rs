use super::*;

pub(crate) const MAX_QUERY_COMPONENTS: usize = 8;

#[derive(Clone)]
pub(crate) struct QueryComponent {
    pub(crate) type_id: TypeId,
    pub(crate) required: bool,
    pub(crate) writes: bool,
    pub(crate) selectors: Vec<QuerySelector>,
}

impl QueryComponent {
    fn of<'a, T: WriteFetch<'a>>() -> Self {
        Self {
            type_id: T::type_id(),
            required: T::required(),
            writes: T::writes(),
            selectors: T::selectors(),
        }
    }
}

#[doc(hidden)]
pub trait QueryItem<'a> {
    type Item;
}

impl<'a, C: ?Sized + Component> QueryItem<'a> for &'a C {
    type Item = &'a C;
}

impl<'a, C: ?Sized + Component> QueryItem<'a> for &'a mut C {
    type Item = &'a mut C;
}

impl<'a, T: QueryItem<'a>> QueryItem<'a> for Option<T> {
    type Item = Option<T::Item>;
}

pub(crate) trait QueryData<'a>: QueryItem<'a> + Sized {
    fn components() -> Vec<QueryComponent>;

    unsafe fn mark_changed(
        change_state: *mut HashMap<(u32, TypeId), ComponentChangeState>,
        component_event_tick: Tick,
        entity: Entity,
        columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
        component_types: &[Option<TypeId>; MAX_QUERY_COMPONENTS],
    );

    unsafe fn fetch(
        entity: Entity,
        columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
        casters: &[Option<crate::QueryCaster>; MAX_QUERY_COMPONENTS],
        row: usize,
    ) -> Option<Self::Item>;
}

pub(crate) trait ReadQueryData<'a>: QueryData<'a> {}

unsafe fn mark_component_changed(
    change_state: *mut HashMap<(u32, TypeId), ComponentChangeState>,
    component_event_tick: Tick,
    entity: Entity,
    type_id: TypeId,
) {
    let change_state = unsafe { &mut *change_state };
    let key = (entity.index, type_id);
    if let Some(state) = change_state.get_mut(&key) {
        state.changed_tick = component_event_tick;
    } else {
        change_state.insert(
            key,
            ComponentChangeState {
                added_tick: component_event_tick,
                changed_tick: component_event_tick,
            },
        );
    }
}

macro_rules! impl_query_data_leaf {
    ([$($generic:tt)*] $data:ty) => {
        impl<'a, $($generic)*> QueryData<'a> for $data
        where
            $data: WriteFetch<'a>,
        {
            fn components() -> Vec<QueryComponent> {
                vec![QueryComponent::of::<Self>()]
            }

            unsafe fn mark_changed(
                change_state: *mut HashMap<(u32, TypeId), ComponentChangeState>,
                component_event_tick: Tick,
                entity: Entity,
                columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
                component_types: &[Option<TypeId>; MAX_QUERY_COMPONENTS],
            ) {
                if <Self as WriteFetch<'a>>::writes() && !columns[0].is_null() {
                    unsafe {
                        mark_component_changed(
                            change_state,
                            component_event_tick,
                            entity,
                            component_types[0].expect("query access is missing its component type"),
                        );
                    }
                }
            }

            unsafe fn fetch(
                _entity: Entity,
                columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
                casters: &[Option<crate::QueryCaster>; MAX_QUERY_COMPONENTS],
                row: usize,
            ) -> Option<Self::Item> {
                unsafe { <Self as WriteFetch<'a>>::get(columns[0], casters[0].as_ref(), row) }
            }
        }

        impl<'a, $($generic)*> ReadQueryData<'a> for $data
        where
            $data: ReadFetch<'a> + WriteFetch<'a>,
        {
        }
    };
}

impl_query_data_leaf!([C: ?Sized + Component] &'a C);
impl_query_data_leaf!([C: ?Sized + Component] &'a mut C);
impl_query_data_leaf!([T] Option<T>);
impl_query_data_leaf!([T, const START: usize, const END: usize] Range<T, START, END>);
impl_query_data_leaf!([T, const COUNT: usize] Skip<T, COUNT>);
impl_query_data_leaf!([T, const COUNT: usize] Take<T, COUNT>);
impl_query_data_leaf!([T] First<T>);
impl_query_data_leaf!([T] Last<T>);

macro_rules! impl_tuple_query_data {
    ($($ty:ident: $index:literal),+) => {
        impl<'a, $($ty),+> QueryItem<'a> for ($($ty,)+)
        where
            $($ty: QueryItem<'a> + 'a),+
        {
            type Item = ($(<$ty as QueryItem<'a>>::Item,)+);
        }

        impl<'a, $($ty),+> QueryData<'a> for ($($ty,)+)
        where
            $($ty: WriteFetch<'a> + 'a),+
        {
            fn components() -> Vec<QueryComponent> {
                vec![$(QueryComponent::of::<$ty>()),+]
            }

            unsafe fn mark_changed(
                change_state: *mut HashMap<(u32, TypeId), ComponentChangeState>,
                component_event_tick: Tick,
                entity: Entity,
                columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
                component_types: &[Option<TypeId>; MAX_QUERY_COMPONENTS],
            ) {
                $(
                    if <$ty as WriteFetch<'a>>::writes() && !columns[$index].is_null() {
                        unsafe {
                            mark_component_changed(
                                change_state,
                                component_event_tick,
                                entity,
                                component_types[$index]
                                    .expect("query access is missing its component type"),
                            );
                        }
                    }
                )+
            }

            unsafe fn fetch(
                _entity: Entity,
                columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
                casters: &[Option<crate::QueryCaster>; MAX_QUERY_COMPONENTS],
                row: usize,
            ) -> Option<Self::Item> {
                unsafe {
                    Some(($(<$ty as WriteFetch<'a>>::get(
                        columns[$index],
                        casters[$index].as_ref(),
                        row,
                    )?,)+))
                }
            }
        }

        impl<'a, $($ty),+> ReadQueryData<'a> for ($($ty,)+)
        where
            $($ty: ReadFetch<'a> + WriteFetch<'a> + 'a),+
        {}
    };
}

macro_rules! impl_entity_tuple_query_data {
    ($($ty:ident: $index:literal),+) => {
        impl<'a, $($ty),+> QueryItem<'a> for (Entity, $($ty,)+)
        where
            $($ty: QueryItem<'a> + 'a),+
        {
            type Item = (Entity, $(<$ty as QueryItem<'a>>::Item,)+);
        }

        impl<'a, $($ty),+> QueryData<'a> for (Entity, $($ty,)+)
        where
            $($ty: WriteFetch<'a> + 'a),+
        {
            fn components() -> Vec<QueryComponent> {
                vec![$(QueryComponent::of::<$ty>()),+]
            }

            unsafe fn mark_changed(
                change_state: *mut HashMap<(u32, TypeId), ComponentChangeState>,
                component_event_tick: Tick,
                entity: Entity,
                columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
                component_types: &[Option<TypeId>; MAX_QUERY_COMPONENTS],
            ) {
                $(
                    if <$ty as WriteFetch<'a>>::writes() && !columns[$index].is_null() {
                        unsafe {
                            mark_component_changed(
                                change_state,
                                component_event_tick,
                                entity,
                                component_types[$index]
                                    .expect("query access is missing its component type"),
                            );
                        }
                    }
                )+
            }

            unsafe fn fetch(
                entity: Entity,
                columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
                casters: &[Option<crate::QueryCaster>; MAX_QUERY_COMPONENTS],
                row: usize,
            ) -> Option<Self::Item> {
                unsafe {
                    Some((entity, $(<$ty as WriteFetch<'a>>::get(
                        columns[$index],
                        casters[$index].as_ref(),
                        row,
                    )?,)+))
                }
            }
        }

        impl<'a, $($ty),+> ReadQueryData<'a> for (Entity, $($ty,)+)
        where
            $($ty: ReadFetch<'a> + WriteFetch<'a> + 'a),+
        {}
    };
}

impl_tuple_query_data!(A: 0, B: 1);
impl_tuple_query_data!(A: 0, B: 1, C: 2);
impl_tuple_query_data!(A: 0, B: 1, C: 2, D: 3);
impl_tuple_query_data!(A: 0, B: 1, C: 2, D: 3, E: 4);
impl_tuple_query_data!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5);
impl_tuple_query_data!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6);
impl_tuple_query_data!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7);

impl_entity_tuple_query_data!(A: 0);
impl_entity_tuple_query_data!(A: 0, B: 1);
impl_entity_tuple_query_data!(A: 0, B: 1, C: 2);
impl_entity_tuple_query_data!(A: 0, B: 1, C: 2, D: 3);
impl_entity_tuple_query_data!(A: 0, B: 1, C: 2, D: 3, E: 4);
impl_entity_tuple_query_data!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5);
impl_entity_tuple_query_data!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6);
impl_entity_tuple_query_data!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7);

impl<'a, Data, Filters> QuerySpec<'a> for QueryParam<Data, Filters>
where
    Data: ReadQueryData<'a> + 'a,
    Filters: QueryFilterSet,
{
    type Data = Data;
    type Filters = Filters;

    fn build(scene: &'a Scene) -> Query<'a, Data, Filters> {
        Query::from_state(scene, typed_filters::<Filters>(scene))
    }
}

impl<'a, Data, Filters> QuerySpecMut<'a> for QueryParam<Data, Filters>
where
    Data: QueryData<'a> + 'a,
    Filters: QueryFilterSet,
{
    type Data = Data;
    type Filters = Filters;

    fn build(scene: &'a mut Scene) -> Query<'a, Data, Filters> {
        let state = typed_filters::<Filters>(scene);
        Query::from_state_mut(scene, state)
    }
}
