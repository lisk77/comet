use crate::{App, Module};

pub trait ModuleTuple {
    fn add_to(self, app: App) -> App;
}

impl<M: Module> ModuleTuple for M {
    fn add_to(self, app: App) -> App {
        app.with_module(self)
    }
}

macro_rules! impl_module_tuple {
    ($($T:ident),+) => {
        #[allow(non_snake_case)]
        impl<$($T: Module),+> ModuleTuple for ($($T,)+) {
            fn add_to(self, app: App) -> App {
                let ($($T,)+) = self;
                app$(.with_module($T))+
            }
        }
    };
}

impl_module_tuple!(A, B);
impl_module_tuple!(A, B, C);
impl_module_tuple!(A, B, C, D);
impl_module_tuple!(A, B, C, D, E);
impl_module_tuple!(A, B, C, D, E, F);
impl_module_tuple!(A, B, C, D, E, F, G);
impl_module_tuple!(A, B, C, D, E, F, G, H);
