use super::*;

pub(crate) const MAX_QUERY_COMPONENTS: usize = 8;

#[derive(Clone, Copy)]
pub(crate) struct QueryComponent {
    pub(crate) type_id: TypeId,
    pub(crate) required: bool,
}

pub(crate) trait QueryData<'a>: Sized {
    fn components() -> Vec<QueryComponent>;

    unsafe fn mark_changed(
        scene: *mut Scene,
        entity: Entity,
        columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
    );

    unsafe fn fetch(
        entity: Entity,
        columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
        row: usize,
    ) -> Option<Self>;
}

pub(crate) trait ReadQueryData<'a>: QueryData<'a> {}

macro_rules! impl_query_data_leaf {
    ($data:ty) => {
        impl<'a, C: Component> QueryData<'a> for $data {
            fn components() -> Vec<QueryComponent> {
                vec![QueryComponent {
                    type_id: <Self as WriteFetch<'a>>::type_id(),
                    required: <Self as WriteFetch<'a>>::required(),
                }]
            }

            unsafe fn mark_changed(
                scene: *mut Scene,
                entity: Entity,
                columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
            ) {
                if <Self as WriteFetch<'a>>::writes() && !columns[0].is_null() {
                    unsafe {
                        (&mut *scene).mark_component_changed_for_query(
                            entity,
                            <Self as WriteFetch<'a>>::type_id(),
                        );
                    }
                }
            }

            unsafe fn fetch(
                _entity: Entity,
                columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
                row: usize,
            ) -> Option<Self> {
                unsafe { <Self as WriteFetch<'a>>::get(columns[0], row) }
            }
        }
    };
}

impl_query_data_leaf!(&'a C);
impl_query_data_leaf!(&'a mut C);
impl_query_data_leaf!(Option<&'a C>);
impl_query_data_leaf!(Option<&'a mut C>);

impl<'a, C: Component> ReadQueryData<'a> for &'a C {}
impl<'a, C: Component> ReadQueryData<'a> for Option<&'a C> {}

macro_rules! impl_tuple_query_data {
    ($($ty:ident: $index:literal),+) => {
        impl<'a, $($ty),+> QueryData<'a> for ($($ty,)+)
        where
            $($ty: WriteFetch<'a, Item = $ty> + 'a),+
        {
            fn components() -> Vec<QueryComponent> {
                vec![
                    $(QueryComponent {
                        type_id: <$ty as WriteFetch<'a>>::type_id(),
                        required: <$ty as WriteFetch<'a>>::required(),
                    }),+
                ]
            }

            unsafe fn mark_changed(
                scene: *mut Scene,
                entity: Entity,
                columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
            ) {
                $(
                    if <$ty as WriteFetch<'a>>::writes() && !columns[$index].is_null() {
                        unsafe {
                            (&mut *scene).mark_component_changed_for_query(
                                entity,
                                <$ty as WriteFetch<'a>>::type_id(),
                            );
                        }
                    }
                )+
            }

            unsafe fn fetch(
                _entity: Entity,
                columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
                row: usize,
            ) -> Option<Self> {
                unsafe {
                    Some(($(<$ty as WriteFetch<'a>>::get(columns[$index], row)?,)+))
                }
            }
        }

        impl<'a, $($ty),+> ReadQueryData<'a> for ($($ty,)+)
        where
            $($ty: ReadFetch<'a> + WriteFetch<'a, Item = $ty> + 'a),+
        {}
    };
}

macro_rules! impl_entity_tuple_query_data {
    ($($ty:ident: $index:literal),+) => {
        impl<'a, $($ty),+> QueryData<'a> for (Entity, $($ty,)+)
        where
            $($ty: WriteFetch<'a, Item = $ty> + 'a),+
        {
            fn components() -> Vec<QueryComponent> {
                vec![
                    $(QueryComponent {
                        type_id: <$ty as WriteFetch<'a>>::type_id(),
                        required: <$ty as WriteFetch<'a>>::required(),
                    }),+
                ]
            }

            unsafe fn mark_changed(
                scene: *mut Scene,
                entity: Entity,
                columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
            ) {
                $(
                    if <$ty as WriteFetch<'a>>::writes() && !columns[$index].is_null() {
                        unsafe {
                            (&mut *scene).mark_component_changed_for_query(
                                entity,
                                <$ty as WriteFetch<'a>>::type_id(),
                            );
                        }
                    }
                )+
            }

            unsafe fn fetch(
                entity: Entity,
                columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
                row: usize,
            ) -> Option<Self> {
                unsafe {
                    Some((entity, $(<$ty as WriteFetch<'a>>::get(columns[$index], row)?,)+))
                }
            }
        }

        impl<'a, $($ty),+> ReadQueryData<'a> for (Entity, $($ty,)+)
        where
            $($ty: ReadFetch<'a> + WriteFetch<'a, Item = $ty> + 'a),+
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

    fn build(scene: &'a Scene) -> Query<'a, Data, Filters> {
        Query::from_state(scene, typed_filters::<Filters>(scene))
    }
}
