use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, LitStr};

pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    if !input.generics.params.is_empty() {
        return syn::Error::new_spanned(name, "generic materials are not supported yet")
            .into_compile_error()
            .into();
    }
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let mut shader_path = None;

    for attribute in input
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("material"))
    {
        if let Err(error) = attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("shader") {
                if shader_path.is_some() {
                    return Err(meta.error("duplicate material shader option"));
                }
                shader_path = Some(meta.value()?.parse::<LitStr>()?);
                return Ok(());
            }
            Err(meta.error("unsupported material option"))
        }) {
            return error.into_compile_error().into();
        }
    }

    let Some(shader_path) = shader_path else {
        return syn::Error::new_spanned(
            name,
            "Material requires #[material(shader = \"res://path/to/shader.wgsl\")]",
        )
        .into_compile_error()
        .into();
    };
    if !shader_path.value().starts_with("res://") {
        return syn::Error::new_spanned(shader_path, "material shader path must start with res://")
            .into_compile_error()
            .into();
    }

    let Data::Struct(data) = &input.data else {
        return syn::Error::new_spanned(name, "Material can only be derived for structs")
            .into_compile_error()
            .into();
    };
    let Fields::Named(fields) = &data.fields else {
        return syn::Error::new_spanned(name, "Material requires named fields")
            .into_compile_error()
            .into();
    };

    let mut uniform_fields = Vec::new();
    let mut builders = Vec::new();
    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        let role_count = field
            .attrs
            .iter()
            .filter(|attribute| {
                attribute.path().is_ident("uniform")
                    || attribute.path().is_ident("texture")
                    || attribute.path().is_ident("sampler")
            })
            .count();
        if role_count > 1 {
            return syn::Error::new_spanned(
                field,
                "material field can only have one resource role",
            )
            .into_compile_error()
            .into();
        }
        if field
            .attrs
            .iter()
            .any(|attribute| attribute.path().is_ident("uniform"))
        {
            uniform_fields.push((field_name, field_type));
        }
        let builder_name = quote::format_ident!("with_{}", field_name);
        builders.push(quote! {
            pub fn #builder_name(mut self, value: #field_type) -> Self {
                self.#field_name = value;
                self
            }
        });
    }

    let uniform_descriptors = uniform_fields.iter().map(|(field_name, field_type)| {
        quote! {
            UniformDescriptor::new(
                stringify!(#field_name),
                <#field_type as ShaderData>::schema,
            )
        }
    });
    let uniform_encoders = uniform_fields.iter().map(|(field_name, _)| {
        quote! {
            encoder.write_uniform(stringify!(#field_name), &self.#field_name);
        }
    });

    quote! {
        impl #impl_generics Component for #name #type_generics #where_clause {
            fn register_needed_components(needs: &mut NeededComponents) {
                needs.need::<Mesh>();
            }

            fn register_query_targets(targets: &mut QueryTargets) {
                targets.register::<dyn Material>(
                    |value, output| unsafe {
                        let value =
                            (&*(value as *const Self)) as &dyn Material as *const dyn Material;
                        output.cast::<*const dyn Material>().write(value);
                    },
                    |value, output| unsafe {
                        let value =
                            (&mut *(value as *mut Self)) as &mut dyn Material as *mut dyn Material;
                        output.cast::<*mut dyn Material>().write(value);
                    },
                );
            }
        }

        impl #impl_generics Material for #name #type_generics #where_clause {
            fn descriptor(&self) -> &'static MaterialDescriptor {
                static UNIFORMS: &[UniformDescriptor] = &[
                    #(#uniform_descriptors),*
                ];
                static DESCRIPTOR: MaterialDescriptor = MaterialDescriptor::new(
                    stringify!(#name),
                    #shader_path,
                    UNIFORMS,
                );
                &DESCRIPTOR
            }

            fn encode(&self, encoder: &mut MaterialEncoder) {
                #(#uniform_encoders)*
            }
        }

        impl #impl_generics #name #type_generics #where_clause {
            #(#builders)*
        }
    }
    .into()
}
