use crate::{Component, Scene};
use std::any::TypeId;
use std::marker::PhantomData;

fn has_duplicate_type_ids(ids: &[TypeId]) -> bool {
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            if ids[i] == ids[j] {
                return true;
            }
        }
    }
    false
}

macro_rules! for_each_tuple_arity {
    ($m:ident) => {
        $m!(
            Query2Builder,
            Query2,
            Query2Access,
            Query2MutBuilder,
            Query2Mut,
            Query2MutAccess,
            A,
            a_col,
            B,
            b_idx,
            b_col
        );
        $m!(
            Query3Builder,
            Query3,
            Query3Access,
            Query3MutBuilder,
            Query3Mut,
            Query3MutAccess,
            A,
            a_col,
            B,
            b_idx,
            b_col,
            C,
            c_idx,
            c_col
        );
        $m!(
            Query4Builder,
            Query4,
            Query4Access,
            Query4MutBuilder,
            Query4Mut,
            Query4MutAccess,
            A,
            a_col,
            B,
            b_idx,
            b_col,
            C,
            c_idx,
            c_col,
            D,
            d_idx,
            d_col
        );
        $m!(
            Query5Builder,
            Query5,
            Query5Access,
            Query5MutBuilder,
            Query5Mut,
            Query5MutAccess,
            A,
            a_col,
            B,
            b_idx,
            b_col,
            C,
            c_idx,
            c_col,
            D,
            d_idx,
            d_col,
            E,
            e_idx,
            e_col
        );
        $m!(
            Query6Builder,
            Query6,
            Query6Access,
            Query6MutBuilder,
            Query6Mut,
            Query6MutAccess,
            A,
            a_col,
            B,
            b_idx,
            b_col,
            C,
            c_idx,
            c_col,
            D,
            d_idx,
            d_col,
            E,
            e_idx,
            e_col,
            F,
            f_idx,
            f_col
        );
        $m!(
            Query7Builder,
            Query7,
            Query7Access,
            Query7MutBuilder,
            Query7Mut,
            Query7MutAccess,
            A,
            a_col,
            B,
            b_idx,
            b_col,
            C,
            c_idx,
            c_col,
            D,
            d_idx,
            d_col,
            E,
            e_idx,
            e_col,
            F,
            f_idx,
            f_col,
            G,
            g_idx,
            g_col
        );
        $m!(
            Query8Builder,
            Query8,
            Query8Access,
            Query8MutBuilder,
            Query8Mut,
            Query8MutAccess,
            A,
            a_col,
            B,
            b_idx,
            b_col,
            C,
            c_idx,
            c_col,
            D,
            d_idx,
            d_col,
            E,
            e_idx,
            e_col,
            F,
            f_idx,
            f_col,
            G,
            g_idx,
            g_col,
            H,
            h_idx,
            h_col
        );
    };
}

struct QueryAccess {
    col: *const comet_structs::Column,
    len: usize,
    row: usize,
}

struct QueryMutAccess {
    col: *mut comet_structs::Column,
    len: usize,
    row: usize,
}

pub trait ReadFetch<'a> {
    type Component: Component;
    type Item;

    fn type_id() -> TypeId {
        TypeId::of::<Self::Component>()
    }

    unsafe fn get(col: *const comet_structs::Column, row: usize) -> Option<Self::Item>;

    fn as_ref(item: &Self::Item) -> &Self::Component;
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
}

pub trait WriteFetch<'a> {
    type Component: Component;
    type Item;

    fn type_id() -> TypeId {
        TypeId::of::<Self::Component>()
    }

    unsafe fn get(col: *mut comet_structs::Column, row: usize) -> Option<Self::Item>;

    fn as_ref(item: &Self::Item) -> &Self::Component;
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
}

pub struct QueryIter<'a, P: ReadFetch<'a>> {
    accesses: Vec<QueryAccess>,
    idx: usize,
    _marker: PhantomData<&'a P>,
}

pub struct QueryIterMut<'a, P: WriteFetch<'a>> {
    accesses: Vec<QueryMutAccess>,
    idx: usize,
    _marker: PhantomData<&'a P>,
}

pub struct QueryBuilder<'a, P: ReadFetch<'a>> {
    scene: &'a Scene,
    tags: Vec<TypeId>,
    without_tags: Vec<TypeId>,
    _marker: PhantomData<P>,
}

pub struct Query<'a, P: WriteFetch<'a>> {
    scene: &'a mut Scene,
    tags: Vec<TypeId>,
    without_tags: Vec<TypeId>,
    _marker: PhantomData<P>,
}

pub struct QueryBuilderFiltered<'a, P: ReadFetch<'a>, F>
where
    F: Fn(&P::Component) -> bool + 'a,
{
    scene: &'a Scene,
    tags: Vec<TypeId>,
    without_tags: Vec<TypeId>,
    filter: F,
    _marker: PhantomData<P>,
}

pub struct QueryMutBuilderFiltered<'a, P: WriteFetch<'a>, F>
where
    F: Fn(&P::Component) -> bool + 'a,
{
    scene: &'a mut Scene,
    tags: Vec<TypeId>,
    without_tags: Vec<TypeId>,
    filter: F,
    _marker: PhantomData<P>,
}

pub struct QueryIterFiltered<'a, P: ReadFetch<'a>, F>
where
    F: Fn(&P::Component) -> bool + 'a,
{
    inner: QueryIter<'a, P>,
    filter: F,
    _marker: PhantomData<&'a P>,
}

pub struct QueryIterMutFiltered<'a, P: WriteFetch<'a>, F>
where
    F: Fn(&P::Component) -> bool + 'a,
{
    inner: QueryIterMut<'a, P>,
    filter: F,
    _marker: PhantomData<&'a P>,
}

pub trait QuerySpec<'a> {
    type Builder;
    fn build(scene: &'a Scene) -> Self::Builder;
}

pub trait QuerySpecMut<'a> {
    type Builder;
    fn build(scene: &'a mut Scene) -> Self::Builder;
}

mod builders;
mod iterators;
mod scene_query;
mod tuple_types;
