extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

// This is the procedural macro for `derive(MyTrait)`
#[proc_macro_derive(Component)]
pub fn my_trait_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree (AST)
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the struct
    let name = &input.ident;
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

    let default_fields = if let Data::Struct(data) = &input.data {
        match &data.fields {
            Fields::Named(fields) => {
                // Generate code to assign each field a default value
                fields.named.iter().map(|field| {
                    let field_name = &field.ident;
                    quote! { #field_name: Default::default() }
                }).collect::<Vec<_>>()
            },
            Fields::Unnamed(fields) => {
                // Generate default values for tuple structs
                fields.unnamed.iter().map(|_field| {
                    quote! { Default::default() }
                }).collect::<Vec<_>>()
            },
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

    for field in &fields {
        if !is_copy_field(&field) {
            panic!("All fields in the struct must implement Copy");
        }
    }

    // Generate the implementation of MyTrait for the given struct
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
                // Construct the struct with default values
                Self {
                    #(#default_fields),*
                }
            }
        }

        impl Clone for #name {
            fn clone(&self) -> Self {
                Self {
                    ..*self
                }
            }
        }

        impl Copy for #name {}

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

    // Convert the generated code into a TokenStream and return it
    TokenStream::from(expanded)
}


fn is_copy_field(field: &syn::Field) -> bool {
    // Logic to check if the field type implements Copy (this is simplified)
    // You might need more sophisticated logic to check the actual type of the field.
    let field_type = &field.ty;
    // Implement a check for Copy trait for the field type if needed
    // Return true if it implements Copy; false otherwise
    true // Assuming it does, just for simplicity
}