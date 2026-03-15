use super::*;

pub(crate) struct QueryAccess {
    pub(crate) entities: *const Entity,
    pub(crate) scene: *const Scene,
    pub(crate) col: *const comet_structs::Column,
    pub(crate) len: usize,
    pub(crate) row: usize,
}

pub(crate) struct QueryMutAccess {
    pub(crate) entities: *const Entity,
    pub(crate) col: *mut comet_structs::Column,
    pub(crate) scene: *mut Scene,
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

        Some(*entities.add(row))
    }
}

pub trait ReadFetch<'a> {
    type Component: Component;
    type Item;

    fn type_id() -> TypeId {
        TypeId::of::<Self::Component>()
    }

    unsafe fn get(col: *const comet_structs::Column, row: usize) -> Option<Self::Item>;

    fn as_ref(item: &Self::Item) -> &Self::Component;

    fn required() -> bool {
        true
    }

    fn is_present(item: &Self::Item) -> bool;
}

impl<'a, C: Component> ReadFetch<'a> for &'a C {
    type Component = C;
    type Item = &'a C;

    unsafe fn get(col: *const comet_structs::Column, row: usize) -> Option<Self::Item> {
        unsafe { (&*col).get::<C>(row) }
    }

    fn as_ref(item: &Self::Item) -> &Self::Component {
        item
    }

    fn is_present(_item: &Self::Item) -> bool {
        true
    }
}

impl<'a, C: Component> ReadFetch<'a> for Option<&'a C> {
    type Component = C;
    type Item = Option<&'a C>;

    unsafe fn get(col: *const comet_structs::Column, row: usize) -> Option<Self::Item> {
        if col.is_null() {
            return Some(None);
        }
        Some(unsafe { (&*col).get::<C>(row) })
    }

    fn as_ref(item: &Self::Item) -> &Self::Component {
        item.expect("optional query filter expected component to be present")
    }

    fn required() -> bool {
        false
    }

    fn is_present(item: &Self::Item) -> bool {
        item.is_some()
    }
}

pub trait WriteFetch<'a> {
    type Component: Component;
    type Item;

    fn type_id() -> TypeId {
        TypeId::of::<Self::Component>()
    }

    unsafe fn get(col: *mut comet_structs::Column, row: usize) -> Option<Self::Item>;

    fn as_ref(item: &Self::Item) -> &Self::Component;

    fn writes() -> bool;

    fn required() -> bool {
        true
    }

    fn is_present(item: &Self::Item) -> bool;
}

impl<'a, C: Component> WriteFetch<'a> for &'a mut C {
    type Component = C;
    type Item = &'a mut C;

    unsafe fn get(col: *mut comet_structs::Column, row: usize) -> Option<Self::Item> {
        unsafe { (&mut *col).get_mut::<C>(row) }
    }

    fn as_ref(item: &Self::Item) -> &Self::Component {
        item
    }

    fn writes() -> bool {
        true
    }

    fn is_present(_item: &Self::Item) -> bool {
        true
    }
}

impl<'a, C: Component> WriteFetch<'a> for &'a C {
    type Component = C;
    type Item = &'a C;

    unsafe fn get(col: *mut comet_structs::Column, row: usize) -> Option<Self::Item> {
        unsafe { (&*col).get::<C>(row) }
    }

    fn as_ref(item: &Self::Item) -> &Self::Component {
        item
    }

    fn writes() -> bool {
        false
    }

    fn is_present(_item: &Self::Item) -> bool {
        true
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

    fn as_ref(item: &Self::Item) -> &Self::Component {
        item.expect("optional query filter expected component to be present")
    }

    fn writes() -> bool {
        false
    }

    fn required() -> bool {
        false
    }

    fn is_present(item: &Self::Item) -> bool {
        item.is_some()
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

    fn as_ref(item: &Self::Item) -> &Self::Component {
        item.as_deref()
            .expect("optional query filter expected component to be present")
    }

    fn writes() -> bool {
        true
    }

    fn required() -> bool {
        false
    }

    fn is_present(item: &Self::Item) -> bool {
        item.is_some()
    }
}
