use super::tuple_types::*;
use super::*;

impl<'a, P: ReadFetch<'a>> Iterator for QueryIter<'a, P> {
    type Item = P::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let access = self.accesses.get_mut(self.idx)?;
            if access.row >= access.len {
                self.idx += 1;
                continue;
            }
            let row = access.row;
            access.row += 1;
            unsafe { return P::get(access.col, row); }
        }
    }
}

impl<'a, P: WriteFetch<'a>> Iterator for QueryIterMut<'a, P> {
    type Item = P::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let access = self.accesses.get_mut(self.idx)?;
            if access.row >= access.len {
                self.idx += 1;
                continue;
            }
            let row = access.row;
            access.row += 1;
            unsafe { return P::get(access.col, row); }
        }
    }
}

impl<'a, P: ReadFetch<'a>, F> Iterator for QueryIterFiltered<'a, P, F>
where
    F: Fn(&P::Component) -> bool + 'a,
{
    type Item = P::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.inner.next()?;
            if (self.filter)(P::as_ref(&item)) {
                return Some(item);
            }
        }
    }
}

impl<'a, P: WriteFetch<'a>, F> Iterator for QueryIterMutFiltered<'a, P, F>
where
    F: Fn(&P::Component) -> bool + 'a,
{
    type Item = P::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.inner.next()?;
            if (self.filter)(P::as_ref(&item)) {
                return Some(item);
            }
        }
    }
}

macro_rules! impl_tuple_iterators_arity {
    (
        $builder:ident,
        $iter:ident,
        $access:ident,
        $builder_mut:ident,
        $iter_mut:ident,
        $access_mut:ident,
        $first_ty:ident,
        $first_col:ident,
        $($ty:ident, $idx:ident, $col:ident),+
    ) => {
        impl<'a, $first_ty: ReadFetch<'a>, $($ty: ReadFetch<'a>),+> Iterator for $iter<'a, $first_ty, $($ty),+> {
            type Item = ($first_ty::Item, $($ty::Item),+);

            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    let access = self.accesses.get_mut(self.idx)?;
                    if access.row >= access.len {
                        self.idx += 1;
                        continue;
                    }
                    let row = access.row;
                    access.row += 1;
                    unsafe {
                        return Some((
                            $first_ty::get(access.$first_col, row)?,
                            $($ty::get(access.$col, row)?,)+
                        ));
                    }
                }
            }
        }

        impl<'a, $first_ty: WriteFetch<'a>, $($ty: WriteFetch<'a>),+> Iterator for $iter_mut<'a, $first_ty, $($ty),+> {
            type Item = ($first_ty::Item, $($ty::Item),+);

            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    let access = self.accesses.get_mut(self.idx)?;
                    if access.row >= access.len {
                        self.idx += 1;
                        continue;
                    }
                    let row = access.row;
                    access.row += 1;
                    unsafe {
                        return Some((
                            $first_ty::get(access.$first_col, row)?,
                            $($ty::get(access.$col, row)?,)+
                        ));
                    }
                }
            }
        }
    };
}

for_each_tuple_arity!(impl_tuple_iterators_arity);
