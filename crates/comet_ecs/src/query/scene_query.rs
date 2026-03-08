use super::*;

impl Scene {
    pub fn query_iter<C: Component>(&self) -> QueryIter<'_, C> {
        let mut accesses = Vec::new();
        for (arch_id, col_idx) in self.cached_single_plan(C::type_id(), &[], &[]) {
            let arch = self.archetypes().get(arch_id);
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

    pub fn query_mut_iter<C: Component>(&mut self) -> QueryIterMut<'_, C> {
        let mut accesses = Vec::new();
        for (arch_id, col_idx) in self.cached_single_plan(C::type_id(), &[], &[]) {
            let arch = self.archetypes_mut().get_mut(arch_id);
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

    pub fn query<'a, Q>(&'a self) -> <Q as QuerySpec<'a>>::Builder
    where
        Q: QuerySpec<'a>,
    {
        Q::build(self)
    }

    pub fn query_mut<'a, Q>(&'a mut self) -> <Q as QuerySpecMut<'a>>::Builder
    where
        Q: QuerySpecMut<'a>,
    {
        Q::build(self)
    }
}
