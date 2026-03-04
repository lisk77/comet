use super::*;

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
