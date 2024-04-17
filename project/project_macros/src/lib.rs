use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{self, Field, Ident};

#[proc_macro_derive(PlotteryParamsDefinition, attributes(default))]
pub fn plottery_params(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    plottery_params_impl(&ast)
}

fn perform_sanity_checks(data: &syn::DataStruct) {
    for field in &data.fields {
        match field.ty {
            syn::Type::Array(_) => panic!("Parameter fields cannot be of type array."),
            syn::Type::BareFn(_) => panic!("Parameter fields cannot be of type function."),
            syn::Type::Group(_) => panic!("Parameter fields cannot be of type group."),
            syn::Type::ImplTrait(_) => panic!("Parameter fields cannot be of type impl trait."),
            syn::Type::Infer(_) => panic!("Parameter fields must have a specified type."),
            syn::Type::Macro(_) => panic!("Parameter fields cannot be of type macro."),
            syn::Type::Never(_) => panic!("Parameter fields cannot be of type never."),
            syn::Type::Paren(_) => panic!("Parameter fields cannot be of type paren."),
            syn::Type::Path(_) => {}
            syn::Type::Ptr(_) => panic!("Parameter fields cannot be of type pointer."),
            syn::Type::Reference(_) => panic!("Parameter fields cannot be of type reference."),
            syn::Type::Slice(_) => panic!("Parameter fields cannot be of type slice."),
            syn::Type::TraitObject(_) => panic!("Parameter fields cannot be of type trait object."),
            syn::Type::Tuple(_) => panic!("Parameter fields cannot be of type tuple."),
            syn::Type::Verbatim(_) => panic!("Parameter fields cannot be of type verbatim."),
            _ => panic!("Parameter fields must have a specified type."),
        }
    }
}

fn get_field_type_name(field: &Field) -> String {
    match &field.ty {
        syn::Type::Path(path) => path
            .path
            .segments
            .last()
            .expect("Invalid field type.")
            .ident
            .to_string(),
        _ => panic!("Parameter field type is invalid."),
    }
}

fn get_parameters_vector_items(data: &syn::DataStruct) -> Vec<proc_macro2::TokenStream> {
    data.fields
        .iter()
        .map(|field| {
            let field_type = &field.ty;
            let default_value = quote!(#field_type::default());
            let default_value = field
                .attrs
                .iter()
                .find_map(|attr| {
                    if attr.path().is_ident("default") {
                        let value = &attr
                            .meta
                            .require_name_value()
                            .expect("default attribute must have a value")
                            .value;
                        Some(quote!(#value))
                    } else {
                        None
                    }
                })
                .unwrap_or(default_value);

            let field_name = field.ident.as_ref().unwrap().to_string();
            quote! {
                ProjectParam::new(#field_name, ProjectParamValue::Float(#default_value)),
            }
        })
        .collect()
}

fn get_constructor_fields_items(data: &syn::DataStruct) -> Vec<proc_macro2::TokenStream> {
    data
        .fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().expect("Failed to access field.");
            let field_type_name = get_field_type_name(field);
            let accessor_function = Ident::new(&format!("get_{}", field_type_name), Span::call_site());
            quote! {
                #field_name: params.get(stringify!(#field_name)).unwrap().value.#accessor_function().unwrap(),
            }
        })
        .collect()
}

fn plottery_params_impl(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let name = &ast.ident;
    let data = match &ast.data {
        syn::Data::Struct(data) => data,
        syn::Data::Enum(_) => {
            panic!("PlotteryProjectParams cannot be derived for enums. Use a struct instead.")
        }
        syn::Data::Union(_) => {
            panic!("PlotteryProjectParams cannot be derived for unions. Use a struct instead.")
        }
    };

    // SANITY CHECKS
    perform_sanity_checks(data);

    // GET PARAMS
    let parameter_vector_items = get_parameters_vector_items(data);
    let get_params_impl = quote! {
        fn get_params(&self) -> Vec<ProjectParam> {
            vec![
                #(#parameter_vector_items)*
            ]
        }
    };

    // NEW FROM LIST
    let constructor_fields = get_constructor_fields_items(data);
    let new_from_list_impl = quote! {
        fn new_from_list(params: &std::collections::HashMap<String, ProjectParam>) -> Self {
            Self {
                #(#constructor_fields)*
            }
        }
    };

    // TRAIT IMPLEMENTATION
    let expanded = quote! {
        impl PlotteryParamsDefinition for #name {
            #get_params_impl
            #new_from_list_impl
        }
    };
    TokenStream::from(expanded)
}
