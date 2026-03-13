use super::tuple_types::*;
use super::*;
use crate::{ComponentTuple, Tick};

macro_rules! impl_base_tuple_query_arities {
    (
        $(
            (
                $($gen:ident),+ ;
                $tuple:ty,
                $builder:ty,
                $builder_mut:ty
            )
        ),+ $(,)?
    ) => {
        $(
            impl<'a, $($gen: ReadFetch<'a> + 'a),+> QuerySpec<'a> for $tuple {
                type Builder = $builder;

                fn build(scene: &'a Scene) -> Self::Builder {
                    <Self::Builder>::new(scene)
                }
            }

            impl<'a, $($gen: WriteFetch<'a> + 'a),+> QuerySpecMut<'a> for $tuple {
                type Builder = $builder_mut;

                fn build(scene: &'a mut Scene) -> Self::Builder {
                    <Self::Builder>::new(scene)
                }
            }
        )+
    };
}

impl<'a, P: ReadFetch<'a> + 'a, Filters> QueryBuilder<'a, P, Filters> {
    fn new(scene: &'a Scene) -> Self {
        Self {
            scene,
            with_components: Vec::new(),
            without_components: Vec::new(),
            with_any_components: Vec::new(),
            without_any_components: Vec::new(),
            added_filter: None,
            changed_filter: None,
            _marker: PhantomData,
        }
    }

    pub fn with<C: Component>(mut self) -> Self {
        self.with_components.push(C::type_id());
        self
    }

    pub fn without<C: Component>(mut self) -> Self {
        self.without_components.push(C::type_id());
        self
    }

    pub fn with_any<Cs: ComponentTuple>(mut self) -> Self {
        self.with_any_components.extend(Cs::type_ids());
        self
    }

    pub fn without_any<Cs: ComponentTuple>(mut self) -> Self {
        self.without_any_components.extend(Cs::type_ids());
        self
    }

    pub fn with_all<Cs: ComponentTuple>(mut self) -> Self {
        self.with_components.extend(Cs::type_ids());
        self
    }

    pub fn without_all<Cs: ComponentTuple>(mut self) -> Self {
        self.without_components.extend(Cs::type_ids());
        self
    }

    pub fn added(mut self) -> Self {
        self.added_filter = Some((P::type_id(), self.scene.query_default_tick()));
        self
    }

    pub fn changed(mut self) -> Self {
        self.changed_filter = Some((P::type_id(), self.scene.query_default_tick()));
        self
    }

    pub fn added_since(mut self, tick: Tick) -> Self {
        self.added_filter = Some((P::type_id(), tick));
        self
    }

    pub fn changed_since(mut self, tick: Tick) -> Self {
        self.changed_filter = Some((P::type_id(), tick));
        self
    }

    pub fn filter<F>(self, f: F) -> QueryBuilderFiltered<'a, P, Filters, F>
    where
        F: Fn(&P::Component) -> bool + 'a,
    {
        QueryBuilderFiltered {
            scene: self.scene,
            with_components: self.with_components,
            without_components: self.without_components,
            with_any_components: self.with_any_components,
            without_any_components: self.without_any_components,
            added_filter: self.added_filter,
            changed_filter: self.changed_filter,
            filter: f,
            _marker: PhantomData,
        }
    }

    pub fn iter(self) -> QueryIter<'a, P> {
        let mut accesses = Vec::new();
        for (arch_id, col_idx) in
            self.scene
                .cached_single_plan(P::type_id(), &self.with_components, &self.without_components, &self.with_any_components, &self.without_any_components)
        {
            let arch = self.scene.archetypes().get(arch_id);
            let col = &arch.columns()[col_idx] as *const _;
            let entities = arch.entities().as_ptr();
            let scene = self.scene as *const Scene;
            accesses.push(QueryAccess {
                entities,
                scene,
                col,
                len: arch.len(),
                row: 0,
            });
        }
        QueryIter {
            accesses,
            idx: 0,
            added_filter: self.added_filter,
            changed_filter: self.changed_filter,
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
        Self {
            scene,
            with_components: Vec::new(),
            without_components: Vec::new(),
            with_any_components: Vec::new(),
            without_any_components: Vec::new(),
            added_filter: None,
            changed_filter: None,
            _marker: PhantomData,
        }
    }

    pub fn with<C: Component>(mut self) -> Self {
        self.with_components.push(C::type_id());
        self
    }

    pub fn without<C: Component>(mut self) -> Self {
        self.without_components.push(C::type_id());
        self
    }

    pub fn with_any<Cs: ComponentTuple>(mut self) -> Self {
        self.with_any_components.extend(Cs::type_ids());
        self
    }

    pub fn without_any<Cs: ComponentTuple>(mut self) -> Self {
        self.without_any_components.extend(Cs::type_ids());
        self
    }

    pub fn with_all<Cs: ComponentTuple>(mut self) -> Self {
        self.with_components.extend(Cs::type_ids());
        self
    }

    pub fn without_all<Cs: ComponentTuple>(mut self) -> Self {
        self.without_components.extend(Cs::type_ids());
        self
    }

    pub fn added(mut self) -> Self {
        self.added_filter = Some((P::type_id(), unsafe { &*self.scene }.query_default_tick()));
        self
    }

    pub fn changed(mut self) -> Self {
        self.changed_filter = Some((P::type_id(), unsafe { &*self.scene }.query_default_tick()));
        self
    }

    pub fn added_since(mut self, tick: Tick) -> Self {
        self.added_filter = Some((P::type_id(), tick));
        self
    }

    pub fn changed_since(mut self, tick: Tick) -> Self {
        self.changed_filter = Some((P::type_id(), tick));
        self
    }

    pub fn filter<F>(self, f: F) -> QueryFiltered<'a, P, Filters, F>
    where
        F: Fn(&P::Component) -> bool + 'a,
    {
        QueryFiltered {
            scene: self.scene,
            with_components: self.with_components,
            without_components: self.without_components,
            with_any_components: self.with_any_components,
            without_any_components: self.without_any_components,
            added_filter: self.added_filter,
            changed_filter: self.changed_filter,
            filter: f,
            _marker: PhantomData,
        }
    }

    pub fn iter(self) -> QueryIterMut<'a, P> {
        let mut accesses = Vec::new();
        for (arch_id, col_idx) in
            unsafe { &*self.scene }
                .cached_single_plan(P::type_id(), &self.with_components, &self.without_components, &self.with_any_components, &self.without_any_components)
        {
            let arch = unsafe { &mut *self.scene }.archetypes_mut().get_mut(arch_id);
            let len = arch.len();
            let col = &mut arch.columns_mut()[col_idx] as *mut _;
            let entities = arch.entities().as_ptr();
            let scene = self.scene as *mut Scene;
            accesses.push(QueryMutAccess {
                entities,
                col,
                scene,
                len,
                row: 0,
            });
        }
        QueryIterMut {
            accesses,
            idx: 0,
            added_filter: self.added_filter,
            changed_filter: self.changed_filter,
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
    pub fn with<C: Component>(mut self) -> Self {
        self.with_components.push(C::type_id());
        self
    }

    pub fn without<C: Component>(mut self) -> Self {
        self.without_components.push(C::type_id());
        self
    }

    pub fn with_any<Cs: ComponentTuple>(mut self) -> Self {
        self.with_any_components.extend(Cs::type_ids());
        self
    }

    pub fn without_any<Cs: ComponentTuple>(mut self) -> Self {
        self.without_any_components.extend(Cs::type_ids());
        self
    }

    pub fn with_all<Cs: ComponentTuple>(mut self) -> Self {
        self.with_components.extend(Cs::type_ids());
        self
    }

    pub fn without_all<Cs: ComponentTuple>(mut self) -> Self {
        self.without_components.extend(Cs::type_ids());
        self
    }

    pub fn iter(self) -> QueryIterFiltered<'a, P, F> {
        let mut accesses = Vec::new();
        for (arch_id, col_idx) in
            self.scene
                .cached_single_plan(P::type_id(), &self.with_components, &self.without_components, &self.with_any_components, &self.without_any_components)
        {
            let arch = self.scene.archetypes().get(arch_id);
            let col = &arch.columns()[col_idx] as *const _;
            let entities = arch.entities().as_ptr();
            let scene = self.scene as *const Scene;
            accesses.push(QueryAccess {
                entities,
                scene,
                col,
                len: arch.len(),
                row: 0,
            });
        }
        QueryIterFiltered {
            inner: QueryIter {
                accesses,
                idx: 0,
                added_filter: self.added_filter,
                changed_filter: self.changed_filter,
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
    pub fn with<C: Component>(mut self) -> Self {
        self.with_components.push(C::type_id());
        self
    }

    pub fn without<C: Component>(mut self) -> Self {
        self.without_components.push(C::type_id());
        self
    }

    pub fn with_any<Cs: ComponentTuple>(mut self) -> Self {
        self.with_any_components.extend(Cs::type_ids());
        self
    }

    pub fn without_any<Cs: ComponentTuple>(mut self) -> Self {
        self.without_any_components.extend(Cs::type_ids());
        self
    }

    pub fn with_all<Cs: ComponentTuple>(mut self) -> Self {
        self.with_components.extend(Cs::type_ids());
        self
    }

    pub fn without_all<Cs: ComponentTuple>(mut self) -> Self {
        self.without_components.extend(Cs::type_ids());
        self
    }

    pub fn added(mut self) -> Self {
        self.added_filter = Some((P::type_id(), unsafe { &*self.scene }.query_default_tick()));
        self
    }

    pub fn changed(mut self) -> Self {
        self.changed_filter = Some((P::type_id(), unsafe { &*self.scene }.query_default_tick()));
        self
    }

    pub fn added_since(mut self, tick: Tick) -> Self {
        self.added_filter = Some((P::type_id(), tick));
        self
    }

    pub fn changed_since(mut self, tick: Tick) -> Self {
        self.changed_filter = Some((P::type_id(), tick));
        self
    }

    pub fn iter(self) -> QueryIterMutFiltered<'a, P, F> {
        let mut accesses = Vec::new();
        for (arch_id, col_idx) in
            unsafe { &*self.scene }
                .cached_single_plan(P::type_id(), &self.with_components, &self.without_components, &self.with_any_components, &self.without_any_components)
        {
            let arch = unsafe { &mut *self.scene }.archetypes_mut().get_mut(arch_id);
            let len = arch.len();
            let col = &mut arch.columns_mut()[col_idx] as *mut _;
            let entities = arch.entities().as_ptr();
            let scene = self.scene as *mut Scene;
            accesses.push(QueryMutAccess {
                entities,
                col,
                scene,
                len,
                row: 0,
            });
        }
        QueryIterMutFiltered {
            inner: QueryIterMut {
                accesses,
                idx: 0,
                added_filter: self.added_filter,
                changed_filter: self.changed_filter,
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

macro_rules! impl_tuple_builders_arity {
    (
        $builder:ident,
        $iter:ident,
        $access:ident,
        $builder_mut:ident,
        $iter_mut:ident,
        $access_mut:ident,
        $first_ty:ident,
        $first_col:ident,
        $($ty:ident, $idx:ident, $col:ident),+
    ) => {
        impl<'a, $first_ty: ReadFetch<'a> + 'a, $($ty: ReadFetch<'a> + 'a),+, Filters: QueryFilterSet> QuerySpec<'a> for crate::query::QueryParam<($first_ty, $($ty,)+), Filters> {
            type Builder = $builder<'a, $first_ty, $($ty),+, Filters>;

            fn build(scene: &'a Scene) -> Self::Builder {
                let filter_state = typed_filters::<Filters>(scene);
                $builder {
                    scene,
                    with_components: filter_state.with_components,
                    without_components: filter_state.without_components,
                    with_any_components: filter_state.with_any_components,
                    without_any_components: filter_state.without_any_components,
                    added_filter: filter_state.added_filter,
                    changed_filter: filter_state.changed_filter,
                    _marker: PhantomData,
                }
            }
        }

        impl<'a, $first_ty: ReadFetch<'a> + 'a, $($ty: ReadFetch<'a> + 'a),+> QuerySpec<'a> for ($first_ty, $($ty,)+) {
            type Builder = $builder<'a, $first_ty, $($ty),+, ()>;

            fn build(scene: &'a Scene) -> Self::Builder {
                $builder {
                    scene,
                    with_components: Vec::new(),
                    without_components: Vec::new(),
                    with_any_components: Vec::new(),
                    without_any_components: Vec::new(),
                    added_filter: None,
                    changed_filter: None,
                    _marker: PhantomData,
                }
            }
        }

        impl<'a, $first_ty: WriteFetch<'a> + 'a, $($ty: WriteFetch<'a> + 'a),+, Filters: QueryFilterSet> QuerySpecMut<'a> for crate::query::QueryParam<($first_ty, $($ty,)+), Filters> {
            type Builder = $builder_mut<'a, $first_ty, $($ty),+, Filters>;

            fn build(scene: &'a mut Scene) -> Self::Builder {
                let filter_state = typed_filters::<Filters>(unsafe { &*scene });
                $builder_mut {
                    scene,
                    with_components: filter_state.with_components,
                    without_components: filter_state.without_components,
                    with_any_components: filter_state.with_any_components,
                    without_any_components: filter_state.without_any_components,
                    added_filter: filter_state.added_filter,
                    changed_filter: filter_state.changed_filter,
                    _marker: PhantomData,
                }
            }
        }

        impl<'a, $first_ty: WriteFetch<'a> + 'a, $($ty: WriteFetch<'a> + 'a),+> QuerySpecMut<'a> for ($first_ty, $($ty,)+) {
            type Builder = $builder_mut<'a, $first_ty, $($ty),+, ()>;

            fn build(scene: &'a mut Scene) -> Self::Builder {
                $builder_mut {
                    scene,
                    with_components: Vec::new(),
                    without_components: Vec::new(),
                    with_any_components: Vec::new(),
                    without_any_components: Vec::new(),
                    added_filter: None,
                    changed_filter: None,
                    _marker: PhantomData,
                }
            }
        }

        impl<'a, $first_ty: ReadFetch<'a> + 'a, $($ty: ReadFetch<'a> + 'a),+, Filters> $builder<'a, $first_ty, $($ty),+, Filters> {
            pub fn with<Co: Component>(mut self) -> Self {
                self.with_components.push(Co::type_id());
                self
            }

            pub fn without<Co: Component>(mut self) -> Self {
                self.without_components.push(Co::type_id());
                self
            }

            pub fn with_any<Cs: ComponentTuple>(mut self) -> Self {
                self.with_any_components.extend(Cs::type_ids());
                self
            }

            pub fn without_any<Cs: ComponentTuple>(mut self) -> Self {
                self.without_any_components.extend(Cs::type_ids());
                self
            }

            pub fn with_all<Cs: ComponentTuple>(mut self) -> Self {
                self.with_components.extend(Cs::type_ids());
                self
            }

            pub fn without_all<Cs: ComponentTuple>(mut self) -> Self {
                self.without_components.extend(Cs::type_ids());
                self
            }

            pub fn iter(self) -> $iter<'a, $first_ty $(, $ty)*> {
                let mut accesses = Vec::new();
                let required = [$first_ty::type_id(), $($ty::type_id()),+];
                assert!(
                    !has_duplicate_type_ids(&required),
                    "query called with duplicate component types"
                );

                for (arch_id, first_idx) in self
                    .scene
                    .cached_single_plan($first_ty::type_id(), &self.with_components, &self.without_components, &self.with_any_components, &self.without_any_components)
                {
                    let arch = self.scene.archetypes().get(arch_id);
                    $(let $idx = match arch.column_index($ty::type_id()) {
                        Some(idx) => idx,
                        None => continue,
                    };)+
                    let cols = arch.columns();
                    let $first_col = &cols[first_idx] as *const _;
                    $(let $col = &cols[$idx] as *const _;)+
                    let entities = arch.entities().as_ptr();
                    let scene = self.scene as *const Scene;
                    accesses.push($access {
                        entities,
                        scene,
                        $first_col,
                        $($col,)+
                        len: arch.len(),
                        row: 0,
                    });
                }

                $iter {
                    accesses,
                    idx: 0,
                    added_filter: self.added_filter,
                    changed_filter: self.changed_filter,
                    _marker: PhantomData,
                }
            }

            pub fn for_each(self, mut f: impl FnMut(($first_ty::Item, $($ty::Item),+))) {
                let mut iter = self.iter();
                while let Some(item) = iter.next() {
                    f(item);
                }
            }
        }

        impl<'a, $first_ty: WriteFetch<'a> + 'a, $($ty: WriteFetch<'a> + 'a),+, Filters> $builder_mut<'a, $first_ty, $($ty),+, Filters> {
            pub fn with<Co: Component>(mut self) -> Self {
                self.with_components.push(Co::type_id());
                self
            }

            pub fn without<Co: Component>(mut self) -> Self {
                self.without_components.push(Co::type_id());
                self
            }

            pub fn with_any<Cs: ComponentTuple>(mut self) -> Self {
                self.with_any_components.extend(Cs::type_ids());
                self
            }

            pub fn without_any<Cs: ComponentTuple>(mut self) -> Self {
                self.without_any_components.extend(Cs::type_ids());
                self
            }

            pub fn with_all<Cs: ComponentTuple>(mut self) -> Self {
                self.with_components.extend(Cs::type_ids());
                self
            }

            pub fn without_all<Cs: ComponentTuple>(mut self) -> Self {
                self.without_components.extend(Cs::type_ids());
                self
            }

            pub fn iter(self) -> $iter_mut<'a, $first_ty, $($ty),+> {
                let mut accesses = Vec::new();
                let required = [$first_ty::type_id(), $($ty::type_id()),+];
                assert!(
                    !has_duplicate_type_ids(&required),
                    "query called with duplicate component types"
                );

                for (arch_id, first_idx) in self
                    .scene
                    .cached_single_plan($first_ty::type_id(), &self.with_components, &self.without_components, &self.with_any_components, &self.without_any_components)
                {
                    let arch = self.scene.archetypes_mut().get_mut(arch_id);
                    $(let $idx = match arch.column_index($ty::type_id()) {
                        Some(idx) => idx,
                        None => continue,
                    };)+
                    let len = arch.len();
                    let cols = arch.columns_mut();
                    let $first_col = &mut cols[first_idx] as *mut _;
                    $(let $col = &mut cols[$idx] as *mut _;)+
                    let entities = arch.entities().as_ptr();
                    let scene = self.scene as *mut Scene;
                    accesses.push($access_mut {
                        entities,
                        $first_col,
                        $($col,)+
                        scene,
                        len,
                        row: 0,
                    });
                }

                $iter_mut {
                    accesses,
                    idx: 0,
                    added_filter: self.added_filter,
                    changed_filter: self.changed_filter,
                    _marker: PhantomData,
                }
            }

            pub fn for_each(self, mut f: impl FnMut(($first_ty::Item, $($ty::Item),+))) {
                let mut iter = self.iter();
                while let Some(item) = iter.next() {
                    f(item);
                }
            }
        }
    };
}

macro_rules! impl_entity_tuple_builders_arity {
    (
        $builder:ident,
        $iter:ident,
        $access:ident,
        $builder_mut:ident,
        $iter_mut:ident,
        $access_mut:ident,
        $first_ty:ident,
        $first_col:ident
        $(,
            $ty:ident,
            $idx:ident,
            $col:ident
        )*
    ) => {
        impl<'a, $first_ty: ReadFetch<'a> + 'a $(, $ty: ReadFetch<'a> + 'a)*, Filters: QueryFilterSet> QuerySpec<'a> for crate::query::QueryParam<(Entity, $first_ty $(, $ty)*), Filters> {
            type Builder = $builder<'a, $first_ty $(, $ty)*, Filters>;

            fn build(scene: &'a Scene) -> Self::Builder {
                let filter_state = typed_filters::<Filters>(scene);
                $builder {
                    scene,
                    with_components: filter_state.with_components,
                    without_components: filter_state.without_components,
                    with_any_components: filter_state.with_any_components,
                    without_any_components: filter_state.without_any_components,
                    added_filter: filter_state.added_filter,
                    changed_filter: filter_state.changed_filter,
                    _marker: PhantomData,
                }
            }
        }

        impl<'a, $first_ty: ReadFetch<'a> + 'a $(, $ty: ReadFetch<'a> + 'a)*> QuerySpec<'a> for (Entity, $first_ty $(, $ty)*) {
            type Builder = $builder<'a, $first_ty $(, $ty)*, ()>;

            fn build(scene: &'a Scene) -> Self::Builder {
                $builder {
                    scene,
                    with_components: Vec::new(),
                    without_components: Vec::new(),
                    with_any_components: Vec::new(),
                    without_any_components: Vec::new(),
                    added_filter: None,
                    changed_filter: None,
                    _marker: PhantomData,
                }
            }
        }

        impl<'a, $first_ty: WriteFetch<'a> + 'a $(, $ty: WriteFetch<'a> + 'a)*, Filters: QueryFilterSet> QuerySpecMut<'a> for crate::query::QueryParam<(Entity, $first_ty $(, $ty)*), Filters> {
            type Builder = $builder_mut<'a, $first_ty $(, $ty)*, Filters>;

            fn build(scene: &'a mut Scene) -> Self::Builder {
                let filter_state = typed_filters::<Filters>(unsafe { &*scene });
                $builder_mut {
                    scene,
                    with_components: filter_state.with_components,
                    without_components: filter_state.without_components,
                    with_any_components: filter_state.with_any_components,
                    without_any_components: filter_state.without_any_components,
                    added_filter: filter_state.added_filter,
                    changed_filter: filter_state.changed_filter,
                    _marker: PhantomData,
                }
            }
        }

        impl<'a, $first_ty: WriteFetch<'a> + 'a $(, $ty: WriteFetch<'a> + 'a)*> QuerySpecMut<'a> for (Entity, $first_ty $(, $ty)*) {
            type Builder = $builder_mut<'a, $first_ty $(, $ty)*, ()>;

            fn build(scene: &'a mut Scene) -> Self::Builder {
                $builder_mut {
                    scene,
                    with_components: Vec::new(),
                    without_components: Vec::new(),
                    with_any_components: Vec::new(),
                    without_any_components: Vec::new(),
                    added_filter: None,
                    changed_filter: None,
                    _marker: PhantomData,
                }
            }
        }

        impl<'a, $first_ty: ReadFetch<'a> + 'a $(, $ty: ReadFetch<'a> + 'a)*, Filters> $builder<'a, $first_ty $(, $ty)*, Filters> {
            pub fn with<Co: Component>(mut self) -> Self {
                self.with_components.push(Co::type_id());
                self
            }

            pub fn without<Co: Component>(mut self) -> Self {
                self.without_components.push(Co::type_id());
                self
            }

            pub fn with_any<Cs: ComponentTuple>(mut self) -> Self {
                self.with_any_components.extend(Cs::type_ids());
                self
            }

            pub fn without_any<Cs: ComponentTuple>(mut self) -> Self {
                self.without_any_components.extend(Cs::type_ids());
                self
            }

            pub fn with_all<Cs: ComponentTuple>(mut self) -> Self {
                self.with_components.extend(Cs::type_ids());
                self
            }

            pub fn without_all<Cs: ComponentTuple>(mut self) -> Self {
                self.without_components.extend(Cs::type_ids());
                self
            }

            pub fn iter(self) -> $iter<'a, $first_ty $(, $ty)*> {
                let mut accesses = Vec::new();
                let required = [$first_ty::type_id() $(, $ty::type_id())*];
                assert!(
                    !has_duplicate_type_ids(&required),
                    "query called with duplicate component types"
                );

                for (arch_id, first_idx) in self
                    .scene
                    .cached_single_plan($first_ty::type_id(), &self.with_components, &self.without_components, &self.with_any_components, &self.without_any_components)
                {
                    let arch = self.scene.archetypes().get(arch_id);
                    $(let $idx = match arch.column_index($ty::type_id()) {
                        Some(idx) => idx,
                        None => continue,
                    };)*
                    let cols = arch.columns();
                    let $first_col = &cols[first_idx] as *const _;
                    $(let $col = &cols[$idx] as *const _;)*
                    let entities = arch.entities().as_ptr();
                    let scene = self.scene as *const Scene;
                    accesses.push($access {
                        entities,
                        scene,
                        $first_col,
                        $($col,)*
                        len: arch.len(),
                        row: 0,
                    });
                }

                $iter {
                    accesses,
                    idx: 0,
                    added_filter: self.added_filter,
                    changed_filter: self.changed_filter,
                    _marker: PhantomData,
                }
            }

            pub fn for_each(self, mut f: impl FnMut((Entity, $first_ty::Item $(, $ty::Item)*))) {
                let mut iter = self.iter();
                while let Some(item) = iter.next() {
                    f(item);
                }
            }
        }

        impl<'a, $first_ty: WriteFetch<'a> + 'a $(, $ty: WriteFetch<'a> + 'a)*, Filters> $builder_mut<'a, $first_ty $(, $ty)*, Filters> {
            pub fn with<Co: Component>(mut self) -> Self {
                self.with_components.push(Co::type_id());
                self
            }

            pub fn without<Co: Component>(mut self) -> Self {
                self.without_components.push(Co::type_id());
                self
            }

            pub fn with_any<Cs: ComponentTuple>(mut self) -> Self {
                self.with_any_components.extend(Cs::type_ids());
                self
            }

            pub fn without_any<Cs: ComponentTuple>(mut self) -> Self {
                self.without_any_components.extend(Cs::type_ids());
                self
            }

            pub fn with_all<Cs: ComponentTuple>(mut self) -> Self {
                self.with_components.extend(Cs::type_ids());
                self
            }

            pub fn without_all<Cs: ComponentTuple>(mut self) -> Self {
                self.without_components.extend(Cs::type_ids());
                self
            }

            pub fn iter(self) -> $iter_mut<'a, $first_ty $(, $ty)*> {
                let mut accesses = Vec::new();
                let required = [$first_ty::type_id() $(, $ty::type_id())*];
                assert!(
                    !has_duplicate_type_ids(&required),
                    "query called with duplicate component types"
                );

                for (arch_id, first_idx) in self
                    .scene
                    .cached_single_plan($first_ty::type_id(), &self.with_components, &self.without_components, &self.with_any_components, &self.without_any_components)
                {
                    let arch = self.scene.archetypes_mut().get_mut(arch_id);
                    $(let $idx = match arch.column_index($ty::type_id()) {
                        Some(idx) => idx,
                        None => continue,
                    };)*
                    let len = arch.len();
                    let cols = arch.columns_mut();
                    let $first_col = &mut cols[first_idx] as *mut _;
                    $(let $col = &mut cols[$idx] as *mut _;)*
                    let entities = arch.entities().as_ptr();
                    let scene = self.scene as *mut Scene;
                    accesses.push($access_mut {
                        entities,
                        $first_col,
                        $($col,)*
                        scene,
                        len,
                        row: 0,
                    });
                }

                $iter_mut {
                    accesses,
                    idx: 0,
                    added_filter: self.added_filter,
                    changed_filter: self.changed_filter,
                    _marker: PhantomData,
                }
            }

            pub fn for_each(self, mut f: impl FnMut((Entity, $first_ty::Item $(, $ty::Item)*))) {
                let mut iter = self.iter();
                while let Some(item) = iter.next() {
                    f(item);
                }
            }
        }
    };
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
        let filter_state = typed_filters::<Filters>(scene);
        QueryBuilder {
            scene,
            with_components: filter_state.with_components,
            without_components: filter_state.without_components,
            with_any_components: filter_state.with_any_components,
            without_any_components: filter_state.without_any_components,
            added_filter: filter_state.added_filter,
            changed_filter: filter_state.changed_filter,
            _marker: PhantomData,
        }
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
        let filter_state = typed_filters::<Filters>(unsafe { &*scene });
        Query {
            scene,
            with_components: filter_state.with_components,
            without_components: filter_state.without_components,
            with_any_components: filter_state.with_any_components,
            without_any_components: filter_state.without_any_components,
            added_filter: filter_state.added_filter,
            changed_filter: filter_state.changed_filter,
            _marker: PhantomData,
        }
    }
}

for_each_tuple_arity!(impl_tuple_builders_arity);
for_each_entity_tuple_arity!(impl_entity_tuple_builders_arity);
