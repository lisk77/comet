use super::*;

pub struct Query<'a, Data, Filters = ()> {
    pub(crate) accesses: Vec<QueryAccess>,
    pub(crate) idx: usize,
    pub(crate) entity_ranges: Vec<QueryRange>,
    pub(crate) row_ranges: Vec<QueryRange>,
    pub(crate) selected_entities: Option<HashSet<Entity>>,
    pub(crate) selected_rows: Option<HashSet<(usize, usize)>>,
    pub(crate) _marker: PhantomData<(&'a (), Data, Filters)>,
}

impl<'a, Data, Filters> Query<'a, Data, Filters> {
    pub(crate) fn from_state(scene: &'a Scene, state: QueryFilterState) -> Self
    where
        Data: QueryData<'a>,
    {
        let layout = Data::layout();
        let row_ranges = concrete_row_ranges(scene, &layout.components, &layout.amounts);
        let accesses = build_query_accesses(scene, &state, &layout);
        Self {
            accesses,
            idx: 0,
            entity_ranges: layout.entity_ranges,
            row_ranges,
            selected_entities: None,
            selected_rows: None,
            _marker: PhantomData,
        }
    }

    pub(crate) fn from_state_mut(scene: &'a mut Scene, state: QueryFilterState) -> Self
    where
        Data: QueryData<'a>,
    {
        let layout = Data::layout();
        let row_ranges = concrete_row_ranges(scene, &layout.components, &layout.amounts);
        let accesses = build_query_accesses_mut(scene, &state, &layout);
        Self {
            accesses,
            idx: 0,
            entity_ranges: layout.entity_ranges,
            row_ranges,
            selected_entities: None,
            selected_rows: None,
            _marker: PhantomData,
        }
    }

    pub fn baseline_tick(mut self, tick: Tick) -> Self {
        for access in &mut self.accesses {
            unsafe {
                access.filter.set_baseline_tick(
                    tick,
                    access.change_state,
                    access.entities,
                    access.len,
                )
            };
        }
        self.selected_entities = None;
        self.selected_rows = None;
        self
    }
}
