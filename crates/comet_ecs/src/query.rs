use crate::{Component, Scene, Tag};
use comet_log::error;
use std::marker::PhantomData;

struct QueryAccess {
    col: *const comet_structs::Column,
    len: usize,
    row: usize,
}

struct QueryMutAccess {
    col: *mut comet_structs::Column,
    len: usize,
    row: usize,
}

struct QueryPairAccess {
    a_col: *const comet_structs::Column,
    b_col: *const comet_structs::Column,
    len: usize,
    row: usize,
}

struct QueryPairMutAccess {
    a_col: *mut comet_structs::Column,
    b_col: *mut comet_structs::Column,
    len: usize,
    row: usize,
}

pub struct Query<'a, C: Component> {
    accesses: Vec<QueryAccess>,
    idx: usize,
    _marker: PhantomData<&'a C>,
}

pub struct QueryMut<'a, C: Component> {
    accesses: Vec<QueryMutAccess>,
    idx: usize,
    _marker: PhantomData<&'a mut C>,
}

pub struct QueryPair<'a, A: Component, B: Component> {
    accesses: Vec<QueryPairAccess>,
    idx: usize,
    _marker: PhantomData<(&'a A, &'a B)>,
}

pub struct QueryPairMut<'a, A: Component, B: Component> {
    accesses: Vec<QueryPairMutAccess>,
    idx: usize,
    _marker: PhantomData<(&'a mut A, &'a mut B)>,
}

pub struct QueryBuilder<'a, C: Component> {
    scene: &'a Scene,
    tags: Vec<std::any::TypeId>,
    _marker: PhantomData<&'a C>,
}

pub struct QueryPairBuilder<'a, A: Component, B: Component> {
    scene: &'a Scene,
    tags: Vec<std::any::TypeId>,
    _marker: PhantomData<(&'a A, &'a B)>,
}

pub struct QueryMutBuilder<'a, C: Component> {
    scene: &'a mut Scene,
    tags: Vec<std::any::TypeId>,
    _marker: PhantomData<&'a mut C>,
}

pub struct QueryPairMutBuilder<'a, A: Component, B: Component> {
    scene: &'a mut Scene,
    tags: Vec<std::any::TypeId>,
    _marker: PhantomData<(&'a mut A, &'a mut B)>,
}

pub struct QueryBuilderFiltered<'a, C: Component, F>
where
    F: Fn(&C) -> bool + 'a,
{
    scene: &'a Scene,
    tags: Vec<std::any::TypeId>,
    filter: F,
    _marker: PhantomData<&'a C>,
}

pub struct QueryPairBuilderFiltered<'a, A: Component, B: Component, F>
where
    F: Fn(&A, &B) -> bool + 'a,
{
    scene: &'a Scene,
    tags: Vec<std::any::TypeId>,
    filter: F,
    _marker: PhantomData<(&'a A, &'a B)>,
}

pub struct QueryMutBuilderFiltered<'a, C: Component, F>
where
    F: Fn(&C) -> bool + 'a,
{
    scene: &'a mut Scene,
    tags: Vec<std::any::TypeId>,
    filter: F,
    _marker: PhantomData<&'a mut C>,
}

pub struct QueryPairMutBuilderFiltered<'a, A: Component, B: Component, F>
where
    F: Fn(&A, &B) -> bool + 'a,
{
    scene: &'a mut Scene,
    tags: Vec<std::any::TypeId>,
    filter: F,
    _marker: PhantomData<(&'a mut A, &'a mut B)>,
}

pub struct QueryFiltered<'a, C: Component, F>
where
    F: Fn(&C) -> bool + 'a,
{
    inner: Query<'a, C>,
    filter: F,
}

pub struct QueryPairFiltered<'a, A: Component, B: Component, F>
where
    F: Fn(&A, &B) -> bool + 'a,
{
    inner: QueryPair<'a, A, B>,
    filter: F,
}

pub struct QueryMutFiltered<'a, C: Component, F>
where
    F: Fn(&C) -> bool + 'a,
{
    inner: QueryMut<'a, C>,
    filter: F,
}

pub struct QueryPairMutFiltered<'a, A: Component, B: Component, F>
where
    F: Fn(&A, &B) -> bool + 'a,
{
    inner: QueryPairMut<'a, A, B>,
    filter: F,
}

pub trait QueryTuple<'a> {
    type Builder;
    fn build(scene: &'a Scene) -> Self::Builder;
}

pub trait QueryTupleMut<'a> {
    type Builder;
    fn build(scene: &'a mut Scene) -> Self::Builder;
}

impl<'a, C: Component> QueryTuple<'a> for (C,) {
    type Builder = QueryBuilder<'a, C>;

    fn build(scene: &'a Scene) -> Self::Builder {
        QueryBuilder {
            scene,
            tags: Vec::new(),
            _marker: PhantomData,
        }
    }
}

impl<'a, A: Component, B: Component> QueryTuple<'a> for (A, B) {
    type Builder = QueryPairBuilder<'a, A, B>;

    fn build(scene: &'a Scene) -> Self::Builder {
        QueryPairBuilder {
            scene,
            tags: Vec::new(),
            _marker: PhantomData,
        }
    }
}

impl<'a, C: Component> QueryTupleMut<'a> for (C,) {
    type Builder = QueryMutBuilder<'a, C>;

    fn build(scene: &'a mut Scene) -> Self::Builder {
        QueryMutBuilder {
            scene,
            tags: Vec::new(),
            _marker: PhantomData,
        }
    }
}

impl<'a, A: Component, B: Component> QueryTupleMut<'a> for (A, B) {
    type Builder = QueryPairMutBuilder<'a, A, B>;

    fn build(scene: &'a mut Scene) -> Self::Builder {
        QueryPairMutBuilder {
            scene,
            tags: Vec::new(),
            _marker: PhantomData,
        }
    }
}

impl<'a, C: Component> QueryTuple<'a> for C {
    type Builder = QueryBuilder<'a, C>;

    fn build(scene: &'a Scene) -> Self::Builder {
        QueryBuilder {
            scene,
            tags: Vec::new(),
            _marker: PhantomData,
        }
    }
}

impl<'a, C: Component> QueryTupleMut<'a> for C {
    type Builder = QueryMutBuilder<'a, C>;

    fn build(scene: &'a mut Scene) -> Self::Builder {
        QueryMutBuilder {
            scene,
            tags: Vec::new(),
            _marker: PhantomData,
        }
    }
}

impl Scene {
    pub fn query_iter<C: Component>(&self) -> Query<'_, C> {
        let mut accesses = Vec::new();
        for (arch_id, col_idx) in self.cached_single_plan(C::type_id(), &[]) {
            let arch = self.archetypes().get(arch_id);
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

    pub fn query_mut_iter<C: Component>(&mut self) -> QueryMut<'_, C> {
        let mut accesses = Vec::new();
        for (arch_id, col_idx) in self.cached_single_plan(C::type_id(), &[]) {
            let arch = self.archetypes_mut().get_mut(arch_id);
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

    pub fn query_pair_iter<A: Component, B: Component>(&self) -> QueryPair<'_, A, B> {
        if A::type_id() == B::type_id() {
            error!("query_pair called with identical component types");
            return QueryPair {
                accesses: Vec::new(),
                idx: 0,
                _marker: PhantomData,
            };
        }

        let mut accesses = Vec::new();
        for (arch_id, a_idx, b_idx) in self.cached_pair_plan(A::type_id(), B::type_id(), &[]) {
            let arch = self.archetypes().get(arch_id);
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

    pub fn query<'a, Q>(&'a self) -> <Q as QueryTuple<'a>>::Builder
    where
        Q: QueryTuple<'a>,
    {
        Q::build(self)
    }

    pub fn query_pair_mut_iter<A: Component, B: Component>(&mut self) -> QueryPairMut<'_, A, B> {
        if A::type_id() == B::type_id() {
            error!("query_pair_mut called with identical component types");
            return QueryPairMut {
                accesses: Vec::new(),
                idx: 0,
                _marker: PhantomData,
            };
        }

        let mut accesses = Vec::new();
        for (arch_id, a_idx, b_idx) in self.cached_pair_plan(A::type_id(), B::type_id(), &[]) {
            let arch = self.archetypes_mut().get_mut(arch_id);
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

    pub fn query_mut<'a, Q>(&'a mut self) -> <Q as QueryTupleMut<'a>>::Builder
    where
        Q: QueryTupleMut<'a>,
    {
        Q::build(self)
    }
}

impl<'a, C: Component> Iterator for Query<'a, C> {
    type Item = &'a C;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let access = self.accesses.get_mut(self.idx)?;
            if access.row >= access.len {
                self.idx += 1;
                continue;
            }
            let row = access.row;
            access.row += 1;
            unsafe {
                let col = &*access.col;
                return col.get::<C>(row);
            }
        }
    }
}

impl<'a, C: Component> Iterator for QueryMut<'a, C> {
    type Item = &'a mut C;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let access = self.accesses.get_mut(self.idx)?;
            if access.row >= access.len {
                self.idx += 1;
                continue;
            }
            let row = access.row;
            access.row += 1;
            unsafe {
                let col = &mut *access.col;
                return col.get_mut::<C>(row);
            }
        }
    }
}

impl<'a, A: Component, B: Component> Iterator for QueryPair<'a, A, B> {
    type Item = (&'a A, &'a B);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let access = self.accesses.get_mut(self.idx)?;
            if access.row >= access.len {
                self.idx += 1;
                continue;
            }
            let row = access.row;
            access.row += 1;
            unsafe {
                let a_col = &*access.a_col;
                let b_col = &*access.b_col;
                let a = a_col.get::<A>(row)?;
                let b = b_col.get::<B>(row)?;
                return Some((a, b));
            }
        }
    }
}

impl<'a, A: Component, B: Component> Iterator for QueryPairMut<'a, A, B> {
    type Item = (&'a mut A, &'a mut B);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let access = self.accesses.get_mut(self.idx)?;
            if access.row >= access.len {
                self.idx += 1;
                continue;
            }
            let row = access.row;
            access.row += 1;
            unsafe {
                let a_col = &mut *access.a_col;
                let b_col = &mut *access.b_col;
                let a = a_col.get_mut::<A>(row)?;
                let b = b_col.get_mut::<B>(row)?;
                return Some((a, b));
            }
        }
    }
}

impl<'a, C: Component> QueryBuilder<'a, C> {
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

impl<'a, A: Component, B: Component> QueryPairBuilder<'a, A, B> {
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
        let mut accesses = Vec::new();
        for (arch_id, a_idx, b_idx) in self
            .scene
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

impl<'a, A: Component, B: Component, F> QueryPairBuilderFiltered<'a, A, B, F>
where
    F: Fn(&A, &B) -> bool + 'a,
{
    pub fn with<T: Tag>(mut self) -> Self {
        self.tags.push(T::type_id());
        self
    }

    pub fn iter(self) -> QueryPairFiltered<'a, A, B, F> {
        let mut accesses = Vec::new();
        for (arch_id, a_idx, b_idx) in self
            .scene
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

impl<'a, C: Component, F> Iterator for QueryFiltered<'a, C, F>
where
    F: Fn(&C) -> bool + 'a,
{
    type Item = &'a C;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.inner.next()?;
            if (self.filter)(item) {
                return Some(item);
            }
        }
    }
}

impl<'a, A: Component, B: Component, F> Iterator for QueryPairFiltered<'a, A, B, F>
where
    F: Fn(&A, &B) -> bool + 'a,
{
    type Item = (&'a A, &'a B);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (a, b) = self.inner.next()?;
            if (self.filter)(a, b) {
                return Some((a, b));
            }
        }
    }
}

impl<'a, C: Component> QueryMutBuilder<'a, C> {
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

impl<'a, A: Component, B: Component> QueryPairMutBuilder<'a, A, B> {
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
        let mut accesses = Vec::new();
        for (arch_id, a_idx, b_idx) in self
            .scene
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

impl<'a, A: Component, B: Component, F> QueryPairMutBuilderFiltered<'a, A, B, F>
where
    F: Fn(&A, &B) -> bool + 'a,
{
    pub fn with<T: Tag>(mut self) -> Self {
        self.tags.push(T::type_id());
        self
    }

    pub fn iter(self) -> QueryPairMutFiltered<'a, A, B, F> {
        let mut accesses = Vec::new();
        for (arch_id, a_idx, b_idx) in self
            .scene
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

impl<'a, C: Component, F> Iterator for QueryMutFiltered<'a, C, F>
where
    F: Fn(&C) -> bool + 'a,
{
    type Item = &'a mut C;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.inner.next()?;
            if (self.filter)(&*item) {
                return Some(item);
            }
        }
    }
}

impl<'a, A: Component, B: Component, F> Iterator for QueryPairMutFiltered<'a, A, B, F>
where
    F: Fn(&A, &B) -> bool + 'a,
{
    type Item = (&'a mut A, &'a mut B);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (a, b) = self.inner.next()?;
            if (self.filter)(&*a, &*b) {
                return Some((a, b));
            }
        }
    }
}
