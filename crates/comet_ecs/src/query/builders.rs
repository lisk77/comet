use super::tuple_types::*;
use super::*;
use crate::ComponentTuple;

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

impl<'a, P: ReadFetch<'a> + 'a> QueryBuilder<'a, P> {
    fn new(scene: &'a Scene) -> Self {
        Self {
            scene,
            with_components: Vec::new(),
            without_components: Vec::new(),
            with_any_components: Vec::new(),
            without_any_components: Vec::new(),
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

    pub fn filter<F>(self, f: F) -> QueryBuilderFiltered<'a, P, F>
    where
        F: Fn(&P::Component) -> bool + 'a,
    {
        QueryBuilderFiltered {
            scene: self.scene,
            with_components: self.with_components,
            without_components: self.without_components,
            with_any_components: self.with_any_components,
            without_any_components: self.without_any_components,
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
            accesses.push(QueryAccess {
                col,
                len: arch.len(),
                row: 0,
            });
        }
        QueryIter {
            accesses,
            idx: 0,
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

impl<'a, P: WriteFetch<'a> + 'a> Query<'a, P> {
    fn new(scene: &'a mut Scene) -> Self {
        Self {
            scene,
            with_components: Vec::new(),
            without_components: Vec::new(),
            with_any_components: Vec::new(),
            without_any_components: Vec::new(),
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

    pub fn filter<F>(self, f: F) -> QueryMutBuilderFiltered<'a, P, F>
    where
        F: Fn(&P::Component) -> bool + 'a,
    {
        QueryMutBuilderFiltered {
            scene: self.scene,
            with_components: self.with_components,
            without_components: self.without_components,
            with_any_components: self.with_any_components,
            without_any_components: self.without_any_components,
            filter: f,
            _marker: PhantomData,
        }
    }

    pub fn iter(self) -> QueryIterMut<'a, P> {
        let mut accesses = Vec::new();
        for (arch_id, col_idx) in
            self.scene
                .cached_single_plan(P::type_id(), &self.with_components, &self.without_components, &self.with_any_components, &self.without_any_components)
        {
            let arch = self.scene.archetypes_mut().get_mut(arch_id);
            let len = arch.len();
            let col = &mut arch.columns_mut()[col_idx] as *mut _;
            accesses.push(QueryMutAccess { col, len, row: 0 });
        }
        QueryIterMut {
            accesses,
            idx: 0,
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

impl<'a, P: ReadFetch<'a> + 'a, F> QueryBuilderFiltered<'a, P, F>
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
            accesses.push(QueryAccess {
                col,
                len: arch.len(),
                row: 0,
            });
        }
        QueryIterFiltered {
            inner: QueryIter {
                accesses,
                idx: 0,
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

impl<'a, P: WriteFetch<'a> + 'a, F> QueryMutBuilderFiltered<'a, P, F>
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

    pub fn iter(self) -> QueryIterMutFiltered<'a, P, F> {
        let mut accesses = Vec::new();
        for (arch_id, col_idx) in
            self.scene
                .cached_single_plan(P::type_id(), &self.with_components, &self.without_components, &self.with_any_components, &self.without_any_components)
        {
            let arch = self.scene.archetypes_mut().get_mut(arch_id);
            let len = arch.len();
            let col = &mut arch.columns_mut()[col_idx] as *mut _;
            accesses.push(QueryMutAccess { col, len, row: 0 });
        }
        QueryIterMutFiltered {
            inner: QueryIterMut {
                accesses,
                idx: 0,
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
        impl<'a, $first_ty: ReadFetch<'a> + 'a, $($ty: ReadFetch<'a> + 'a),+> QuerySpec<'a> for ($first_ty, $($ty,)+) {
            type Builder = $builder<'a, $first_ty, $($ty),+>;

            fn build(scene: &'a Scene) -> Self::Builder {
                $builder {
                    scene,
                    with_components: Vec::new(),
                    without_components: Vec::new(),
                    with_any_components: Vec::new(),
                    without_any_components: Vec::new(),
                    _marker: PhantomData,
                }
            }
        }

        impl<'a, $first_ty: WriteFetch<'a> + 'a, $($ty: WriteFetch<'a> + 'a),+> QuerySpecMut<'a> for ($first_ty, $($ty,)+) {
            type Builder = $builder_mut<'a, $first_ty, $($ty),+>;

            fn build(scene: &'a mut Scene) -> Self::Builder {
                $builder_mut {
                    scene,
                    with_components: Vec::new(),
                    without_components: Vec::new(),
                    with_any_components: Vec::new(),
                    without_any_components: Vec::new(),
                    _marker: PhantomData,
                }
            }
        }

        impl<'a, $first_ty: ReadFetch<'a> + 'a, $($ty: ReadFetch<'a> + 'a),+> $builder<'a, $first_ty, $($ty),+> {
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
                    accesses.push($access {
                        $first_col,
                        $($col,)+
                        len: arch.len(),
                        row: 0,
                    });
                }

                $iter {
                    accesses,
                    idx: 0,
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

        impl<'a, $first_ty: WriteFetch<'a> + 'a, $($ty: WriteFetch<'a> + 'a),+> $builder_mut<'a, $first_ty, $($ty),+> {
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
                    accesses.push($access_mut {
                        $first_col,
                        $($col,)+
                        len,
                        row: 0,
                    });
                }

                $iter_mut {
                    accesses,
                    idx: 0,
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
        impl<'a, $first_ty: ReadFetch<'a> + 'a $(, $ty: ReadFetch<'a> + 'a)*> QuerySpec<'a> for (Entity, $first_ty $(, $ty)*) {
            type Builder = $builder<'a, $first_ty $(, $ty)*>;

            fn build(scene: &'a Scene) -> Self::Builder {
                $builder {
                    scene,
                    with_components: Vec::new(),
                    without_components: Vec::new(),
                    with_any_components: Vec::new(),
                    without_any_components: Vec::new(),
                    _marker: PhantomData,
                }
            }
        }

        impl<'a, $first_ty: WriteFetch<'a> + 'a $(, $ty: WriteFetch<'a> + 'a)*> QuerySpecMut<'a> for (Entity, $first_ty $(, $ty)*) {
            type Builder = $builder_mut<'a, $first_ty $(, $ty)*>;

            fn build(scene: &'a mut Scene) -> Self::Builder {
                $builder_mut {
                    scene,
                    with_components: Vec::new(),
                    without_components: Vec::new(),
                    with_any_components: Vec::new(),
                    without_any_components: Vec::new(),
                    _marker: PhantomData,
                }
            }
        }

        impl<'a, $first_ty: ReadFetch<'a> + 'a $(, $ty: ReadFetch<'a> + 'a)*> $builder<'a, $first_ty $(, $ty)*> {
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
                    accesses.push($access {
                        entities,
                        $first_col,
                        $($col,)*
                        len: arch.len(),
                        row: 0,
                    });
                }

                $iter {
                    accesses,
                    idx: 0,
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

        impl<'a, $first_ty: WriteFetch<'a> + 'a $(, $ty: WriteFetch<'a> + 'a)*> $builder_mut<'a, $first_ty $(, $ty)*> {
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
                    accesses.push($access_mut {
                        entities,
                        $first_col,
                        $($col,)*
                        len,
                        row: 0,
                    });
                }

                $iter_mut {
                    accesses,
                    idx: 0,
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
    type Builder = QueryBuilder<'a, P>;

    fn build(scene: &'a Scene) -> Self::Builder {
        QueryBuilder::new(scene)
    }
}

impl<'a, P: WriteFetch<'a> + 'a> QuerySpecMut<'a> for P {
    type Builder = Query<'a, P>;

    fn build(scene: &'a mut Scene) -> Self::Builder {
        Query::new(scene)
    }
}

impl_base_tuple_query_arities!(
    (A; (A,), QueryBuilder<'a, A>, Query<'a, A>),
);

for_each_tuple_arity!(impl_tuple_builders_arity);
for_each_entity_tuple_arity!(impl_entity_tuple_builders_arity);
