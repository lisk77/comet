use super::*;

pub struct QueryIter<'a, P: ReadFetch<'a>> {
    pub(crate) accesses: Vec<QueryAccess>,
    pub(crate) idx: usize,
    pub(crate) added_filter: Option<(TypeId, Tick)>,
    pub(crate) changed_filter: Option<(TypeId, Tick)>,
    pub(crate) _marker: PhantomData<&'a P>,
}

pub struct QueryIterMut<'a, P: WriteFetch<'a>> {
    pub(crate) accesses: Vec<QueryMutAccess>,
    pub(crate) idx: usize,
    pub(crate) added_filter: Option<(TypeId, Tick)>,
    pub(crate) changed_filter: Option<(TypeId, Tick)>,
    pub(crate) _marker: PhantomData<&'a P>,
}

pub struct QueryBuilder<'a, P: ReadFetch<'a>, Filters = ()> {
    pub(crate) scene: &'a Scene,
    pub(crate) state: QueryFilterState,
    pub(crate) _marker: PhantomData<(P, Filters)>,
}

pub struct Query<'a, P, Filters = ()> {
    pub(crate) scene: *mut Scene,
    pub(crate) state: QueryFilterState,
    pub(crate) _marker: PhantomData<(&'a (), P, Filters)>,
}

pub struct QueryBuilderFiltered<'a, P: ReadFetch<'a>, Filters, F>
where
    F: Fn(&P::Component) -> bool + 'a,
{
    pub(crate) scene: &'a Scene,
    pub(crate) state: QueryFilterState,
    pub(crate) filter: F,
    pub(crate) _marker: PhantomData<(P, Filters)>,
}

pub struct QueryFiltered<'a, P: WriteFetch<'a>, Filters, F>
where
    F: Fn(&P::Component) -> bool + 'a,
{
    pub(crate) scene: *mut Scene,
    pub(crate) state: QueryFilterState,
    pub(crate) filter: F,
    pub(crate) _marker: PhantomData<(&'a (), P, Filters)>,
}

pub struct QueryIterFiltered<'a, P: ReadFetch<'a>, F>
where
    F: Fn(&P::Component) -> bool + 'a,
{
    pub(crate) inner: QueryIter<'a, P>,
    pub(crate) filter: F,
    pub(crate) _marker: PhantomData<&'a P>,
}

pub struct QueryIterMutFiltered<'a, P: WriteFetch<'a>, F>
where
    F: Fn(&P::Component) -> bool + 'a,
{
    pub(crate) inner: QueryIterMut<'a, P>,
    pub(crate) filter: F,
    pub(crate) _marker: PhantomData<&'a P>,
}
