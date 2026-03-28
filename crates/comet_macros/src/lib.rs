use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, ImplItem, ItemImpl, Pat, Type, Visibility};

/// Attribute macro for module impl blocks.
///
/// Generates a `{TypeName}Ext` trait and a forwarding `impl {TypeName}Ext for ::comet_app::App`
/// for all `pub` methods with a self receiver (excluding `build`).
///
/// # Example
/// ```ignore
/// #[module]
/// impl AudioModule {
///     pub fn play_audio(&mut self, name: &str, looped: bool) { ... }
///     pub fn is_playing(&self, name: &str) -> bool { ... }
/// }
/// ```
/// Generates `AudioModuleExt` with those methods forwarded through `App::get_module_mut` / `App::get_module`.
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

        let is_mut = match sig.inputs.first() {
            Some(FnArg::Receiver(r)) => r.mutability.is_some(),
            _ => false,
        };

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

        trait_methods.push(quote! { #sig; });

        let accessor = if is_mut {
            quote! { self.get_module_mut::<#ty>() }
        } else {
            quote! { self.get_module::<#ty>() }
        };

        impl_methods.push(quote! {
            #sig {
                #accessor.#method_name(#(#param_names),*)
            }
        });
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
