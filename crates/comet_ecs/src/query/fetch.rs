use super::*;

pub(crate) struct QueryAccess {
    pub(crate) entities: *const Entity,
    pub(crate) columns: [*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
    pub(crate) component_types: [Option<TypeId>; MAX_QUERY_COMPONENTS],
    pub(crate) casters: [Option<Arc<dyn Any + Send + Sync>>; MAX_QUERY_COMPONENTS],
    pub(crate) change_state: *mut HashMap<(u32, TypeId), ComponentChangeState>,
    pub(crate) component_event_tick: Tick,
    pub(crate) len: usize,
    pub(crate) row: usize,
}

pub trait EntityFetch {
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

pub trait ReadFetch<'a> {}

impl<'a, C: ?Sized + Component> ReadFetch<'a> for &'a C {}
impl<'a, C: ?Sized + Component> ReadFetch<'a> for Option<&'a C> {}

pub trait WriteFetch<'a> {
    type Target: ?Sized + Component;
    type Item;

    fn type_id() -> TypeId {
        TypeId::of::<Self::Target>()
    }

    unsafe fn get(
        col: *mut comet_structs::Column,
        caster: Option<&(dyn Any + Send + Sync)>,
        row: usize,
    ) -> Option<Self::Item>;

    fn writes() -> bool;

    fn required() -> bool {
        true
    }
}

impl<'a, C: ?Sized + Component> WriteFetch<'a> for &'a mut C {
    type Target = C;
    type Item = &'a mut C;

    unsafe fn get(
        col: *mut comet_structs::Column,
        caster: Option<&(dyn Any + Send + Sync)>,
        row: usize,
    ) -> Option<Self::Item> {
        let caster = caster?.downcast_ref::<crate::QueryCaster<C>>()?;
        let value = unsafe { (&*col).get_raw(row) };
        Some(unsafe { &mut *caster.cast_mut(value) })
    }

    fn writes() -> bool {
        true
    }
}

impl<'a, C: ?Sized + Component> WriteFetch<'a> for &'a C {
    type Target = C;
    type Item = &'a C;

    unsafe fn get(
        col: *mut comet_structs::Column,
        caster: Option<&(dyn Any + Send + Sync)>,
        row: usize,
    ) -> Option<Self::Item> {
        let caster = caster?.downcast_ref::<crate::QueryCaster<C>>()?;
        let value = unsafe { (&*col).get_raw(row) };
        Some(unsafe { &*caster.cast_ref(value) })
    }

    fn writes() -> bool {
        false
    }
}

impl<'a, C: ?Sized + Component> WriteFetch<'a> for Option<&'a C> {
    type Target = C;
    type Item = Option<&'a C>;

    unsafe fn get(
        col: *mut comet_structs::Column,
        caster: Option<&(dyn Any + Send + Sync)>,
        row: usize,
    ) -> Option<Self::Item> {
        if col.is_null() {
            return Some(None);
        }
        let caster = caster?.downcast_ref::<crate::QueryCaster<C>>()?;
        let value = unsafe { (&*col).get_raw(row) };
        Some(Some(unsafe { &*caster.cast_ref(value) }))
    }

    fn writes() -> bool {
        false
    }

    fn required() -> bool {
        false
    }
}

impl<'a, C: ?Sized + Component> WriteFetch<'a> for Option<&'a mut C> {
    type Target = C;
    type Item = Option<&'a mut C>;

    unsafe fn get(
        col: *mut comet_structs::Column,
        caster: Option<&(dyn Any + Send + Sync)>,
        row: usize,
    ) -> Option<Self::Item> {
        if col.is_null() {
            return Some(None);
        }
        let caster = caster?.downcast_ref::<crate::QueryCaster<C>>()?;
        let value = unsafe { (&*col).get_raw(row) };
        Some(Some(unsafe { &mut *caster.cast_mut(value) }))
    }

    fn writes() -> bool {
        true
    }

    fn required() -> bool {
        false
    }
}
