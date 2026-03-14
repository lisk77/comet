use crate::{Component, ErasedComponent};
use comet_structs::Column;
use std::any::TypeId;

pub trait ComponentTuple {
    fn type_ids() -> Vec<TypeId>;
}

pub trait ComponentValueTuple {
    fn type_ids(&self) -> Vec<TypeId>;
    fn into_components(self) -> Vec<ErasedComponent>;
    fn write_components(self, columns: &mut [Column], column_indices: &[usize], row: usize);
    fn write_components_reserved(
        self,
        columns: &mut [Column],
        column_indices: &[usize],
        row: usize,
    );
}

impl ComponentTuple for () {
    fn type_ids() -> Vec<TypeId> {
        Vec::new()
    }
}

impl ComponentValueTuple for () {
    fn type_ids(&self) -> Vec<TypeId> {
        Vec::new()
    }

    fn into_components(self) -> Vec<ErasedComponent> {
        Vec::new()
    }

    fn write_components(self, _columns: &mut [Column], _column_indices: &[usize], _row: usize) {}

    fn write_components_reserved(
        self,
        _columns: &mut [Column],
        _column_indices: &[usize],
        _row: usize,
    ) {
    }
}

impl<C: Component> ComponentValueTuple for C {
    fn type_ids(&self) -> Vec<TypeId> {
        vec![C::type_id()]
    }

    fn into_components(self) -> Vec<ErasedComponent> {
        vec![ErasedComponent::new(self)]
    }

    fn write_components(self, columns: &mut [Column], column_indices: &[usize], _row: usize) {
        let col_idx = column_indices[0];
        unsafe {
            columns[col_idx].push_unchecked::<C>(self);
        }
    }

    fn write_components_reserved(
        self,
        columns: &mut [Column],
        column_indices: &[usize],
        _row: usize,
    ) {
        let col_idx = column_indices[0];
        unsafe {
            columns[col_idx].push_unchecked_reserved::<C>(self);
        }
    }
}

macro_rules! impl_component_tuple {
    ($($name:ident),+ $(,)?) => {
        impl<$($name: Component),+> ComponentTuple for ($($name,)+) {
            fn type_ids() -> Vec<TypeId> {
                vec![$($name::type_id()),+]
            }
        }

        impl<$($name: Component),+> ComponentValueTuple for ($($name,)+) {
            fn type_ids(&self) -> Vec<TypeId> {
                vec![$($name::type_id()),+]
            }

            #[allow(non_snake_case)]
            fn into_components(self) -> Vec<ErasedComponent> {
                let ($($name,)+) = self;
                vec![$(ErasedComponent::new($name)),+]
            }

            #[allow(non_snake_case, unused_assignments)]
            fn write_components(self, columns: &mut [Column], column_indices: &[usize], _row: usize) {
                let ($($name,)+) = self;
                let mut component_i = 0usize;
                $(
                    {
                        let col_idx = column_indices[component_i];
                        component_i += 1;
                        unsafe {
                            columns[col_idx].push_unchecked::<$name>($name);
                        }
                    }
                )+
            }

            #[allow(non_snake_case, unused_assignments)]
            fn write_components_reserved(
                self,
                columns: &mut [Column],
                column_indices: &[usize],
                _row: usize,
            ) {
                let ($($name,)+) = self;
                let mut component_i = 0usize;
                $(
                    {
                        let col_idx = column_indices[component_i];
                        component_i += 1;
                        unsafe {
                            columns[col_idx].push_unchecked_reserved::<$name>($name);
                        }
                    }
                )+
            }
        }
    };
}

impl_component_tuple!(A);
impl_component_tuple!(A, B);
impl_component_tuple!(A, B, C);
impl_component_tuple!(A, B, C, D);
impl_component_tuple!(A, B, C, D, E);
impl_component_tuple!(A, B, C, D, E, F);
impl_component_tuple!(A, B, C, D, E, F, G);
impl_component_tuple!(A, B, C, D, E, F, G, H);
