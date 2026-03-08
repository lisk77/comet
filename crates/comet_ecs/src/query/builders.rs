use super::tuple_types::*;
use super::*;

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
            impl<'a, $($gen: Component),+> QueryTuple<'a> for $tuple {
                type Builder = $builder;

                fn build(scene: &'a Scene) -> Self::Builder {
                    <Self::Builder>::new(scene)
                }
            }

            impl<'a, $($gen: Component),+> QueryTupleMut<'a> for $tuple {
                type Builder = $builder_mut;

                fn build(scene: &'a mut Scene) -> Self::Builder {
                    <Self::Builder>::new(scene)
                }
            }
        )+
    };
}

impl<'a, C: Component> QueryBuilder<'a, C> {
    fn new(scene: &'a Scene) -> Self {
        Self {
            scene,
            tags: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn with<T: Tag>(mut self) -> Self {
        self.tags.push(T::type_id());
        self
    }

    pub fn filter<F>(self, f: F) -> QueryBuilderFiltered<'a, C, F>
    where
        F: Fn(&C) -> bool + 'a,
    {
        QueryBuilderFiltered {
            scene: self.scene,
            tags: self.tags,
            filter: f,
            _marker: PhantomData,
        }
    }

    pub fn iter(self) -> Query<'a, C> {
        let mut accesses = Vec::new();
        for (arch_id, col_idx) in self.scene.cached_single_plan(C::type_id(), &self.tags) {
            let arch = self.scene.archetypes().get(arch_id);
            let col = &arch.columns()[col_idx] as *const _;
            accesses.push(QueryAccess {
                col,
                len: arch.len(),
                row: 0,
            });
        }
        Query {
            accesses,
            idx: 0,
            _marker: PhantomData,
        }
    }

    pub fn iter_unfiltered(self) -> Query<'a, C> {
        self.iter()
    }

    pub fn for_each(self, mut f: impl FnMut(&C)) {
        let mut iter = self.iter();
        while let Some(item) = iter.next() {
            f(item);
        }
    }
}

impl<'a, C: Component> QueryMutBuilder<'a, C> {
    fn new(scene: &'a mut Scene) -> Self {
        Self {
            scene,
            tags: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn with<T: Tag>(mut self) -> Self {
        self.tags.push(T::type_id());
        self
    }

    pub fn filter<F>(self, f: F) -> QueryMutBuilderFiltered<'a, C, F>
    where
        F: Fn(&C) -> bool + 'a,
    {
        QueryMutBuilderFiltered {
            scene: self.scene,
            tags: self.tags,
            filter: f,
            _marker: PhantomData,
        }
    }

    pub fn iter(self) -> QueryMut<'a, C> {
        let mut accesses = Vec::new();
        for (arch_id, col_idx) in self.scene.cached_single_plan(C::type_id(), &self.tags) {
            let arch = self.scene.archetypes_mut().get_mut(arch_id);
            let len = arch.len();
            let col = &mut arch.columns_mut()[col_idx] as *mut _;
            accesses.push(QueryMutAccess { col, len, row: 0 });
        }
        QueryMut {
            accesses,
            idx: 0,
            _marker: PhantomData,
        }
    }

    pub fn iter_unfiltered(self) -> QueryMut<'a, C> {
        self.iter()
    }

    pub fn for_each(self, mut f: impl FnMut(&mut C)) {
        let mut iter = self.iter();
        while let Some(item) = iter.next() {
            f(item);
        }
    }
}

impl<'a, A: Component, B: Component> QueryPairBuilder<'a, A, B> {
    fn new(scene: &'a Scene) -> Self {
        Self {
            scene,
            tags: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn with<T: Tag>(mut self) -> Self {
        self.tags.push(T::type_id());
        self
    }

    pub fn filter<F>(self, f: F) -> QueryPairBuilderFiltered<'a, A, B, F>
    where
        F: Fn(&A, &B) -> bool + 'a,
    {
        QueryPairBuilderFiltered {
            scene: self.scene,
            tags: self.tags,
            filter: f,
            _marker: PhantomData,
        }
    }

    pub fn iter(self) -> QueryPair<'a, A, B> {
        if A::type_id() == B::type_id() {
            error!("query_pair called with identical component types");
            return QueryPair {
                accesses: Vec::new(),
                idx: 0,
                _marker: PhantomData,
            };
        }

        let mut accesses = Vec::new();
        for (arch_id, a_idx, b_idx) in
            self.scene
                .cached_pair_plan(A::type_id(), B::type_id(), &self.tags)
        {
            let arch = self.scene.archetypes().get(arch_id);
            let cols = arch.columns();
            let a_col = &cols[a_idx] as *const _;
            let b_col = &cols[b_idx] as *const _;
            accesses.push(QueryPairAccess {
                a_col,
                b_col,
                len: arch.len(),
                row: 0,
            });
        }
        QueryPair {
            accesses,
            idx: 0,
            _marker: PhantomData,
        }
    }

    pub fn iter_unfiltered(self) -> QueryPair<'a, A, B> {
        self.iter()
    }

    pub fn for_each(self, mut f: impl FnMut(&A, &B)) {
        let mut iter = self.iter();
        while let Some((a, b)) = iter.next() {
            f(a, b);
        }
    }
}

impl<'a, A: Component, B: Component> QueryPairMutBuilder<'a, A, B> {
    fn new(scene: &'a mut Scene) -> Self {
        Self {
            scene,
            tags: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn with<T: Tag>(mut self) -> Self {
        self.tags.push(T::type_id());
        self
    }

    pub fn filter<F>(self, f: F) -> QueryPairMutBuilderFiltered<'a, A, B, F>
    where
        F: Fn(&A, &B) -> bool + 'a,
    {
        QueryPairMutBuilderFiltered {
            scene: self.scene,
            tags: self.tags,
            filter: f,
            _marker: PhantomData,
        }
    }

    pub fn iter(self) -> QueryPairMut<'a, A, B> {
        if A::type_id() == B::type_id() {
            error!("query_pair_mut called with identical component types");
            return QueryPairMut {
                accesses: Vec::new(),
                idx: 0,
                _marker: PhantomData,
            };
        }

        let mut accesses = Vec::new();
        for (arch_id, a_idx, b_idx) in
            self.scene
                .cached_pair_plan(A::type_id(), B::type_id(), &self.tags)
        {
            let arch = self.scene.archetypes_mut().get_mut(arch_id);
            let len = arch.len();
            let cols = arch.columns_mut();
            let a_col = &mut cols[a_idx] as *mut _;
            let b_col = &mut cols[b_idx] as *mut _;
            accesses.push(QueryPairMutAccess {
                a_col,
                b_col,
                len,
                row: 0,
            });
        }
        QueryPairMut {
            accesses,
            idx: 0,
            _marker: PhantomData,
        }
    }

    pub fn iter_unfiltered(self) -> QueryPairMut<'a, A, B> {
        self.iter()
    }

    pub fn for_each(self, mut f: impl FnMut(&mut A, &mut B)) {
        let mut iter = self.iter();
        while let Some((a, b)) = iter.next() {
            f(a, b);
        }
    }
}

impl<'a, C: Component, F> QueryBuilderFiltered<'a, C, F>
where
    F: Fn(&C) -> bool + 'a,
{
    pub fn with<T: Tag>(mut self) -> Self {
        self.tags.push(T::type_id());
        self
    }

    pub fn iter(self) -> QueryFiltered<'a, C, F> {
        let mut accesses = Vec::new();
        for (arch_id, col_idx) in self.scene.cached_single_plan(C::type_id(), &self.tags) {
            let arch = self.scene.archetypes().get(arch_id);
            let col = &arch.columns()[col_idx] as *const _;
            accesses.push(QueryAccess {
                col,
                len: arch.len(),
                row: 0,
            });
        }
        QueryFiltered {
            inner: Query {
                accesses,
                idx: 0,
                _marker: PhantomData,
            },
            filter: self.filter,
        }
    }

    pub fn for_each(self, mut f: impl FnMut(&C)) {
        let mut iter = self.iter();
        while let Some(item) = iter.next() {
            f(item);
        }
    }
}

impl<'a, A: Component, B: Component, F> QueryPairBuilderFiltered<'a, A, B, F>
where
    F: Fn(&A, &B) -> bool + 'a,
{
    pub fn with<T: Tag>(mut self) -> Self {
        self.tags.push(T::type_id());
        self
    }

    pub fn iter(self) -> QueryPairFiltered<'a, A, B, F> {
        if A::type_id() == B::type_id() {
            error!("query_pair called with identical component types");
            return QueryPairFiltered {
                inner: QueryPair {
                    accesses: Vec::new(),
                    idx: 0,
                    _marker: PhantomData,
                },
                filter: self.filter,
            };
        }

        let mut accesses = Vec::new();
        for (arch_id, a_idx, b_idx) in
            self.scene
                .cached_pair_plan(A::type_id(), B::type_id(), &self.tags)
        {
            let arch = self.scene.archetypes().get(arch_id);
            let cols = arch.columns();
            let a_col = &cols[a_idx] as *const _;
            let b_col = &cols[b_idx] as *const _;
            accesses.push(QueryPairAccess {
                a_col,
                b_col,
                len: arch.len(),
                row: 0,
            });
        }
        QueryPairFiltered {
            inner: QueryPair {
                accesses,
                idx: 0,
                _marker: PhantomData,
            },
            filter: self.filter,
        }
    }

    pub fn for_each(self, mut f: impl FnMut(&A, &B)) {
        let mut iter = self.iter();
        while let Some((a, b)) = iter.next() {
            f(a, b);
        }
    }
}

impl<'a, C: Component, F> QueryMutBuilderFiltered<'a, C, F>
where
    F: Fn(&C) -> bool + 'a,
{
    pub fn with<T: Tag>(mut self) -> Self {
        self.tags.push(T::type_id());
        self
    }

    pub fn iter(self) -> QueryMutFiltered<'a, C, F> {
        let mut accesses = Vec::new();
        for (arch_id, col_idx) in self.scene.cached_single_plan(C::type_id(), &self.tags) {
            let arch = self.scene.archetypes_mut().get_mut(arch_id);
            let len = arch.len();
            let col = &mut arch.columns_mut()[col_idx] as *mut _;
            accesses.push(QueryMutAccess { col, len, row: 0 });
        }
        QueryMutFiltered {
            inner: QueryMut {
                accesses,
                idx: 0,
                _marker: PhantomData,
            },
            filter: self.filter,
        }
    }

    pub fn for_each(self, mut f: impl FnMut(&mut C)) {
        let mut iter = self.iter();
        while let Some(item) = iter.next() {
            f(item);
        }
    }
}

impl<'a, A: Component, B: Component, F> QueryPairMutBuilderFiltered<'a, A, B, F>
where
    F: Fn(&A, &B) -> bool + 'a,
{
    pub fn with<T: Tag>(mut self) -> Self {
        self.tags.push(T::type_id());
        self
    }

    pub fn iter(self) -> QueryPairMutFiltered<'a, A, B, F> {
        if A::type_id() == B::type_id() {
            error!("query_pair_mut called with identical component types");
            return QueryPairMutFiltered {
                inner: QueryPairMut {
                    accesses: Vec::new(),
                    idx: 0,
                    _marker: PhantomData,
                },
                filter: self.filter,
            };
        }

        let mut accesses = Vec::new();
        for (arch_id, a_idx, b_idx) in
            self.scene
                .cached_pair_plan(A::type_id(), B::type_id(), &self.tags)
        {
            let arch = self.scene.archetypes_mut().get_mut(arch_id);
            let len = arch.len();
            let cols = arch.columns_mut();
            let a_col = &mut cols[a_idx] as *mut _;
            let b_col = &mut cols[b_idx] as *mut _;
            accesses.push(QueryPairMutAccess {
                a_col,
                b_col,
                len,
                row: 0,
            });
        }
        QueryPairMutFiltered {
            inner: QueryPairMut {
                accesses,
                idx: 0,
                _marker: PhantomData,
            },
            filter: self.filter,
        }
    }

    pub fn for_each(self, mut f: impl FnMut(&mut A, &mut B)) {
        let mut iter = self.iter();
        while let Some((a, b)) = iter.next() {
            f(a, b);
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
        impl<'a, $first_ty: Component, $($ty: Component),+> QueryTuple<'a> for ($first_ty, $($ty,)+) {
            type Builder = $builder<'a, $first_ty, $($ty),+>;

            fn build(scene: &'a Scene) -> Self::Builder {
                $builder {
                    scene,
                    tags: Vec::new(),
                    _marker: PhantomData,
                }
            }
        }

        impl<'a, $first_ty: Component, $($ty: Component),+> QueryTupleMut<'a> for ($first_ty, $($ty,)+) {
            type Builder = $builder_mut<'a, $first_ty, $($ty),+>;

            fn build(scene: &'a mut Scene) -> Self::Builder {
                $builder_mut {
                    scene,
                    tags: Vec::new(),
                    _marker: PhantomData,
                }
            }
        }

        impl<'a, $first_ty: Component, $($ty: Component),+> $builder<'a, $first_ty, $($ty),+> {
            pub fn with<T: Tag>(mut self) -> Self {
                self.tags.push(T::type_id());
                self
            }

            pub fn iter(self) -> $iter<'a, $first_ty, $($ty),+> {
                let mut accesses = Vec::new();
                let required = [$first_ty::type_id(), $($ty::type_id()),+];
                if has_duplicate_type_ids(&required) {
                    error!("query called with duplicate component types");
                    return $iter {
                        accesses,
                        idx: 0,
                        _marker: PhantomData,
                    };
                }

                for (arch_id, first_idx) in self
                    .scene
                    .cached_single_plan($first_ty::type_id(), &self.tags)
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

            pub fn for_each(self, mut f: impl FnMut((&$first_ty, $(&$ty),+))) {
                let mut iter = self.iter();
                while let Some(item) = iter.next() {
                    f(item);
                }
            }
        }

        impl<'a, $first_ty: Component, $($ty: Component),+> $builder_mut<'a, $first_ty, $($ty),+> {
            pub fn with<T: Tag>(mut self) -> Self {
                self.tags.push(T::type_id());
                self
            }

            pub fn iter(self) -> $iter_mut<'a, $first_ty, $($ty),+> {
                let mut accesses = Vec::new();
                let required = [$first_ty::type_id(), $($ty::type_id()),+];
                if has_duplicate_type_ids(&required) {
                    error!("query_mut called with duplicate component types");
                    return $iter_mut {
                        accesses,
                        idx: 0,
                        _marker: PhantomData,
                    };
                }

                for (arch_id, first_idx) in self
                    .scene
                    .cached_single_plan($first_ty::type_id(), &self.tags)
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

            pub fn for_each(self, mut f: impl FnMut((&mut $first_ty, $(&mut $ty),+))) {
                let mut iter = self.iter();
                while let Some(item) = iter.next() {
                    f(item);
                }
            }
        }
    };
}

impl<'a, C: Component> QueryTuple<'a> for C {
    type Builder = QueryBuilder<'a, C>;

    fn build(scene: &'a Scene) -> Self::Builder {
        QueryBuilder::new(scene)
    }
}

impl<'a, C: Component> QueryTupleMut<'a> for C {
    type Builder = QueryMutBuilder<'a, C>;

    fn build(scene: &'a mut Scene) -> Self::Builder {
        QueryMutBuilder::new(scene)
    }
}

impl_base_tuple_query_arities!(
    (A; (A,), QueryBuilder<'a, A>, QueryMutBuilder<'a, A>),
);

for_each_tuple_arity!(impl_tuple_builders_arity);
