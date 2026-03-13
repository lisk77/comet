use super::*;

impl Scene {
    pub(crate) fn query_iter<'a, P: ReadFetch<'a>>(&'a self) -> QueryIter<'a, P> {
        let mut accesses = Vec::new();
        for (arch_id, col_idx) in self.cached_single_plan(P::type_id(), &[], &[], &[], &[]) {
            let arch = self.archetypes().get(arch_id);
            let col = &arch.columns()[col_idx] as *const _;
            let entities = arch.entities().as_ptr();
            let scene = self as *const Scene;
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
            added_filter: None,
            changed_filter: None,
            _marker: PhantomData,
        }
    }

    pub(crate) fn query_mut_iter<'a, P: WriteFetch<'a>>(&'a mut self) -> QueryIterMut<'a, P> {
        let mut accesses = Vec::new();
        for (arch_id, col_idx) in self.cached_single_plan(P::type_id(), &[], &[], &[], &[]) {
            let arch = self.archetypes_mut().get_mut(arch_id);
            let len = arch.len();
            let col = &mut arch.columns_mut()[col_idx] as *mut _;
            let entities = arch.entities().as_ptr();
            let scene = self as *mut Scene;
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
            added_filter: None,
            changed_filter: None,
            _marker: PhantomData,
        }
    }

    pub fn query<'a, Data, Filters>(
        &'a self,
    ) -> <crate::query::QueryParam<Data, Filters> as QuerySpec<'a>>::Builder
    where
        crate::query::QueryParam<Data, Filters>: QuerySpec<'a>,
    {
        <crate::query::QueryParam<Data, Filters> as QuerySpec<'a>>::build(self)
    }

    pub fn query_mut<'a, Data, Filters>(
        &'a mut self,
    ) -> <crate::query::QueryParam<Data, Filters> as QuerySpecMut<'a>>::Builder
    where
        crate::query::QueryParam<Data, Filters>: QuerySpecMut<'a>,
    {
        <crate::query::QueryParam<Data, Filters> as QuerySpecMut<'a>>::build(self)
    }
}
