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

    impl_query_state_methods_scene_ref!(P);

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
        QueryIter {
            accesses: build_single_read_accesses::<P>(self.scene, &self.state),
            idx: 0,
            added_filter: self.state.added_filter,
            changed_filter: self.state.changed_filter,
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

    impl_query_state_methods_write_ptr!(P);

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
        QueryIterMut {
            accesses: build_single_write_accesses::<P>(self.scene, &self.state),
            idx: 0,
            added_filter: self.state.added_filter,
            changed_filter: self.state.changed_filter,
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
    impl_query_state_methods_scene_ref!(P);

    pub fn iter(self) -> QueryIterFiltered<'a, P, F> {
        QueryIterFiltered {
            inner: QueryIter {
                accesses: build_single_read_accesses::<P>(self.scene, &self.state),
                idx: 0,
                added_filter: self.state.added_filter,
                changed_filter: self.state.changed_filter,
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
    impl_query_state_methods_write_ptr!(P);

    pub fn iter(self) -> QueryIterMutFiltered<'a, P, F> {
        QueryIterMutFiltered {
            inner: QueryIterMut {
                accesses: build_single_write_accesses::<P>(self.scene, &self.state),
                idx: 0,
                added_filter: self.state.added_filter,
                changed_filter: self.state.changed_filter,
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
