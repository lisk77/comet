use super::*;
use super::tuple_types::*;

impl<'a, C: Component> Iterator for Query<'a, C> {
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

impl<'a, C: Component> Iterator for QueryMut<'a, C> {
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

impl<'a, A: Component, B: Component> Iterator for QueryPair<'a, A, B> {
    type Item = (&'a A, &'a B);

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
                let a_col = &*access.a_col;
                let b_col = &*access.b_col;
                let a = a_col.get::<A>(row)?;
                let b = b_col.get::<B>(row)?;
                return Some((a, b));
            }
        }
    }
}

impl<'a, A: Component, B: Component> Iterator for QueryPairMut<'a, A, B> {
    type Item = (&'a mut A, &'a mut B);

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
                let a_col = &mut *access.a_col;
                let b_col = &mut *access.b_col;
                let a = a_col.get_mut::<A>(row)?;
                let b = b_col.get_mut::<B>(row)?;
                return Some((a, b));
            }
        }
    }
}

impl<'a, C: Component, F> Iterator for QueryFiltered<'a, C, F>
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

impl<'a, A: Component, B: Component, F> Iterator for QueryPairFiltered<'a, A, B, F>
where
    F: Fn(&A, &B) -> bool + 'a,
{
    type Item = (&'a A, &'a B);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (a, b) = self.inner.next()?;
            if (self.filter)(a, b) {
                return Some((a, b));
            }
        }
    }
}

impl<'a, C: Component, F> Iterator for QueryMutFiltered<'a, C, F>
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

impl<'a, A: Component, B: Component, F> Iterator for QueryPairMutFiltered<'a, A, B, F>
where
    F: Fn(&A, &B) -> bool + 'a,
{
    type Item = (&'a mut A, &'a mut B);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (a, b) = self.inner.next()?;
            if (self.filter)(&*a, &*b) {
                return Some((a, b));
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
