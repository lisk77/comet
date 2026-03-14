use super::*;

impl Scene {
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
