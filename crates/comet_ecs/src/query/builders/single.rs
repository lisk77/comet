use super::*;

impl<'a, P: ReadFetch<'a> + 'a, Filters> QueryBuilder<'a, P, Filters> {
    fn new(scene: &'a Scene) -> Self {
        Self::from_state(scene, QueryFilterState::empty())
    }

    fn from_state(scene: &'a Scene, state: QueryFilterState) -> Self {
        Self {
            scene,
            state,
            _marker: PhantomData,
        }
    }

    impl_query_state_methods_scene_ref!();

    pub fn filter<F>(self, f: F) -> QueryBuilderFiltered<'a, P, Filters, F>
    where
        F: Fn(&P::Component) -> bool + 'a,
    {
        QueryBuilderFiltered {
            scene: self.scene,
            state: self.state,
            filter: f,
            _marker: PhantomData,
        }
    }

    pub fn iter(self) -> QueryIter<'a, P> {
        assert!(P::required(), "standalone optional query fetches are not supported");
        QueryIter {
            accesses: build_single_read_accesses::<P>(self.scene, &self.state),
            idx: 0,
            added_since_filters: self.state.added_since_filters,
            changed_since_filters: self.state.changed_since_filters,
            _marker: PhantomData,
        }
    }

    pub fn iter_unfiltered(self) -> QueryIter<'a, P> {
        self.iter()
    }

    pub fn for_each(self, mut f: impl FnMut(P::Item)) {
        let mut iter = self.iter();
        while let Some(item) = iter.next() {
            f(item);
        }
    }
}

impl<'a, P: WriteFetch<'a> + 'a, Filters> Query<'a, P, Filters> {
    fn new(scene: &'a mut Scene) -> Self {
        Self::from_state(scene, QueryFilterState::empty())
    }

    fn from_state(scene: &'a mut Scene, state: QueryFilterState) -> Self {
        Self {
            scene,
            state,
            _marker: PhantomData,
        }
    }

    impl_query_state_methods_write_ptr!();

    pub fn filter<F>(self, f: F) -> QueryFiltered<'a, P, Filters, F>
    where
        F: Fn(&P::Component) -> bool + 'a,
    {
        QueryFiltered {
            scene: self.scene,
            state: self.state,
            filter: f,
            _marker: PhantomData,
        }
    }

    pub fn iter(self) -> QueryIterMut<'a, P> {
        assert!(P::required(), "standalone optional query fetches are not supported");
        QueryIterMut {
            accesses: build_single_write_accesses::<P>(self.scene, &self.state),
            idx: 0,
            added_since_filters: self.state.added_since_filters,
            changed_since_filters: self.state.changed_since_filters,
            _marker: PhantomData,
        }
    }

    pub fn iter_unfiltered(self) -> QueryIterMut<'a, P> {
        self.iter()
    }

    pub fn for_each(self, mut f: impl FnMut(P::Item)) {
        let mut iter = self.iter();
        while let Some(item) = iter.next() {
            f(item);
        }
    }
}

impl<'a, P: ReadFetch<'a> + 'a, Filters, F> QueryBuilderFiltered<'a, P, Filters, F>
where
    F: Fn(&P::Component) -> bool + 'a,
{
    impl_query_state_methods_scene_ref!();

    pub fn iter(self) -> QueryIterFiltered<'a, P, F> {
        assert!(P::required(), "standalone optional query fetches are not supported");
        QueryIterFiltered {
            inner: QueryIter {
                accesses: build_single_read_accesses::<P>(self.scene, &self.state),
                idx: 0,
                added_since_filters: self.state.added_since_filters,
                changed_since_filters: self.state.changed_since_filters,
                _marker: PhantomData,
            },
            filter: self.filter,
            _marker: PhantomData,
        }
    }

    pub fn for_each(self, mut f: impl FnMut(P::Item)) {
        let mut iter = self.iter();
        while let Some(item) = iter.next() {
            f(item);
        }
    }
}

impl<'a, P: WriteFetch<'a> + 'a, Filters, F> QueryFiltered<'a, P, Filters, F>
where
    F: Fn(&P::Component) -> bool + 'a,
{
    impl_query_state_methods_write_ptr!();

    pub fn iter(self) -> QueryIterMutFiltered<'a, P, F> {
        assert!(P::required(), "standalone optional query fetches are not supported");
        QueryIterMutFiltered {
            inner: QueryIterMut {
                accesses: build_single_write_accesses::<P>(self.scene, &self.state),
                idx: 0,
                added_since_filters: self.state.added_since_filters,
                changed_since_filters: self.state.changed_since_filters,
                _marker: PhantomData,
            },
            filter: self.filter,
            _marker: PhantomData,
        }
    }

    pub fn for_each(self, mut f: impl FnMut(P::Item)) {
        let mut iter = self.iter();
        while let Some(item) = iter.next() {
            f(item);
        }
    }
}

impl<'a, P: ReadFetch<'a> + 'a> QuerySpec<'a> for P {
    type Builder = QueryBuilder<'a, P, ()>;

    fn build(scene: &'a Scene) -> Self::Builder {
        QueryBuilder::new(scene)
    }
}

impl<'a, P: ReadFetch<'a> + 'a, Filters: QueryFilterSet> QuerySpec<'a>
    for crate::query::QueryParam<P, Filters>
{
    type Builder = QueryBuilder<'a, P, Filters>;

    fn build(scene: &'a Scene) -> Self::Builder {
        QueryBuilder::from_state(scene, typed_filters::<Filters>(scene))
    }
}

impl<'a, P: WriteFetch<'a> + 'a> QuerySpecMut<'a> for P {
    type Builder = Query<'a, P, ()>;

    fn build(scene: &'a mut Scene) -> Self::Builder {
        Query::new(scene)
    }
}

impl<'a, P: WriteFetch<'a> + 'a, Filters: QueryFilterSet> QuerySpecMut<'a>
    for crate::query::QueryParam<P, Filters>
{
    type Builder = Query<'a, P, Filters>;

    fn build(scene: &'a mut Scene) -> Self::Builder {
        Query::from_state(scene, typed_filters::<Filters>(scene))
    }
}
