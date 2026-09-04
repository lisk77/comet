use super::*;

pub(crate) const MAX_QUERY_COMPONENTS: usize = 8;

#[derive(Clone)]
pub(crate) struct QueryComponent {
    pub(crate) type_id: TypeId,
    pub(crate) required: bool,
    pub(crate) writes: bool,
    pub(crate) ranges: Vec<QueryRange>,
    pub(crate) row_order: usize,
}

impl QueryComponent {
    fn of<'a, T: WriteFetch<'a>>() -> Self {
        Self {
            type_id: T::type_id(),
            required: T::required(),
            writes: T::writes(),
            ranges: T::ranges(),
            row_order: 0,
        }
    }
}

pub(crate) struct QueryAmount {
    pub(crate) type_id: TypeId,
    pub(crate) ranges: Vec<QueryRange>,
    pub(crate) row_order: usize,
}

pub(crate) enum QueryElementInfo {
    Entity(Vec<QueryRange>),
    Component(QueryComponent),
    Amount(QueryAmount),
}

pub(crate) struct QueryLayout {
    pub(crate) components: Vec<QueryComponent>,
    pub(crate) amounts: Vec<QueryAmount>,
    pub(crate) entity_ranges: Vec<QueryRange>,
    row_order: usize,
}

impl QueryLayout {
    fn empty() -> Self {
        Self {
            components: Vec::new(),
            amounts: Vec::new(),
            entity_ranges: Vec::new(),
            row_order: 0,
        }
    }

    fn push(&mut self, info: QueryElementInfo) {
        match info {
            QueryElementInfo::Entity(ranges) => {
                if !ranges.is_empty() {
                    assert!(
                        self.entity_ranges.is_empty(),
                        "query cannot contain multiple ranged entity fetches"
                    );
                    self.entity_ranges = ranges;
                }
            }
            QueryElementInfo::Component(mut component) => {
                component.row_order = self.row_order;
                self.row_order += 1;
                self.components.push(component);
            }
            QueryElementInfo::Amount(mut amount) => {
                amount.row_order = self.row_order;
                self.row_order += 1;
                self.amounts.push(amount);
            }
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

impl<'a> QueryItem<'a> for Entity {
    type Item = Entity;
}

pub(crate) trait QueryElement<'a>: QueryItem<'a> {
    fn info() -> QueryElementInfo;

    unsafe fn mark_changed(
        change_state: *mut HashMap<(u32, TypeId), ComponentChangeState>,
        component_event_tick: Tick,
        entity: Entity,
        columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
        component_types: &[Option<TypeId>; MAX_QUERY_COMPONENTS],
        component_slot: &mut usize,
    );

    unsafe fn fetch(
        entity: Entity,
        columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
        casters: &[Option<crate::QueryCaster>; MAX_QUERY_COMPONENTS],
        amounts: &[usize],
        row: usize,
        component_slot: &mut usize,
        amount_slot: &mut usize,
    ) -> Option<Self::Item>;
}

pub(crate) trait ReadQueryElement<'a>: QueryElement<'a> {}

pub(crate) trait QueryData<'a>: QueryItem<'a> + Sized {
    fn layout() -> QueryLayout;

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
        amounts: &[usize],
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

macro_rules! impl_component_query_element {
    ($data:ty) => {
        impl<'a, C: ?Sized + Component> QueryElement<'a> for $data {
            fn info() -> QueryElementInfo {
                QueryElementInfo::Component(QueryComponent::of::<Self>())
            }

            unsafe fn mark_changed(
                change_state: *mut HashMap<(u32, TypeId), ComponentChangeState>,
                component_event_tick: Tick,
                entity: Entity,
                columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
                component_types: &[Option<TypeId>; MAX_QUERY_COMPONENTS],
                component_slot: &mut usize,
            ) {
                let slot = *component_slot;
                *component_slot += 1;
                if <Self as WriteFetch<'a>>::writes() && !columns[slot].is_null() {
                    unsafe {
                        mark_component_changed(
                            change_state,
                            component_event_tick,
                            entity,
                            component_types[slot]
                                .expect("query access is missing its component type"),
                        );
                    }
                }
            }

            unsafe fn fetch(
                _entity: Entity,
                columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
                casters: &[Option<crate::QueryCaster>; MAX_QUERY_COMPONENTS],
                _amounts: &[usize],
                row: usize,
                component_slot: &mut usize,
                _amount_slot: &mut usize,
            ) -> Option<Self::Item> {
                let slot = *component_slot;
                *component_slot += 1;
                unsafe { <Self as WriteFetch<'a>>::get(columns[slot], casters[slot].as_ref(), row) }
            }
        }
    };
}

impl_component_query_element!(&'a C);
impl_component_query_element!(&'a mut C);

impl<'a, C: ?Sized + Component> ReadQueryElement<'a> for &'a C {}

impl<'a, T> QueryElement<'a> for Option<T>
where
    Option<T>: WriteFetch<'a>,
{
    fn info() -> QueryElementInfo {
        QueryElementInfo::Component(QueryComponent::of::<Self>())
    }

    unsafe fn mark_changed(
        change_state: *mut HashMap<(u32, TypeId), ComponentChangeState>,
        component_event_tick: Tick,
        entity: Entity,
        columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
        component_types: &[Option<TypeId>; MAX_QUERY_COMPONENTS],
        component_slot: &mut usize,
    ) {
        let slot = *component_slot;
        *component_slot += 1;
        if <Self as WriteFetch<'a>>::writes() && !columns[slot].is_null() {
            unsafe {
                mark_component_changed(
                    change_state,
                    component_event_tick,
                    entity,
                    component_types[slot].expect("query access is missing its component type"),
                );
            }
        }
    }

    unsafe fn fetch(
        _entity: Entity,
        columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
        casters: &[Option<crate::QueryCaster>; MAX_QUERY_COMPONENTS],
        _amounts: &[usize],
        row: usize,
        component_slot: &mut usize,
        _amount_slot: &mut usize,
    ) -> Option<Self::Item> {
        let slot = *component_slot;
        *component_slot += 1;
        unsafe { <Self as WriteFetch<'a>>::get(columns[slot], casters[slot].as_ref(), row) }
    }
}

impl<'a, T> ReadQueryElement<'a> for Option<T> where Option<T>: ReadFetch<'a> + WriteFetch<'a> {}

impl<'a> QueryElement<'a> for Entity {
    fn info() -> QueryElementInfo {
        QueryElementInfo::Entity(Vec::new())
    }

    unsafe fn mark_changed(
        _change_state: *mut HashMap<(u32, TypeId), ComponentChangeState>,
        _component_event_tick: Tick,
        _entity: Entity,
        _columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
        _component_types: &[Option<TypeId>; MAX_QUERY_COMPONENTS],
        _component_slot: &mut usize,
    ) {
    }

    unsafe fn fetch(
        entity: Entity,
        _columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
        _casters: &[Option<crate::QueryCaster>; MAX_QUERY_COMPONENTS],
        _amounts: &[usize],
        _row: usize,
        _component_slot: &mut usize,
        _amount_slot: &mut usize,
    ) -> Option<Self::Item> {
        Some(entity)
    }
}

impl<'a> ReadQueryElement<'a> for Entity {}

macro_rules! impl_query_data_element {
    ([$($generic:tt)*] $data:ty) => {
        impl<'a, $($generic)*> QueryData<'a> for $data
        where
            $data: QueryElement<'a>,
        {
            fn layout() -> QueryLayout {
                let mut layout = QueryLayout::empty();
                layout.push(<Self as QueryElement<'a>>::info());
                layout
            }

            unsafe fn mark_changed(
                change_state: *mut HashMap<(u32, TypeId), ComponentChangeState>,
                component_event_tick: Tick,
                entity: Entity,
                columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
                component_types: &[Option<TypeId>; MAX_QUERY_COMPONENTS],
            ) {
                let mut component_slot = 0;
                unsafe {
                    <Self as QueryElement<'a>>::mark_changed(
                        change_state,
                        component_event_tick,
                        entity,
                        columns,
                        component_types,
                        &mut component_slot,
                    );
                }
            }

            unsafe fn fetch(
                entity: Entity,
                columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
                casters: &[Option<crate::QueryCaster>; MAX_QUERY_COMPONENTS],
                amounts: &[usize],
                row: usize,
            ) -> Option<Self::Item> {
                let mut component_slot = 0;
                let mut amount_slot = 0;
                unsafe {
                    <Self as QueryElement<'a>>::fetch(
                        entity,
                        columns,
                        casters,
                        amounts,
                        row,
                        &mut component_slot,
                        &mut amount_slot,
                    )
                }
            }
        }

        impl<'a, $($generic)*> ReadQueryData<'a> for $data
        where
            $data: ReadQueryElement<'a>,
        {
        }
    };
}

impl_query_data_element!([C: ?Sized + Component] &'a C);
impl_query_data_element!([C: ?Sized + Component] &'a mut C);
impl_query_data_element!([C: ?Sized + Component] Amount<&'a C>);
impl_query_data_element!([T] Option<T>);
impl_query_data_element!([T, const START: usize, const END: usize] Range<T, START, END>);
impl_query_data_element!([T] Last<T>);

impl<'a> QueryData<'a> for Entity {
    fn layout() -> QueryLayout {
        QueryLayout::empty()
    }

    unsafe fn mark_changed(
        _change_state: *mut HashMap<(u32, TypeId), ComponentChangeState>,
        _component_event_tick: Tick,
        _entity: Entity,
        _columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
        _component_types: &[Option<TypeId>; MAX_QUERY_COMPONENTS],
    ) {
    }

    unsafe fn fetch(
        entity: Entity,
        _columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
        _casters: &[Option<crate::QueryCaster>; MAX_QUERY_COMPONENTS],
        _amounts: &[usize],
        _row: usize,
    ) -> Option<Self::Item> {
        Some(entity)
    }
}

impl<'a> ReadQueryData<'a> for Entity {}

macro_rules! impl_tuple_query_data {
    ($($ty:ident),+) => {
        impl<'a, $($ty),+> QueryItem<'a> for ($($ty,)+)
        where
            $($ty: QueryItem<'a> + 'a),+
        {
            type Item = ($(<$ty as QueryItem<'a>>::Item,)+);
        }

        impl<'a, $($ty),+> QueryData<'a> for ($($ty,)+)
        where
            $($ty: QueryElement<'a> + 'a),+
        {
            fn layout() -> QueryLayout {
                let mut layout = QueryLayout::empty();
                $(layout.push(<$ty as QueryElement<'a>>::info());)+
                layout
            }

            unsafe fn mark_changed(
                change_state: *mut HashMap<(u32, TypeId), ComponentChangeState>,
                component_event_tick: Tick,
                entity: Entity,
                columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
                component_types: &[Option<TypeId>; MAX_QUERY_COMPONENTS],
            ) {
                let mut component_slot = 0;
                $(
                    unsafe {
                        <$ty as QueryElement<'a>>::mark_changed(
                            change_state,
                            component_event_tick,
                            entity,
                            columns,
                            component_types,
                            &mut component_slot,
                        );
                    }
                )+
            }

            unsafe fn fetch(
                entity: Entity,
                columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
                casters: &[Option<crate::QueryCaster>; MAX_QUERY_COMPONENTS],
                amounts: &[usize],
                row: usize,
            ) -> Option<Self::Item> {
                let mut component_slot = 0;
                let mut amount_slot = 0;
                unsafe {
                    Some(($(<$ty as QueryElement<'a>>::fetch(
                        entity,
                        columns,
                        casters,
                        amounts,
                        row,
                        &mut component_slot,
                        &mut amount_slot,
                    )?,)+))
                }
            }
        }

        impl<'a, $($ty),+> ReadQueryData<'a> for ($($ty,)+)
        where
            $($ty: ReadQueryElement<'a> + 'a),+
        {
        }
    };
}

impl_tuple_query_data!(A);
impl_tuple_query_data!(A, B);
impl_tuple_query_data!(A, B, C);
impl_tuple_query_data!(A, B, C, D);
impl_tuple_query_data!(A, B, C, D, E);
impl_tuple_query_data!(A, B, C, D, E, F);
impl_tuple_query_data!(A, B, C, D, E, F, G);
impl_tuple_query_data!(A, B, C, D, E, F, G, H);
impl_tuple_query_data!(A, B, C, D, E, F, G, H, I);

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
