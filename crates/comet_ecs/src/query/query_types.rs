use super::*;

pub struct Query<'a, Data, Filters = ()> {
    pub(crate) accesses: Vec<QueryAccess>,
    pub(crate) idx: usize,
    pub(crate) added_since_filters: Vec<(TypeId, Tick)>,
    pub(crate) changed_since_filters: Vec<(TypeId, Tick)>,
    pub(crate) _marker: PhantomData<(&'a (), Data, Filters)>,
}

impl<'a, Data, Filters> Query<'a, Data, Filters> {
    pub(crate) fn from_state(scene: &'a Scene, state: QueryFilterState) -> Self
    where
        Data: QueryData<'a>,
    {
        Self {
            accesses: build_query_accesses::<Data>(scene, &state),
            idx: 0,
            added_since_filters: state.added_since_filters,
            changed_since_filters: state.changed_since_filters,
            _marker: PhantomData,
        }
    }

    pub(crate) fn from_state_mut(scene: &'a mut Scene, state: QueryFilterState) -> Self
    where
        Data: QueryData<'a>,
    {
        Self {
            accesses: build_query_accesses_mut::<Data>(scene, &state),
            idx: 0,
            added_since_filters: state.added_since_filters,
            changed_since_filters: state.changed_since_filters,
            _marker: PhantomData,
        }
    }

    pub fn iter(self) -> Self {
        self
    }

    pub fn added_since<C: Component>(mut self, tick: Tick) -> Self {
        set_since_filter(&mut self.added_since_filters, C::type_id(), tick);
        self
    }

    pub fn changed_since<C: Component>(mut self, tick: Tick) -> Self {
        set_since_filter(&mut self.changed_since_filters, C::type_id(), tick);
        self
    }
}

fn set_since_filter(filters: &mut Vec<(TypeId, Tick)>, type_id: TypeId, tick: Tick) {
    if let Some((_, existing_tick)) = filters
        .iter_mut()
        .find(|(existing_type_id, _)| *existing_type_id == type_id)
    {
        *existing_tick = tick;
    } else {
        filters.push((type_id, tick));
    }
}
