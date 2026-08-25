use super::*;

impl Scene {
    pub fn query<'a, Data, Filters>(&'a self) -> Query<'a, Data, Filters>
    where
        QueryParam<Data, Filters>: QuerySpec<'a, Data = Data, Filters = Filters>,
    {
        <QueryParam<Data, Filters> as QuerySpec<'a>>::build(self)
    }

    pub fn query_mut<'a, Data, Filters>(&'a self) -> Query<'a, Data, Filters>
    where
        QueryParam<Data, Filters>: QuerySpecMut<'a, Data = Data, Filters = Filters>,
    {
        <QueryParam<Data, Filters> as QuerySpecMut<'a>>::build(self)
    }
}
