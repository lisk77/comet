use super::*;

pub(crate) struct QueryAccess {
    pub(crate) entities: *const Entity,
    pub(crate) columns: [*mut comet_structs::Column; MAX_QUERY_COMPONENTS],
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

impl<'a, C: Component> ReadFetch<'a> for &'a C {}
impl<'a, C: Component> ReadFetch<'a> for Option<&'a C> {}

pub trait WriteFetch<'a> {
    type Component: Component;
    type Item;

    fn type_id() -> TypeId {
        TypeId::of::<Self::Component>()
    }

    unsafe fn get(col: *mut comet_structs::Column, row: usize) -> Option<Self::Item>;

    fn writes() -> bool;

    fn required() -> bool {
        true
    }
}

impl<'a, C: Component> WriteFetch<'a> for &'a mut C {
    type Component = C;
    type Item = &'a mut C;

    unsafe fn get(col: *mut comet_structs::Column, row: usize) -> Option<Self::Item> {
        unsafe { (&mut *col).get_mut::<C>(row) }
    }

    fn writes() -> bool {
        true
    }
}

impl<'a, C: Component> WriteFetch<'a> for &'a C {
    type Component = C;
    type Item = &'a C;

    unsafe fn get(col: *mut comet_structs::Column, row: usize) -> Option<Self::Item> {
        unsafe { (&*col).get::<C>(row) }
    }

    fn writes() -> bool {
        false
    }
}

impl<'a, C: Component> WriteFetch<'a> for Option<&'a C> {
    type Component = C;
    type Item = Option<&'a C>;

    unsafe fn get(col: *mut comet_structs::Column, row: usize) -> Option<Self::Item> {
        if col.is_null() {
            return Some(None);
        }
        Some(unsafe { (&*col).get::<C>(row) })
    }

    fn writes() -> bool {
        false
    }

    fn required() -> bool {
        false
    }
}

impl<'a, C: Component> WriteFetch<'a> for Option<&'a mut C> {
    type Component = C;
    type Item = Option<&'a mut C>;

    unsafe fn get(col: *mut comet_structs::Column, row: usize) -> Option<Self::Item> {
        if col.is_null() {
            return Some(None);
        }
        Some(unsafe { (&mut *col).get_mut::<C>(row) })
    }

    fn writes() -> bool {
        true
    }

    fn required() -> bool {
        false
    }
}
