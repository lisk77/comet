use super::tuple_types::*;
use super::*;

impl<'a, C: Component> Iterator for QueryIter<'a, C> {
    type Item = &'a C;

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
                let col = &*access.col;
                return col.get::<C>(row);
            }
        }
    }
}

impl<'a, C: Component> Iterator for QueryIterMut<'a, C> {
    type Item = &'a mut C;

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
                let col = &mut *access.col;
                return col.get_mut::<C>(row);
            }
        }
    }
}

impl<'a, C: Component, F> Iterator for QueryIterFiltered<'a, C, F>
where
    F: Fn(&C) -> bool + 'a,
{
    type Item = &'a C;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.inner.next()?;
            if (self.filter)(item) {
                return Some(item);
            }
        }
    }
}

impl<'a, C: Component, F> Iterator for QueryIterMutFiltered<'a, C, F>
where
    F: Fn(&C) -> bool + 'a,
{
    type Item = &'a mut C;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.inner.next()?;
            if (self.filter)(&*item) {
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
        impl<'a, $first_ty: Component, $($ty: Component),+> Iterator for $iter<'a, $first_ty, $($ty),+> {
            type Item = (&'a $first_ty, $(&'a $ty),+);

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
                            (&*access.$first_col).get::<$first_ty>(row)?,
                            $((&*access.$col).get::<$ty>(row)?,)+
                        ));
                    }
                }
            }
        }

        impl<'a, $first_ty: Component, $($ty: Component),+> Iterator for $iter_mut<'a, $first_ty, $($ty),+> {
            type Item = (&'a mut $first_ty, $(&'a mut $ty),+);

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
                            (&mut *access.$first_col).get_mut::<$first_ty>(row)?,
                            $((&mut *access.$col).get_mut::<$ty>(row)?,)+
                        ));
                    }
                }
            }
        }
    };
}

for_each_tuple_arity!(impl_tuple_iterators_arity);
