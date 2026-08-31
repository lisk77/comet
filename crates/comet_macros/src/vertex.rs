use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, parse_quote, Attribute, Data, DeriveInput, Fields, LitInt, Meta};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Semantic {
    Position,
    Normal,
    Tangent,
    TexCoord(u32),
    Color(u32),
    JointIndices(u32),
    JointWeights(u32),
    Custom(u32),
}

impl Semantic {
    fn parse(attribute: &Attribute) -> syn::Result<Self> {
        let path = attribute.path();
        if path.is_ident("position") {
            require_path(attribute)?;
            return Ok(Self::Position);
        }
        if path.is_ident("normal") {
            require_path(attribute)?;
            return Ok(Self::Normal);
        }
        if path.is_ident("tangent") {
            require_path(attribute)?;
            return Ok(Self::Tangent);
        }

        let index = parse_index(attribute)?;
        if path.is_ident("tex_coord") {
            Ok(Self::TexCoord(index))
        } else if path.is_ident("color") {
            Ok(Self::Color(index))
        } else if path.is_ident("joint_indices") {
            Ok(Self::JointIndices(index))
        } else if path.is_ident("joint_weights") {
            Ok(Self::JointWeights(index))
        } else {
            Ok(Self::Custom(index))
        }
    }
}

impl ToTokens for Semantic {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let semantic = match self {
            Self::Position => quote!(VertexSemantic::Position),
            Self::Normal => quote!(VertexSemantic::Normal),
            Self::Tangent => quote!(VertexSemantic::Tangent),
            Self::TexCoord(index) => quote!(VertexSemantic::TexCoord(#index)),
            Self::Color(index) => quote!(VertexSemantic::Color(#index)),
            Self::JointIndices(index) => quote!(VertexSemantic::JointIndices(#index)),
            Self::JointWeights(index) => quote!(VertexSemantic::JointWeights(#index)),
            Self::Custom(index) => quote!(VertexSemantic::Custom(#index)),
        };
        tokens.extend(semantic);
    }
}

pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand(input) {
        Ok(output) => output.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            name,
            "Vertex can only be derived for structs",
        ));
    };
    let Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new_spanned(
            name,
            "Vertex requires named fields",
        ));
    };

    let mut seen = HashSet::new();
    let mut vertex_fields = Vec::with_capacity(fields.named.len());
    for field in &fields.named {
        let attributes = field
            .attrs
            .iter()
            .filter(|attribute| is_semantic_attribute(attribute))
            .collect::<Vec<_>>();
        if attributes.len() != 1 {
            return Err(syn::Error::new_spanned(
                field,
                "vertex field requires exactly one semantic attribute",
            ));
        }

        let semantic = Semantic::parse(attributes[0])?;
        if !seen.insert(semantic) {
            return Err(syn::Error::new_spanned(
                attributes[0],
                "duplicate vertex semantic",
            ));
        }
        vertex_fields.push((field.ident.as_ref().unwrap(), &field.ty, semantic));
    }

    let mut generics = input.generics.clone();
    for (_, field_type, _) in &vertex_fields {
        generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#field_type: VertexValue));
    }
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let mut offset = quote!(0usize);
    let mut attributes = Vec::with_capacity(vertex_fields.len());
    let mut encoders = Vec::with_capacity(vertex_fields.len());
    for (field_name, field_type, semantic) in &vertex_fields {
        attributes.push(quote! {
            VertexAttribute::new(
                #semantic,
                #offset,
                <#field_type as VertexValue>::FORMAT,
            )
        });
        encoders.push(quote! {
            VertexValue::encode(&self.#field_name, output);
        });
        offset = quote!(#offset + <#field_type as VertexValue>::SIZE);
    }
    let stride = offset;

    Ok(quote! {
        impl #impl_generics Vertex for #name #type_generics #where_clause {
            fn descriptor() -> VertexDescriptor {
                VertexDescriptor::new(
                    #stride,
                    vec![#(#attributes),*],
                )
            }

            fn encode(&self, output: &mut Vec<u8>) {
                #(#encoders)*
            }
        }
    })
}

fn is_semantic_attribute(attribute: &Attribute) -> bool {
    let path = attribute.path();
    path.is_ident("position")
        || path.is_ident("normal")
        || path.is_ident("tangent")
        || path.is_ident("tex_coord")
        || path.is_ident("color")
        || path.is_ident("joint_indices")
        || path.is_ident("joint_weights")
        || path.is_ident("custom")
}

fn require_path(attribute: &Attribute) -> syn::Result<()> {
    if matches!(attribute.meta, Meta::Path(_)) {
        Ok(())
    } else {
        Err(syn::Error::new_spanned(
            attribute,
            "this vertex semantic does not accept an index",
        ))
    }
}

fn parse_index(attribute: &Attribute) -> syn::Result<u32> {
    let Meta::List(_) = &attribute.meta else {
        return Err(syn::Error::new_spanned(
            attribute,
            "this vertex semantic requires one integer index",
        ));
    };
    let literal = attribute.parse_args::<LitInt>().map_err(|_| {
        syn::Error::new_spanned(attribute, "this vertex semantic requires one integer index")
    })?;
    literal
        .base10_parse::<u32>()
        .map_err(|_| syn::Error::new_spanned(literal, "vertex semantic index must be a valid u32"))
}
