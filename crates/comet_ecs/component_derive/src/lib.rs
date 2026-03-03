extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Component)]
pub fn component_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let fields = if let Data::Struct(data) = &input.data {
        match &data.fields {
            Fields::Named(fields) => fields.named.iter().collect::<Vec<_>>(),
            Fields::Unnamed(fields) => fields.unnamed.iter().collect::<Vec<_>>(),
            Fields::Unit => Vec::new(),
        }
    } else {
        panic!("Component derive macro only works on structs");
    };

    let field_comparisons = fields.iter().map(|field| {
        let field_name = &field.ident; // Name of the field
        quote! {
            self.#field_name == other.#field_name
        }
    });

    let clone_fields = fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! {
            #field_name: self.#field_name.clone()
        }
    });

    let default_fields = if let Data::Struct(data) = &input.data {
        match &data.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .map(|field| {
                    let field_name = &field.ident;
                    quote! { #field_name: Default::default() }
                })
                .collect::<Vec<_>>(),
            Fields::Unnamed(fields) => fields
                .unnamed
                .iter()
                .map(|_field| {
                    quote! { Default::default() }
                })
                .collect::<Vec<_>>(),
            Fields::Unit => Vec::new(),
        }
    } else {
        panic!("Default can only be derived for structs");
    };

    let debug_fields = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("Expected named fields");
        quote! {
            .field(stringify!(#field_name), &self.#field_name)
        }
    });

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
                Self {
                    #(#default_fields),*
                }
            }
        }

        impl Clone for #name {
            fn clone(&self) -> Self {
                Self {
                    #(#clone_fields),*
                }
            }
        }

        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!(#name))
                    #(#debug_fields)*
                    .finish()
            }
        }

        impl PartialEq for #name {
            fn eq(&self, other: &Self) -> bool {
                true #(&& #field_comparisons)*
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Tag)]
pub fn tag_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    if !matches!(input.data, Data::Struct(_)) {
        panic!("Tag derive macro only works on structs");
    }

    let expanded = quote! {
        const _: [(); 0] = [(); std::mem::size_of::<#name>()];

        impl Component for #name {
            fn type_id() -> std::any::TypeId {
                std::any::TypeId::of::<Self>()
            }

            fn type_name() -> String {
                std::any::type_name::<Self>().to_string()
            }
        }

        impl Tag for #name {}
    };

    TokenStream::from(expanded)
}
