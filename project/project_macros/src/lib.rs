use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    self, punctuated::Punctuated,Field, Ident,
    Lit, Meta, Token,
};

#[proc_macro_derive(PlotteryParamsDefinition, attributes(value, range))]
pub fn plottery_params(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Failed to parse macro.");
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
            let field_type_name = get_field_type_name(field);
            let field_name = field.ident.as_ref().expect("Failed to get struct field name.").to_string();

            let mut default_value: proc_macro2::TokenStream = quote!(#field_type::default());
            let mut range: Option<(proc_macro2::TokenStream, proc_macro2::TokenStream)> = None;

            for attr in &field.attrs {
                match &attr.meta {
                    Meta::List(list) => {
                        // #[default(1.0)]a
                        if list.path.is_ident("value") {
                            let attribute_name = list.path.get_ident().expect("Failed to get attribute name").to_string();
                            let args = list
                                .parse_args_with(Punctuated::<Lit, Token![,]>::parse_terminated)
                                .unwrap_or_else(|_| panic!("Failed to parse attribute arguments for attribute '{}'", attribute_name));
                            let num_expected_args = 1;
                            if args.len() != num_expected_args {
                                panic!("Invalid number of arguments for attribute '{}'. Expected {}, found {}", attribute_name, num_expected_args, args.len());
                            }

                            let argument = {
                                let parsed_args = list
                                    .parse_args_with(Punctuated::<Lit, Token![,]>::parse_terminated)
                                    .unwrap_or_else(|_| panic!("Failed to parse attribute arguments for attribute '{}'", attribute_name));
                                parsed_args.first().expect("Invalid number of arguments.").clone()
                            };

                            default_value = quote!(#argument);
                        }

                        // #[range(0.0, 1.0)]
                        if list.path.is_ident("range") {
                            let attribute_name = list.path.get_ident().expect("Failed to get attribute name").to_string();
                            let args = list
                                .parse_args_with(Punctuated::<Lit, Token![,]>::parse_terminated)
                                .unwrap_or_else(|_| panic!("Failed to parse attribute arguments for attribute '{}'", attribute_name));
                            let num_expected_args = 2;
                            if args.len() != num_expected_args {
                                panic!("Invalid number of arguments for attribute '{}'. Expected {}, found {}", attribute_name, num_expected_args, args.len());
                            }

                            let arguments = list.parse_args_with(Punctuated::<Lit, Token![,]>::parse_terminated)
                                .expect("Failed to parse attribute argument");
                            let min = arguments.first().expect("Invalid number of arguments.");
                            let max = arguments.last().expect("Invalid number of arguments.");
                            range = Some((quote!(#min), quote!(#max)));
                        }
                    }
                    Meta::Path(_) => panic!("Invalid attribute. Expected list. e.g. #[value(1.0)]"),
                    Meta::NameValue(_) => panic!("Invalid attribute. Expected list. e.g. #[value(1.0)]"),
                }
            }

            match field_type_name.as_str() {
                "f32" => {
                    if let Some((min, max)) = range {
                        quote! {
                            ProjectParam::new(#field_name, ProjectParamValue::FloatRanged{val: #default_value, min: #min, max: #max}),
                        }
                    } else {
                        quote! {
                            ProjectParam::new(#field_name, ProjectParamValue::Float(#default_value)),
                        }
                    }
                },
                "i32" => {
                    if let Some((min, max)) = range {
                        quote! {
                            ProjectParam::new(#field_name, ProjectParamValue::IntRanged{val: #default_value, min: #min, max: #max}),
                        }
                    } else {
                        quote! {
                            ProjectParam::new(#field_name, ProjectParamValue::Int(#default_value)),
                        }
                    }
                },
                _ => panic!("Invalid field type: {}", field_type_name),
            }

            
        })
        .collect::<Vec<_>>()
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
                #field_name: params.get(stringify!(#field_name)).unwrap_or_else(|| panic!("Field '{}' is missing in params from stdin.", stringify!(#field_name))).value.#accessor_function().unwrap(),
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
        fn param_defaults_list() -> std::vec::Vec<ProjectParam> {
            vec![
                #(#parameter_vector_items)*
            ]
        }
    };

    // NEW FROM MAP
    let constructor_fields = get_constructor_fields_items(data);
    let new_from_map_impl = quote! {
        fn new_from_map(params: &std::collections::HashMap<String, ProjectParam>) -> Self {
            Self {
                #(#constructor_fields)*
            }
        }
    };

    // NEW FROM LIST (calls new_from_map internally)
    let new_from_list_impl = quote! {
        fn new_from_list(parms: std::vec::Vec<ProjectParam>) -> Self {
            let mut map = std::collections::HashMap::new();
            for param in parms {
                map.insert(param.name.clone(), param);
            }
            Self::new_from_map(&map)
        }
    };

    // NEW WITH DEFAULTS
    let new_with_defaults_impl = quote! {
        fn new_with_defaults() -> Self {
            Self::new_from_list(Self::param_defaults_list())
        }
    };

    // TRAIT IMPLEMENTATION
    let expanded = quote! {
        impl PlotteryParamsDefinition for #name {
            #get_params_impl
            #new_from_map_impl

            #new_from_list_impl
            #new_with_defaults_impl
        }
    };
    TokenStream::from(expanded)
}
