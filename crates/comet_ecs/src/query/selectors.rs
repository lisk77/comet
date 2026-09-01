use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum QuerySelector {
    Range { start: usize, end: usize },
    Skip(usize),
    Take(usize),
    First,
    Last,
}

impl QuerySelector {
    pub(crate) fn select<T>(self, values: Vec<T>) -> Vec<T> {
        let len = values.len();
        let (start, end) = match self {
            Self::Range { start, end } => (start.min(len), end.min(len)),
            Self::Skip(count) => (count.min(len), len),
            Self::Take(count) => (0, count.min(len)),
            Self::First => (0, 1.min(len)),
            Self::Last => (len.saturating_sub(1), len),
        };
        values
            .into_iter()
            .skip(start)
            .take(end.saturating_sub(start))
            .collect()
    }
}

pub struct Range<T, const START: usize, const END: usize>(PhantomData<T>);
pub struct Skip<T, const COUNT: usize>(PhantomData<T>);
pub struct Take<T, const COUNT: usize>(PhantomData<T>);
pub struct First<T>(PhantomData<T>);
pub struct Last<T>(PhantomData<T>);

impl<'a, T: QueryItem<'a>, const START: usize, const END: usize> QueryItem<'a>
    for Range<T, START, END>
{
    type Item = T::Item;
}

impl<'a, T: QueryItem<'a>, const COUNT: usize> QueryItem<'a> for Skip<T, COUNT> {
    type Item = T::Item;
}

impl<'a, T: QueryItem<'a>, const COUNT: usize> QueryItem<'a> for Take<T, COUNT> {
    type Item = T::Item;
}

impl<'a, T: QueryItem<'a>> QueryItem<'a> for First<T> {
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

    fn selectors() -> Vec<QuerySelector> {
        let mut selectors = T::selectors();
        selectors.push(QuerySelector::Range {
            start: START,
            end: END,
        });
        selectors
    }
}

impl<'a, T, const START: usize, const END: usize> ReadFetch<'a> for Range<T, START, END> where
    T: ReadFetch<'a> + WriteFetch<'a>
{
}

impl<'a, T, const COUNT: usize> WriteFetch<'a> for Skip<T, COUNT>
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

    fn selectors() -> Vec<QuerySelector> {
        let mut selectors = T::selectors();
        selectors.push(QuerySelector::Skip(COUNT));
        selectors
    }
}

impl<'a, T, const COUNT: usize> ReadFetch<'a> for Skip<T, COUNT> where
    T: ReadFetch<'a> + WriteFetch<'a>
{
}

impl<'a, T, const COUNT: usize> WriteFetch<'a> for Take<T, COUNT>
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

    fn selectors() -> Vec<QuerySelector> {
        let mut selectors = T::selectors();
        selectors.push(QuerySelector::Take(COUNT));
        selectors
    }
}

impl<'a, T, const COUNT: usize> ReadFetch<'a> for Take<T, COUNT> where
    T: ReadFetch<'a> + WriteFetch<'a>
{
}

impl<'a, T> WriteFetch<'a> for First<T>
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

    fn selectors() -> Vec<QuerySelector> {
        let mut selectors = T::selectors();
        selectors.push(QuerySelector::First);
        selectors
    }
}

impl<'a, T> ReadFetch<'a> for First<T> where T: ReadFetch<'a> + WriteFetch<'a> {}

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

    fn selectors() -> Vec<QuerySelector> {
        let mut selectors = T::selectors();
        selectors.push(QuerySelector::Last);
        selectors
    }
}

impl<'a, T> ReadFetch<'a> for Last<T> where T: ReadFetch<'a> + WriteFetch<'a> {}
