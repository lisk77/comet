extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    DeriveInput, Expr, Token, Type,
};

struct RequiredComponent {
    component_type: Type,
    factory: Option<Expr>,
}

impl Parse for RequiredComponent {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let component_type = input.parse()?;
        let factory = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Self {
            component_type,
            factory,
        })
    }
}

#[proc_macro_derive(Component, attributes(require))]
pub fn component_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let required_components = match input
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("require"))
        .map(|attribute| {
            attribute.parse_args_with(Punctuated::<RequiredComponent, Token![,]>::parse_terminated)
        })
        .collect::<syn::Result<Vec<_>>>()
    {
        Ok(required_components) => required_components,
        Err(error) => return error.into_compile_error().into(),
    }
    .into_iter()
    .flatten()
    .map(|required| {
        let component_type = required.component_type;
        if let Some(factory) = required.factory {
            quote! {
                requirements.require_with::<#component_type>(#factory);
            }
        } else {
            quote! {
                requirements.require::<#component_type>();
            }
        }
    });

    quote! {
        impl #impl_generics Component for #name #type_generics #where_clause {
            fn register_required_components(requirements: &mut RequiredComponents) {
                #(#required_components)*
            }
        }
    }
    .into()
}
