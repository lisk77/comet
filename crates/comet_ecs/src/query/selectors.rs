use super::*;

#[derive(Clone, Copy)]
enum QueryRangeBound {
    Index(usize),
    End,
    FromEnd(usize),
}

impl QueryRangeBound {
    fn resolve(self, len: usize) -> usize {
        match self {
            Self::Index(index) => index.min(len),
            Self::End => len,
            Self::FromEnd(offset) => len.saturating_sub(offset),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct QueryRange {
    start: QueryRangeBound,
    end: QueryRangeBound,
}

impl QueryRange {
    fn fixed(start: usize, end: usize) -> Self {
        Self {
            start: QueryRangeBound::Index(start),
            end: QueryRangeBound::Index(end),
        }
    }

    fn last() -> Self {
        Self {
            start: QueryRangeBound::FromEnd(1),
            end: QueryRangeBound::End,
        }
    }

    pub(crate) fn select<T>(self, values: Vec<T>) -> Vec<T> {
        let len = values.len();
        let start = self.start.resolve(len);
        let end = self.end.resolve(len);
        values
            .into_iter()
            .skip(start)
            .take(end.saturating_sub(start))
            .collect()
    }
}

pub struct Range<T, const START: usize, const END: usize>(PhantomData<T>);
pub struct Last<T>(PhantomData<T>);

pub type Skip<T, const COUNT: usize> = Range<T, COUNT, { usize::MAX }>;
pub type Take<T, const COUNT: usize> = Range<T, 0, COUNT>;
pub type First<T> = Range<T, 0, 1>;

impl<'a, T: QueryItem<'a>, const START: usize, const END: usize> QueryItem<'a>
    for Range<T, START, END>
{
    type Item = T::Item;
}

impl<'a, T: QueryItem<'a>> QueryItem<'a> for Last<T> {
    type Item = T::Item;
}

impl<'a, T, const START: usize, const END: usize> WriteFetch<'a> for Range<T, START, END>
where
    T: WriteFetch<'a>,
{
    type Target = T::Target;

    unsafe fn get(
        col: *mut comet_structs::Column,
        caster: Option<&crate::QueryCaster>,
        row: usize,
    ) -> Option<Self::Item> {
        unsafe { T::get(col, caster, row) }
    }

    fn writes() -> bool {
        T::writes()
    }

    fn required() -> bool {
        T::required()
    }

    fn ranges() -> Vec<QueryRange> {
        let mut ranges = T::ranges();
        ranges.push(QueryRange::fixed(START, END));
        ranges
    }
}

impl<'a, T, const START: usize, const END: usize> ReadFetch<'a> for Range<T, START, END> where
    T: ReadFetch<'a> + WriteFetch<'a>
{
}

impl<'a, T> WriteFetch<'a> for Last<T>
where
    T: WriteFetch<'a>,
{
    type Target = T::Target;

    unsafe fn get(
        col: *mut comet_structs::Column,
        caster: Option<&crate::QueryCaster>,
        row: usize,
    ) -> Option<Self::Item> {
        unsafe { T::get(col, caster, row) }
    }

    fn writes() -> bool {
        T::writes()
    }

    fn required() -> bool {
        T::required()
    }

    fn ranges() -> Vec<QueryRange> {
        let mut ranges = T::ranges();
        ranges.push(QueryRange::last());
        ranges
    }
}

impl<'a, T> ReadFetch<'a> for Last<T> where T: ReadFetch<'a> + WriteFetch<'a> {}
