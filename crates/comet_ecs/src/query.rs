use crate::{Component, Scene};
use comet_log::error;
use std::marker::PhantomData;
use std::mem;

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
    filter: Option<Box<dyn Fn(&C) -> bool + 'a>>,
}

pub struct QueryPairBuilder<'a, A: Component, B: Component> {
    scene: &'a Scene,
    tags: Vec<std::any::TypeId>,
    filter: Option<Box<dyn Fn(&A, &B) -> bool + 'a>>,
}

pub struct QueryMutBuilder<'a, C: Component> {
    scene: &'a mut Scene,
    tags: Vec<std::any::TypeId>,
    filter: Option<Box<dyn Fn(&C) -> bool + 'a>>,
}

pub struct QueryPairMutBuilder<'a, A: Component, B: Component> {
    scene: &'a mut Scene,
    tags: Vec<std::any::TypeId>,
    filter: Option<Box<dyn Fn(&A, &B) -> bool + 'a>>,
}

pub struct QueryMutFiltered<'a, C: Component> {
    inner: QueryMut<'a, C>,
    filter: Option<Box<dyn Fn(&C) -> bool + 'a>>,
}

pub struct QueryPairMutFiltered<'a, A: Component, B: Component> {
    inner: QueryPairMut<'a, A, B>,
    filter: Option<Box<dyn Fn(&A, &B) -> bool + 'a>>,
}

pub struct QueryFiltered<'a, C: Component> {
    inner: Query<'a, C>,
    filter: Option<Box<dyn Fn(&C) -> bool + 'a>>,
}

pub struct QueryPairFiltered<'a, A: Component, B: Component> {
    inner: QueryPair<'a, A, B>,
    filter: Option<Box<dyn Fn(&A, &B) -> bool + 'a>>,
}

impl Scene {
    pub fn query_iter<C: Component>(&self) -> Query<'_, C> {
        let mut accesses = Vec::new();
        for arch in self.archetypes().iter() {
            if let Some(col_idx) = arch.column_index(C::type_id()) {
                let col = &arch.columns()[col_idx] as *const _;
                accesses.push(QueryAccess {
                    col,
                    len: arch.len(),
                    row: 0,
                });
            }
        }
        Query {
            accesses,
            idx: 0,
            _marker: PhantomData,
        }
    }

    pub fn query_mut_iter<C: Component>(&mut self) -> QueryMut<'_, C> {
        let mut accesses = Vec::new();
        for arch in self.archetypes_mut().iter_mut() {
            if let Some(col_idx) = arch.column_index(C::type_id()) {
                let col = &mut arch.columns_mut()[col_idx] as *mut _;
                accesses.push(QueryMutAccess {
                    col,
                    len: arch.len(),
                    row: 0,
                });
            }
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
        for arch in self.archetypes().iter() {
            if let (Some(a_idx), Some(b_idx)) = (
                arch.column_index(A::type_id()),
                arch.column_index(B::type_id()),
            ) {
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
        }
        QueryPair {
            accesses,
            idx: 0,
            _marker: PhantomData,
        }
    }

    pub fn query<C: Component>(&self) -> QueryBuilder<'_, C> {
        QueryBuilder {
            scene: self,
            tags: Vec::new(),
            filter: None,
        }
    }

    pub fn query_pair<A: Component, B: Component>(&self) -> QueryPairBuilder<'_, A, B> {
        QueryPairBuilder {
            scene: self,
            tags: Vec::new(),
            filter: None,
        }
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
        for arch in self.archetypes_mut().iter_mut() {
            if let (Some(a_idx), Some(b_idx)) = (
                arch.column_index(A::type_id()),
                arch.column_index(B::type_id()),
            ) {
                let cols = arch.columns_mut();
                let a_col = &mut cols[a_idx] as *mut _;
                let b_col = &mut cols[b_idx] as *mut _;
                accesses.push(QueryPairMutAccess {
                    a_col,
                    b_col,
                    len: arch.len(),
                    row: 0,
                });
            }
        }
        QueryPairMut {
            accesses,
            idx: 0,
            _marker: PhantomData,
        }
    }

    pub fn query_mut<C: Component>(&mut self) -> QueryMutBuilder<'_, C> {
        QueryMutBuilder {
            scene: self,
            tags: Vec::new(),
            filter: None,
        }
    }

    pub fn query_pair_mut<A: Component, B: Component>(&mut self) -> QueryPairMutBuilder<'_, A, B> {
        QueryPairMutBuilder {
            scene: self,
            tags: Vec::new(),
            filter: None,
        }
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
    pub fn with<Tag: Component>(mut self) -> Self {
        if mem::size_of::<Tag>() != 0 {
            panic!("with::<Tag>() requires a ZST tag component");
        }
        self.tags.push(Tag::type_id());
        self
    }

    pub fn filter<F>(mut self, f: F) -> Self
    where
        F: Fn(&C) -> bool + 'a,
    {
        self.filter = Some(Box::new(f));
        self
    }

    pub fn iter(self) -> QueryFiltered<'a, C> {
        let mut accesses = Vec::new();
        for arch in self.scene.archetypes().iter() {
            if let Some(col_idx) = arch.column_index(C::type_id()) {
                if self.tags.iter().all(|t| arch.column_index(*t).is_some()) {
                    let col = &arch.columns()[col_idx] as *const _;
                    accesses.push(QueryAccess {
                        col,
                        len: arch.len(),
                        row: 0,
                    });
                }
            }
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

    pub fn iter_unfiltered(self) -> Query<'a, C> {
        self.scene.query_iter::<C>()
    }

    pub fn for_each(self, mut f: impl FnMut(&C)) {
        let mut iter = self.iter();
        while let Some(item) = iter.next() {
            f(item);
        }
    }
}

impl<'a, A: Component, B: Component> QueryPairBuilder<'a, A, B> {
    pub fn with<Tag: Component>(mut self) -> Self {
        if mem::size_of::<Tag>() != 0 {
            panic!("with::<Tag>() requires a ZST tag component");
        }
        self.tags.push(Tag::type_id());
        self
    }

    pub fn filter<F>(mut self, f: F) -> Self
    where
        F: Fn(&A, &B) -> bool + 'a,
    {
        self.filter = Some(Box::new(f));
        self
    }

    pub fn iter(self) -> QueryPairFiltered<'a, A, B> {
        let mut accesses = Vec::new();
        for arch in self.scene.archetypes().iter() {
            if let (Some(a_idx), Some(b_idx)) = (
                arch.column_index(A::type_id()),
                arch.column_index(B::type_id()),
            ) {
                if self.tags.iter().all(|t| arch.column_index(*t).is_some()) {
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
            }
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

    pub fn iter_unfiltered(self) -> QueryPair<'a, A, B> {
        self.scene.query_pair_iter::<A, B>()
    }

    pub fn for_each(self, mut f: impl FnMut(&A, &B)) {
        let mut iter = self.iter();
        while let Some((a, b)) = iter.next() {
            f(a, b);
        }
    }
}

impl<'a, C: Component> Iterator for QueryFiltered<'a, C> {
    type Item = &'a C;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.inner.next()?;
            if let Some(filter) = &self.filter {
                if filter(item) {
                    return Some(item);
                }
            } else {
                return Some(item);
            }
        }
    }
}

impl<'a, A: Component, B: Component> Iterator for QueryPairFiltered<'a, A, B> {
    type Item = (&'a A, &'a B);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (a, b) = self.inner.next()?;
            if let Some(filter) = &self.filter {
                if filter(a, b) {
                    return Some((a, b));
                }
            } else {
                return Some((a, b));
            }
        }
    }
}

impl<'a, C: Component> QueryMutBuilder<'a, C> {
    pub fn with<Tag: Component>(mut self) -> Self {
        if mem::size_of::<Tag>() != 0 {
            panic!("with::<Tag>() requires a ZST tag component");
        }
        self.tags.push(Tag::type_id());
        self
    }

    pub fn filter<F>(mut self, f: F) -> Self
    where
        F: Fn(&C) -> bool + 'a,
    {
        self.filter = Some(Box::new(f));
        self
    }

    pub fn iter(self) -> QueryMutFiltered<'a, C> {
        let mut accesses = Vec::new();
        for arch in self.scene.archetypes_mut().iter_mut() {
            if let Some(col_idx) = arch.column_index(C::type_id()) {
                if self.tags.iter().all(|t| arch.column_index(*t).is_some()) {
                    let col = &mut arch.columns_mut()[col_idx] as *mut _;
                    accesses.push(QueryMutAccess {
                        col,
                        len: arch.len(),
                        row: 0,
                    });
                }
            }
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

    pub fn iter_unfiltered(self) -> QueryMut<'a, C> {
        self.scene.query_mut_iter::<C>()
    }

    pub fn for_each(mut self, mut f: impl FnMut(&mut C)) {
        let mut iter = self.iter();
        while let Some(item) = iter.next() {
            f(item);
        }
    }
}

impl<'a, A: Component, B: Component> QueryPairMutBuilder<'a, A, B> {
    pub fn with<Tag: Component>(mut self) -> Self {
        if mem::size_of::<Tag>() != 0 {
            panic!("with::<Tag>() requires a ZST tag component");
        }
        self.tags.push(Tag::type_id());
        self
    }

    pub fn filter<F>(mut self, f: F) -> Self
    where
        F: Fn(&A, &B) -> bool + 'a,
    {
        self.filter = Some(Box::new(f));
        self
    }

    pub fn iter(self) -> QueryPairMutFiltered<'a, A, B> {
        let mut accesses = Vec::new();
        for arch in self.scene.archetypes_mut().iter_mut() {
            if let (Some(a_idx), Some(b_idx)) = (
                arch.column_index(A::type_id()),
                arch.column_index(B::type_id()),
            ) {
                if self.tags.iter().all(|t| arch.column_index(*t).is_some()) {
                    let cols = arch.columns_mut();
                    let a_col = &mut cols[a_idx] as *mut _;
                    let b_col = &mut cols[b_idx] as *mut _;
                    accesses.push(QueryPairMutAccess {
                        a_col,
                        b_col,
                        len: arch.len(),
                        row: 0,
                    });
                }
            }
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

    pub fn iter_unfiltered(self) -> QueryPairMut<'a, A, B> {
        self.scene.query_pair_mut_iter::<A, B>()
    }

    pub fn for_each(mut self, mut f: impl FnMut(&mut A, &mut B)) {
        let mut iter = self.iter();
        while let Some((a, b)) = iter.next() {
            f(a, b);
        }
    }
}

impl<'a, C: Component> Iterator for QueryMutFiltered<'a, C> {
    type Item = &'a mut C;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.inner.next()?;
            if let Some(filter) = &self.filter {
                if filter(&*item) {
                    return Some(item);
                }
            } else {
                return Some(item);
            }
        }
    }
}

impl<'a, A: Component, B: Component> Iterator for QueryPairMutFiltered<'a, A, B> {
    type Item = (&'a mut A, &'a mut B);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (a, b) = self.inner.next()?;
            if let Some(filter) = &self.filter {
                if filter(&*a, &*b) {
                    return Some((a, b));
                }
            } else {
                return Some((a, b));
            }
        }
    }
}
