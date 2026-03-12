extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Index};

#[proc_macro_derive(Component)]
pub fn component_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let (default_body, clone_body, debug_body, eq_body) = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let names = fields
                    .named
                    .iter()
                    .map(|field| field.ident.clone().expect("named fields expected"))
                    .collect::<Vec<_>>();

                let default_body = quote! {
                    Self {
                        #(#names: Default::default()),*
                    }
                };

                let clone_body = quote! {
                    Self {
                        #(#names: self.#names.clone()),*
                    }
                };

                let debug_body = quote! {
                    let mut ds = f.debug_struct(stringify!(#name));
                    #(ds.field(stringify!(#names), &self.#names);)*
                    ds.finish()
                };

                let eq_body = quote! {
                    true #(&& self.#names == other.#names)*
                };

                (default_body, clone_body, debug_body, eq_body)
            }
            Fields::Unnamed(fields) => {
                let indices = (0..fields.unnamed.len())
                    .map(Index::from)
                    .collect::<Vec<_>>();

                let default_body = quote! {
                    Self(
                        #({ let _ = #indices; Default::default() }),*
                    )
                };

                let clone_body = quote! {
                    Self(
                        #(self.#indices.clone()),*
                    )
                };

                let debug_body = quote! {
                    let mut dt = f.debug_tuple(stringify!(#name));
                    #(dt.field(&self.#indices);)*
                    dt.finish()
                };

                let eq_body = quote! {
                    true #(&& self.#indices == other.#indices)*
                };

                (default_body, clone_body, debug_body, eq_body)
            }
            Fields::Unit => {
                let default_body = quote! { Self };
                let clone_body = quote! { Self };
                let debug_body = quote! { f.write_str(stringify!(#name)) };
                let eq_body = quote! { true };
                (default_body, clone_body, debug_body, eq_body)
            }
        },
        _ => panic!("Component derive macro only works on structs"),
    };

    let expanded = quote! {
        impl Component for #name {
            fn new() -> Self {
                Default::default()
            }

            fn type_id() -> std::any::TypeId {
                std::any::TypeId::of::<Self>()
            }

            fn type_name() -> String {
                std::any::type_name::<Self>().to_string()
            }
        }

        impl Default for #name {
            fn default() -> Self {
                #default_body
            }
        }

        impl Clone for #name {
            fn clone(&self) -> Self {
                #clone_body
            }
        }

        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                #debug_body
            }
        }

        impl PartialEq for #name {
            fn eq(&self, other: &Self) -> bool {
                #eq_body
            }
        }
    };

    TokenStream::from(expanded)
}
