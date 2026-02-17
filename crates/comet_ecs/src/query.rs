use crate::{Component, Scene};
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

impl Scene {
    pub fn query<C: Component>(&self) -> Query<'_, C> {
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

    pub fn query_mut<C: Component>(&mut self) -> QueryMut<'_, C> {
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

    pub fn query_pair<A: Component, B: Component>(&self) -> QueryPair<'_, A, B> {
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
            if let (Some(a_idx), Some(b_idx)) =
                (arch.column_index(A::type_id()), arch.column_index(B::type_id()))
            {
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

    pub fn query_pair_mut<A: Component, B: Component>(&mut self) -> QueryPairMut<'_, A, B> {
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
            if let (Some(a_idx), Some(b_idx)) =
                (arch.column_index(A::type_id()), arch.column_index(B::type_id()))
            {
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
