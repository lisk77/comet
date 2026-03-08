use crate::{Component, Scene, Tag};
use comet_log::error;
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

struct QueryPairAccess {
    a_col: *const comet_structs::Column,
    b_col: *const comet_structs::Column,
    len: usize,
    row: usize,
}

struct QueryPairMutAccess {
    a_col: *mut comet_structs::Column,
    b_col: *mut comet_structs::Column,
    len: usize,
    row: usize,
}

pub struct Query<'a, C: Component> {
    accesses: Vec<QueryAccess>,
    idx: usize,
    _marker: PhantomData<&'a C>,
}

pub struct QueryMut<'a, C: Component> {
    accesses: Vec<QueryMutAccess>,
    idx: usize,
    _marker: PhantomData<&'a mut C>,
}

pub struct QueryPair<'a, A: Component, B: Component> {
    accesses: Vec<QueryPairAccess>,
    idx: usize,
    _marker: PhantomData<(&'a A, &'a B)>,
}

pub struct QueryPairMut<'a, A: Component, B: Component> {
    accesses: Vec<QueryPairMutAccess>,
    idx: usize,
    _marker: PhantomData<(&'a mut A, &'a mut B)>,
}

pub struct QueryBuilder<'a, C: Component> {
    scene: &'a Scene,
    tags: Vec<TypeId>,
    _marker: PhantomData<&'a C>,
}

pub struct QueryPairBuilder<'a, A: Component, B: Component> {
    scene: &'a Scene,
    tags: Vec<TypeId>,
    _marker: PhantomData<(&'a A, &'a B)>,
}

pub struct QueryMutBuilder<'a, C: Component> {
    scene: &'a mut Scene,
    tags: Vec<TypeId>,
    _marker: PhantomData<&'a mut C>,
}

pub struct QueryPairMutBuilder<'a, A: Component, B: Component> {
    scene: &'a mut Scene,
    tags: Vec<TypeId>,
    _marker: PhantomData<(&'a mut A, &'a mut B)>,
}

pub struct QueryBuilderFiltered<'a, C: Component, F>
where
    F: Fn(&C) -> bool + 'a,
{
    scene: &'a Scene,
    tags: Vec<TypeId>,
    filter: F,
    _marker: PhantomData<&'a C>,
}

pub struct QueryPairBuilderFiltered<'a, A: Component, B: Component, F>
where
    F: Fn(&A, &B) -> bool + 'a,
{
    scene: &'a Scene,
    tags: Vec<TypeId>,
    filter: F,
    _marker: PhantomData<(&'a A, &'a B)>,
}

pub struct QueryMutBuilderFiltered<'a, C: Component, F>
where
    F: Fn(&C) -> bool + 'a,
{
    scene: &'a mut Scene,
    tags: Vec<TypeId>,
    filter: F,
    _marker: PhantomData<&'a mut C>,
}

pub struct QueryPairMutBuilderFiltered<'a, A: Component, B: Component, F>
where
    F: Fn(&A, &B) -> bool + 'a,
{
    scene: &'a mut Scene,
    tags: Vec<TypeId>,
    filter: F,
    _marker: PhantomData<(&'a mut A, &'a mut B)>,
}

pub struct QueryFiltered<'a, C: Component, F>
where
    F: Fn(&C) -> bool + 'a,
{
    inner: Query<'a, C>,
    filter: F,
}

pub struct QueryPairFiltered<'a, A: Component, B: Component, F>
where
    F: Fn(&A, &B) -> bool + 'a,
{
    inner: QueryPair<'a, A, B>,
    filter: F,
}

pub struct QueryMutFiltered<'a, C: Component, F>
where
    F: Fn(&C) -> bool + 'a,
{
    inner: QueryMut<'a, C>,
    filter: F,
}

pub struct QueryPairMutFiltered<'a, A: Component, B: Component, F>
where
    F: Fn(&A, &B) -> bool + 'a,
{
    inner: QueryPairMut<'a, A, B>,
    filter: F,
}

pub trait QueryTuple<'a> {
    type Builder;
    fn build(scene: &'a Scene) -> Self::Builder;
}

pub trait QueryTupleMut<'a> {
    type Builder;
    fn build(scene: &'a mut Scene) -> Self::Builder;
}

mod builders;
mod iterators;
mod scene_query;
mod tuple_types;
