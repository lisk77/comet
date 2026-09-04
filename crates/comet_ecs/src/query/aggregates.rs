use super::*;

pub struct Amount<T>(PhantomData<T>);

impl<'a, C: ?Sized + Component> QueryItem<'a> for Amount<&'a C> {
    type Item = usize;
}

impl<'a, C: ?Sized + Component> QueryElement<'a> for Amount<&'a C> {
    fn info() -> QueryElementInfo {
        QueryElementInfo::Amount(QueryAmount {
            type_id: TypeId::of::<C>(),
            ranges: Vec::new(),
            row_order: 0,
        })
    }

    unsafe fn mark_changed(
        _change_state: *mut HashMap<(u32, TypeId), ComponentChangeState>,
        _component_event_tick: Tick,
        _entity: Entity,
        _columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
        _component_types: &[Option<TypeId>; MAX_QUERY_COMPONENTS],
        _component_slot: &mut usize,
    ) {
    }

    unsafe fn fetch(
        _entity: Entity,
        _columns: &[*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
        _casters: &[Option<crate::QueryCaster>; MAX_QUERY_COMPONENTS],
        amounts: &[usize],
        _row: usize,
        _component_slot: &mut usize,
        amount_slot: &mut usize,
    ) -> Option<Self::Item> {
        let amount = amounts.get(*amount_slot).copied();
        *amount_slot += 1;
        amount
    }
}

impl<'a, C: ?Sized + Component> ReadQueryElement<'a> for Amount<&'a C> {}
