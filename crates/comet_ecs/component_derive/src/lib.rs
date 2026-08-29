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

#[proc_macro_derive(Component, attributes(require, needs, query_as))]
pub fn component_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let query_traits = match input
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("query_as"))
        .map(|attribute| attribute.parse_args_with(Punctuated::<Type, Token![,]>::parse_terminated))
        .collect::<syn::Result<Vec<_>>>()
    {
        Ok(query_traits) => query_traits.into_iter().flatten().collect::<Vec<_>>(),
        Err(error) => return error.into_compile_error().into(),
    };
    let query_target_registrations = query_traits.iter().map(|query_trait| {
        quote! {
            targets.register::<dyn #query_trait>(
                |value, output| unsafe {
                    let value =
                        (&*(value as *const Self)) as &dyn #query_trait as *const dyn #query_trait;
                    output.cast::<*const dyn #query_trait>().write(value);
                },
                |value, output| unsafe {
                    let value =
                        (&mut *(value as *mut Self)) as &mut dyn #query_trait as *mut dyn #query_trait;
                    output.cast::<*mut dyn #query_trait>().write(value);
                },
            );
        }
    });
    let register_query_targets = (!query_traits.is_empty()).then(|| {
        quote! {
            fn register_query_targets(targets: &mut QueryTargets) {
                #(#query_target_registrations)*
            }
        }
    });

    let needed_components = match input
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("needs"))
        .map(|attribute| attribute.parse_args_with(Punctuated::<Type, Token![,]>::parse_terminated))
        .collect::<syn::Result<Vec<_>>>()
    {
        Ok(needed_components) => needed_components.into_iter().flatten().collect::<Vec<_>>(),
        Err(error) => return error.into_compile_error().into(),
    };
    let register_needed_components = (!needed_components.is_empty()).then(|| {
        quote! {
            fn register_needed_components(needs: &mut NeededComponents) {
                #(needs.need::<#needed_components>();)*
            }
        }
    });

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

            #register_needed_components
            #register_query_targets
        }
    }
    .into()
}
