extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
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
        Data::Enum(data) => {
            let variants = &data.variants;

            let first = variants.first().expect("Component enum must have at least one variant to derive Default");
            let first_ident = &first.ident;
            let default_body = match &first.fields {
                Fields::Named(fields) => {
                    let names = fields.named.iter().map(|f| f.ident.clone().unwrap()).collect::<Vec<_>>();
                    quote! { Self::#first_ident { #(#names: Default::default()),* } }
                }
                Fields::Unnamed(fields) => {
                    let defaults = fields.unnamed.iter().map(|_| quote! { Default::default() }).collect::<Vec<_>>();
                    quote! { Self::#first_ident(#(#defaults),*) }
                }
                Fields::Unit => quote! { Self::#first_ident },
            };

            let clone_arms = variants.iter().map(|v| {
                let vident = &v.ident;
                match &v.fields {
                    Fields::Named(fields) => {
                        let names = fields.named.iter().map(|f| f.ident.clone().unwrap()).collect::<Vec<_>>();
                        quote! { Self::#vident { #(#names),* } => Self::#vident { #(#names: #names.clone()),* } }
                    }
                    Fields::Unnamed(fields) => {
                        let bindings = (0..fields.unnamed.len()).map(|i| format_ident!("_f{}", i)).collect::<Vec<_>>();
                        quote! { Self::#vident(#(#bindings),*) => Self::#vident(#(#bindings.clone()),*) }
                    }
                    Fields::Unit => quote! { Self::#vident => Self::#vident },
                }
            });
            let clone_body = quote! { match self { #(#clone_arms),* } };

            let debug_arms = variants.iter().map(|v| {
                let vident = &v.ident;
                let vname = vident.to_string();
                match &v.fields {
                    Fields::Named(fields) => {
                        let names = fields.named.iter().map(|f| f.ident.clone().unwrap()).collect::<Vec<_>>();
                        let name_strs = names.iter().map(|n| n.to_string()).collect::<Vec<_>>();
                        quote! {
                            Self::#vident { #(#names),* } => {
                                let mut ds = f.debug_struct(#vname);
                                #(ds.field(#name_strs, #names);)*
                                ds.finish()
                            }
                        }
                    }
                    Fields::Unnamed(fields) => {
                        let bindings = (0..fields.unnamed.len()).map(|i| format_ident!("_f{}", i)).collect::<Vec<_>>();
                        quote! {
                            Self::#vident(#(#bindings),*) => {
                                let mut dt = f.debug_tuple(#vname);
                                #(dt.field(#bindings);)*
                                dt.finish()
                            }
                        }
                    }
                    Fields::Unit => quote! { Self::#vident => f.write_str(#vname) },
                }
            });
            let debug_body = quote! { match self { #(#debug_arms),* } };

            let eq_arms = variants.iter().map(|v| {
                let vident = &v.ident;
                match &v.fields {
                    Fields::Named(fields) => {
                        let names = fields.named.iter().map(|f| f.ident.clone().unwrap()).collect::<Vec<_>>();
                        let s_names = names.iter().map(|n| format_ident!("s_{}", n)).collect::<Vec<_>>();
                        let o_names = names.iter().map(|n| format_ident!("o_{}", n)).collect::<Vec<_>>();
                        quote! {
                            (Self::#vident { #(#names: #s_names),* }, Self::#vident { #(#names: #o_names),* }) => {
                                true #(&& #s_names == #o_names)*
                            }
                        }
                    }
                    Fields::Unnamed(fields) => {
                        let n = fields.unnamed.len();
                        let s_bindings = (0..n).map(|i| format_ident!("s_{}", i)).collect::<Vec<_>>();
                        let o_bindings = (0..n).map(|i| format_ident!("o_{}", i)).collect::<Vec<_>>();
                        quote! {
                            (Self::#vident(#(#s_bindings),*), Self::#vident(#(#o_bindings),*)) => {
                                true #(&& #s_bindings == #o_bindings)*
                            }
                        }
                    }
                    Fields::Unit => quote! { (Self::#vident, Self::#vident) => true },
                }
            });
            let eq_body = quote! { match (self, other) { #(#eq_arms,)* _ => false } };

            (default_body, clone_body, debug_body, eq_body)
        }
        _ => panic!("Component derive macro only works on structs or enums"),
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
