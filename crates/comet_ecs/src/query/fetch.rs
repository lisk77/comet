use super::*;

pub(crate) struct QueryAccess {
    pub(crate) entities: *const Entity,
    pub(crate) columns: [*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
    pub(crate) component_types: [Option<TypeId>; MAX_QUERY_COMPONENTS],
    pub(crate) casters: [Option<crate::QueryCaster>; MAX_QUERY_COMPONENTS],
    pub(crate) change_state: *mut HashMap<(u32, TypeId), ComponentChangeState>,
    pub(crate) component_event_tick: Tick,
    pub(crate) added_since_filters: Vec<ResolvedChangeFilter>,
    pub(crate) changed_since_filters: Vec<ResolvedChangeFilter>,
    pub(crate) len: usize,
    pub(crate) row: usize,
}

pub(crate) trait EntityFetch {
    type Item;

    unsafe fn get(entities: *const Entity, len: usize, row: usize) -> Option<Self::Item>;
}

impl EntityFetch for Entity {
    type Item = Entity;

    unsafe fn get(entities: *const Entity, len: usize, row: usize) -> Option<Self::Item> {
        if row >= len {
            return None;
        }

        Some(unsafe { *entities.add(row) })
    }
}

pub(crate) trait ReadFetch<'a> {}

impl<'a, C: ?Sized + Component> ReadFetch<'a> for &'a C {}
impl<'a, T: ReadFetch<'a>> ReadFetch<'a> for Option<T> {}

pub(crate) trait WriteFetch<'a>: QueryItem<'a> {
    type Target: ?Sized + Component;

    fn type_id() -> TypeId {
        TypeId::of::<Self::Target>()
    }

    unsafe fn get(
        col: *mut comet_structs::Column,
        caster: Option<&crate::QueryCaster>,
        row: usize,
    ) -> Option<Self::Item>;

    fn writes() -> bool;

    fn required() -> bool {
        true
    }

    fn selectors() -> Vec<QuerySelector> {
        Vec::new()
    }
}

impl<'a, C: ?Sized + Component> WriteFetch<'a> for &'a mut C {
    type Target = C;

    unsafe fn get(
        col: *mut comet_structs::Column,
        caster: Option<&crate::QueryCaster>,
        row: usize,
    ) -> Option<Self::Item> {
        let caster = caster?;
        let value = unsafe { (&*col).get_raw(row) };
        Some(unsafe { &mut *caster.cast_mut::<C>(value) })
    }

    fn writes() -> bool {
        true
    }
}

impl<'a, C: ?Sized + Component> WriteFetch<'a> for &'a C {
    type Target = C;

    unsafe fn get(
        col: *mut comet_structs::Column,
        caster: Option<&crate::QueryCaster>,
        row: usize,
    ) -> Option<Self::Item> {
        let caster = caster?;
        let value = unsafe { (&*col).get_raw(row) };
        Some(unsafe { &*caster.cast_ref::<C>(value) })
    }

    fn writes() -> bool {
        false
    }
}

impl<'a, T> WriteFetch<'a> for Option<T>
where
    T: WriteFetch<'a>,
{
    type Target = T::Target;

    unsafe fn get(
        col: *mut comet_structs::Column,
        caster: Option<&crate::QueryCaster>,
        row: usize,
    ) -> Option<Self::Item> {
        if col.is_null() {
            return Some(None);
        }
        unsafe { T::get(col, caster, row).map(Some) }
    }

    fn writes() -> bool {
        T::writes()
    }

    fn required() -> bool {
        false
    }

    fn selectors() -> Vec<QuerySelector> {
        T::selectors()
    }
}
