use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, ImplItem, ItemImpl, Pat, ReturnType, Type, Visibility};

#[proc_macro_attribute]
pub fn module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);

    let ty = &input.self_ty;
    let type_ident = match ty.as_ref() {
        Type::Path(p) => match p.path.segments.last() {
            Some(seg) => seg.ident.clone(),
            None => {
                return syn::Error::new_spanned(ty, "#[module]: expected named type")
                    .to_compile_error()
                    .into()
            }
        },
        _ => {
            return syn::Error::new_spanned(ty, "#[module]: expected path type")
                .to_compile_error()
                .into()
        }
    };

    let trait_name = format_ident!("{}Ext", type_ident);

    let qualifying: Vec<_> = input
        .items
        .iter()
        .filter_map(|item| {
            let ImplItem::Fn(method) = item else {
                return None;
            };
            if !matches!(method.vis, Visibility::Public(_)) {
                return None;
            }
            if method.sig.ident == "build" {
                return None;
            }
            let has_self = method
                .sig
                .inputs
                .first()
                .map(|a| matches!(a, FnArg::Receiver(_)))
                .unwrap_or(false);
            if !has_self {
                return None;
            }
            Some(method)
        })
        .collect();

    let mut trait_methods: Vec<TokenStream2> = Vec::new();
    let mut impl_methods: Vec<TokenStream2> = Vec::new();

    for method in qualifying {
        let sig = &method.sig;
        let method_name = &sig.ident;

        let receiver = match sig.inputs.first() {
            Some(FnArg::Receiver(r)) => r,
            _ => continue,
        };

        // Builder method: takes &mut self and returns &mut Self
        let returns_mut_self = match &sig.output {
            ReturnType::Type(_, t) => match t.as_ref() {
                Type::Reference(r) if r.mutability.is_some() => {
                    matches!(r.elem.as_ref(), Type::Path(p) if p.path.is_ident("Self"))
                }
                _ => false,
            },
            _ => false,
        };
        let is_builder = receiver.reference.is_some()
            && receiver.mutability.is_some()
            && returns_mut_self;

        let param_names: Vec<_> = sig
            .inputs
            .iter()
            .skip(1)
            .filter_map(|arg| {
                if let FnArg::Typed(pt) = arg {
                    if let Pat::Ident(pi) = pt.pat.as_ref() {
                        return Some(pi.ident.clone());
                    }
                }
                None
            })
            .collect();

        // Rebuild param list without the self receiver for trait/impl signatures
        let params: Vec<_> = sig.inputs.iter().skip(1).collect();
        let generics = &sig.generics;
        let where_clause = &sig.generics.where_clause;

        let type_params: Vec<_> = sig.generics.params.iter().filter_map(|p| {
            if let syn::GenericParam::Type(tp) = p { Some(&tp.ident) } else { None }
        }).collect();
        let turbofish = if type_params.is_empty() {
            quote! { #method_name }
        } else {
            quote! { #method_name::<#(#type_params),*> }
        };

        if is_builder {
            trait_methods.push(quote! {
                fn #method_name #generics (self, #(#params),*) -> Self #where_clause;
            });
            impl_methods.push(quote! {
                fn #method_name #generics (mut self, #(#params),*) -> Self #where_clause {
                    self.get_module_mut::<#ty>().#turbofish(#(#param_names),*);
                    self
                }
            });
        } else {
            let is_mut = receiver.mutability.is_some();
            let accessor = if is_mut {
                quote! { self.get_module_mut::<#ty>() }
            } else {
                quote! { self.get_module::<#ty>() }
            };
            let output = &sig.output;
            let self_ref = if is_mut {
                quote! { &mut self }
            } else {
                quote! { &self }
            };
            trait_methods.push(quote! {
                fn #method_name #generics (#self_ref, #(#params),*) #output #where_clause;
            });
            impl_methods.push(quote! {
                fn #method_name #generics (#self_ref, #(#params),*) #output #where_clause {
                    #accessor.#turbofish(#(#param_names),*)
                }
            });
        }
    }

    TokenStream::from(quote! {
        #input

        pub trait #trait_name {
            #(#trait_methods)*
        }

        impl #trait_name for ::comet_app::App {
            #(#impl_methods)*
        }
    })
}
