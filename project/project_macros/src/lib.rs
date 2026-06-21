use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    self, punctuated::Punctuated, Expr, Field, GenericArgument, Ident, Meta, PathArguments, Token,
};

#[proc_macro_derive(PlotteryParams, attributes(value, range))]
pub fn plottery_params(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Failed to parse macro.");
    plottery_params_impl(&ast)
}

#[derive(Clone)]
enum ParsedFieldType {
    Leaf {
        type_name: String,
        ty: syn::Type,
    },
    Struct {
        ty: syn::Type,
    },
    OptionLeaf {
        inner_type_name: String,
        inner_ty: syn::Type,
    },
    OptionStruct {
        inner_ty: syn::Type,
    },
    VecLeaf {
        inner_type_name: String,
        inner_ty: syn::Type,
    },
    VecStruct {
        inner_ty: syn::Type,
    },
}

fn perform_sanity_checks(data: &syn::DataStruct) {
    for field in &data.fields {
        match &field.ty {
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
            syn::Type::TraitObject(_) => {
                panic!("Parameter fields cannot be of type trait object.")
            }
            syn::Type::Tuple(_) => panic!("Parameter fields cannot be of type tuple."),
            syn::Type::Verbatim(_) => panic!("Parameter fields cannot be of type verbatim."),
            _ => panic!("Parameter fields must have a specified type."),
        }
    }
}

fn get_type_name_from_type(ty: &syn::Type) -> String {
    match ty {
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

fn ensure_non_generic_type_path(ty: &syn::Type) {
    match ty {
        syn::Type::Path(path) => {
            for segment in &path.path.segments {
                if !matches!(segment.arguments, PathArguments::None) {
                    panic!(
                        "Generic/wrapper field types are not supported for PlotteryParams (except Option<T> and Vec<T>)."
                    );
                }
            }
        }
        _ => panic!("Parameter field type is invalid."),
    }
}

fn parse_single_generic_type(type_name: &str, args: &PathArguments) -> syn::Type {
    let args = match args {
        PathArguments::AngleBracketed(args) => args,
        _ => panic!("{} fields must be of the form {}<T>.", type_name, type_name),
    };

    if args.args.len() != 1 {
        panic!("{} fields must be of the form {}<T>.", type_name, type_name);
    }

    match args.args.first().expect("Missing type argument") {
        GenericArgument::Type(ty) => ty.clone(),
        _ => panic!("{} fields must be of the form {}<T>.", type_name, type_name),
    }
}

fn parse_field_type(field: &Field) -> ParsedFieldType {
    let field_type = field.ty.clone();

    let path = match &field_type {
        syn::Type::Path(path) => path,
        _ => panic!("Parameter field type is invalid."),
    };

    let last_segment = path.path.segments.last().expect("Invalid field type path.");

    if last_segment.ident == "Option" {
        let inner_ty = parse_single_generic_type("Option", &last_segment.arguments);
        let inner_name = get_type_name_from_type(&inner_ty);

        if inner_name == "Option" {
            panic!("Nested Option<Option<T>> fields are not supported for PlotteryParams.");
        }
        if inner_name == "Vec" {
            panic!("Option<Vec<T>> fields are not supported for PlotteryParams.");
        }

        if is_supported_leaf(inner_name.as_str()) {
            ParsedFieldType::OptionLeaf {
                inner_type_name: inner_name,
                inner_ty,
            }
        } else {
            ensure_non_generic_type_path(&inner_ty);
            ParsedFieldType::OptionStruct { inner_ty }
        }
    } else if last_segment.ident == "Vec" {
        let inner_ty = parse_single_generic_type("Vec", &last_segment.arguments);
        let inner_name = get_type_name_from_type(&inner_ty);

        if inner_name == "Option" {
            panic!("Vec<Option<T>> fields are not supported for PlotteryParams.");
        }
        if inner_name == "Vec" {
            panic!("Nested Vec<Vec<T>> fields are not supported for PlotteryParams.");
        }

        if is_supported_leaf(inner_name.as_str()) {
            ParsedFieldType::VecLeaf {
                inner_type_name: inner_name,
                inner_ty,
            }
        } else {
            ensure_non_generic_type_path(&inner_ty);
            ParsedFieldType::VecStruct { inner_ty }
        }
    } else {
        ensure_non_generic_type_path(&field_type);

        let field_type_name = get_type_name_from_type(&field_type);
        if is_supported_leaf(field_type_name.as_str()) {
            ParsedFieldType::Leaf {
                type_name: field_type_name,
                ty: field_type,
            }
        } else {
            ParsedFieldType::Struct { ty: field_type }
        }
    }
}

fn is_supported_leaf(field_type_name: &str) -> bool {
    matches!(
        field_type_name,
        "f32" | "i32" | "bool" | "Graph2d" | "Curve2DNorm" | "Curve2D"
    )
}

fn parse_field_attributes(
    field: &Field,
    default_type: proc_macro2::TokenStream,
) -> (
    proc_macro2::TokenStream,
    Option<(proc_macro2::TokenStream, proc_macro2::TokenStream)>,
    bool,
    bool,
) {
    let mut default_value: proc_macro2::TokenStream = quote!(#default_type::default());
    let mut range: Option<(proc_macro2::TokenStream, proc_macro2::TokenStream)> = None;
    let mut has_value = false;
    let mut has_range = false;

    for attr in &field.attrs {
        match &attr.meta {
            Meta::List(list) => {
                if list.path.is_ident("value") {
                    has_value = true;
                    let attribute_name = list
                        .path
                        .get_ident()
                        .expect("Failed to get attribute name")
                        .to_string();
                    let args = list
                        .parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)
                        .unwrap_or_else(|_| {
                            panic!(
                                "Failed to parse attribute arguments for attribute '{}'",
                                attribute_name
                            )
                        });
                    let num_expected_args = 1;
                    if args.len() != num_expected_args {
                        panic!(
                            "Invalid number of arguments for attribute '{}'. Expected {}, found {}",
                            attribute_name,
                            num_expected_args,
                            args.len()
                        );
                    }

                    let argument = args.first().expect("Invalid number of arguments.").clone();
                    default_value = quote!(#argument);
                } else if list.path.is_ident("range") {
                    has_range = true;
                    let attribute_name = list
                        .path
                        .get_ident()
                        .expect("Failed to get attribute name")
                        .to_string();
                    let args = list
                        .parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)
                        .unwrap_or_else(|_| {
                            panic!(
                                "Failed to parse attribute arguments for attribute '{}'",
                                attribute_name
                            )
                        });
                    let num_expected_args = 2;
                    if args.len() != num_expected_args {
                        panic!(
                            "Invalid number of arguments for attribute '{}'. Expected {}, found {}",
                            attribute_name,
                            num_expected_args,
                            args.len()
                        );
                    }

                    let min = args.first().expect("Invalid number of arguments.");
                    let max = args.last().expect("Invalid number of arguments.");
                    range = Some((quote!(#min), quote!(#max)));
                } else {
                    panic!(
                        "Unknown attribute '{}'. Only #[value(...)] and #[range(...)] are supported for PlotteryParams fields.",
                        list.path.to_token_stream()
                    );
                }
            }
            Meta::Path(path) => {
                panic!(
                    "Unknown attribute '{}'. Expected #[value(...)] or #[range(...)].",
                    path.to_token_stream()
                )
            }
            Meta::NameValue(name_value) => {
                panic!(
                    "Unknown attribute '{}'. Expected #[value(...)] or #[range(...)].",
                    name_value.path.to_token_stream()
                )
            }
        }
    }

    (default_value, range, has_value, has_range)
}

fn make_leaf_value_tokens(
    field_type_name: &str,
    default_value: proc_macro2::TokenStream,
    range: Option<(proc_macro2::TokenStream, proc_macro2::TokenStream)>,
) -> proc_macro2::TokenStream {
    match field_type_name {
        "f32" => {
            if let Some((min, max)) = range {
                quote! { ProjectParamValue::FloatRanged{val: #default_value, min: #min, max: #max} }
            } else {
                quote! { ProjectParamValue::Float(#default_value) }
            }
        }
        "i32" => {
            if let Some((min, max)) = range {
                quote! { ProjectParamValue::IntRanged{val: #default_value, min: #min, max: #max} }
            } else {
                quote! { ProjectParamValue::Int(#default_value) }
            }
        }
        "bool" => quote! { ProjectParamValue::Bool(#default_value) },
        "Graph2d" => quote! { ProjectParamValue::Graph2d(#default_value) },
        "Curve2DNorm" => quote! { ProjectParamValue::Curve2DNorm(#default_value) },
        "Curve2D" => quote! { ProjectParamValue::Curve2D(#default_value) },
        _ => panic!(
            "Invalid field type '{}': expected supported leaf or struct deriving PlotteryParams",
            field_type_name
        ),
    }
}

fn get_parameters_vector_items(data: &syn::DataStruct) -> Vec<proc_macro2::TokenStream> {
    data.fields
        .iter()
        .map(|field| {
            let parsed_type = parse_field_type(field);
            let field_name = field
                .ident
                .as_ref()
                .expect("Failed to get struct field name.")
                .to_string();

            match parsed_type {
                ParsedFieldType::Leaf { type_name, ty } => {
                    let (default_value, range, _, _) = parse_field_attributes(field, quote!(#ty));
                    let leaf_value = make_leaf_value_tokens(type_name.as_str(), default_value, range);
                    quote! {
                        ProjectParam::new(#field_name, #leaf_value),
                    }
                }
                ParsedFieldType::Struct { ty } => {
                    let (_, _, has_value, has_range) = parse_field_attributes(field, quote!(#ty));
                    if has_value || has_range {
                        panic!(
                            "Attributes #[value(...)] and #[range(...)] are only allowed on leaf parameter fields. Field '{}' must define defaults in its nested PlotteryParams type.",
                            field_name
                        );
                    }

                    quote! {
                        ProjectParam::new(
                            #field_name,
                            ProjectParamValue::Struct(ProjectParamStruct::new(#ty::param_defaults_list())),
                        ),
                    }
                }
                ParsedFieldType::OptionLeaf {
                    inner_type_name,
                    inner_ty,
                } => {
                    let (default_value, range, _, _) =
                        parse_field_attributes(field, quote!(#inner_ty));
                    let inner_value =
                        make_leaf_value_tokens(inner_type_name.as_str(), default_value, range);

                    quote! {
                        ProjectParam::new(
                            #field_name,
                            ProjectParamValue::Optional(ProjectParamOptional::new(false, #inner_value)),
                        ),
                    }
                }
                ParsedFieldType::OptionStruct { inner_ty } => {
                    let (_, _, has_value, has_range) =
                        parse_field_attributes(field, quote!(#inner_ty));
                    if has_value || has_range {
                        panic!(
                            "Attributes #[value(...)] and #[range(...)] are only allowed on leaf parameter fields. Field '{}' must define defaults in its nested PlotteryParams type.",
                            field_name
                        );
                    }

                    quote! {
                        ProjectParam::new(
                            #field_name,
                            ProjectParamValue::Optional(ProjectParamOptional::new(
                                false,
                                ProjectParamValue::Struct(ProjectParamStruct::new(#inner_ty::param_defaults_list())),
                            )),
                        ),
                    }
                }
                ParsedFieldType::VecLeaf {
                    inner_type_name,
                    inner_ty,
                } => {
                    let (default_value, range, has_value, has_range) =
                        parse_field_attributes(field, quote!(#inner_ty));
                    if has_value || has_range {
                        panic!(
                            "Attributes #[value(...)] and #[range(...)] are not supported on Vec<T> fields. Vec fields always default to an empty vector and use T::default() as item prototype. Field '{}'.",
                            field_name
                        );
                    }

                    let inner_value =
                        make_leaf_value_tokens(inner_type_name.as_str(), default_value, range);

                    quote! {
                        ProjectParam::new(
                            #field_name,
                            ProjectParamValue::Vec(ProjectParamVec::new(#inner_value, vec![])),
                        ),
                    }
                }
                ParsedFieldType::VecStruct { inner_ty } => {
                    let (_, _, has_value, has_range) =
                        parse_field_attributes(field, quote!(#inner_ty));
                    if has_value || has_range {
                        panic!(
                            "Attributes #[value(...)] and #[range(...)] are not supported on Vec<T> fields. Vec fields always default to an empty vector and use T::default() as item prototype. Field '{}'.",
                            field_name
                        );
                    }

                    quote! {
                        ProjectParam::new(
                            #field_name,
                            ProjectParamValue::Vec(ProjectParamVec::new(
                                ProjectParamValue::Struct(ProjectParamStruct::new(#inner_ty::param_defaults_list())),
                                vec![],
                            )),
                        ),
                    }
                }
            }
        })
        .collect::<Vec<_>>()
}

fn make_leaf_constructor_expr(
    field_type_name: &str,
    field_name: &Ident,
    value_expr: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    match field_type_name {
        "Graph2d" => quote! {
            match #value_expr {
                ProjectParamValue::Graph2d(g) => g.clone(),
                _ => panic!("Expected Graph2d for field '{}'", stringify!(#field_name)),
            }
        },
        "Curve2DNorm" => quote! {
            match #value_expr {
                ProjectParamValue::Curve2DNorm(c) => c.clone(),
                _ => panic!("Expected Curve2DNorm for field '{}'", stringify!(#field_name)),
            }
        },
        "Curve2D" => quote! {
            match #value_expr {
                ProjectParamValue::Curve2D(c) => c.clone(),
                _ => panic!("Expected Curve2D for field '{}'", stringify!(#field_name)),
            }
        },
        _ => {
            let accessor_function =
                Ident::new(&format!("get_{}", field_type_name), Span::call_site());
            quote! {
                (#value_expr).#accessor_function().unwrap()
            }
        }
    }
}

fn get_constructor_fields_items(data: &syn::DataStruct) -> Vec<proc_macro2::TokenStream> {
    data.fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().expect("Failed to access field.");
            let value_expr = quote! {
                &params.get(stringify!(#field_name))
                    .unwrap_or_else(|| panic!("Field '{}' is missing in params from stdin.", stringify!(#field_name)))
                    .value
            };

            match parse_field_type(field) {
                ParsedFieldType::Leaf { type_name, .. } => {
                    let constructor_expr =
                        make_leaf_constructor_expr(type_name.as_str(), field_name, value_expr);
                    quote! {
                        #field_name: #constructor_expr,
                    }
                }
                ParsedFieldType::Struct { ty } => {
                    quote! {
                        #field_name: match #value_expr {
                            ProjectParamValue::Struct(s) => #ty::new_from_list(s.fields.clone()),
                            _ => panic!("Expected struct for field '{}'", stringify!(#field_name)),
                        },
                    }
                }
                ParsedFieldType::OptionLeaf {
                    inner_type_name, ..
                } => {
                    let inner_value_expr = quote! { inner };
                    let inner_constructor = make_leaf_constructor_expr(
                        inner_type_name.as_str(),
                        field_name,
                        inner_value_expr,
                    );
                    quote! {
                        #field_name: match #value_expr {
                            ProjectParamValue::Optional(optional) => {
                                if optional.enabled {
                                    let inner = optional.value.as_ref();
                                    Some(#inner_constructor)
                                } else {
                                    None
                                }
                            }
                            _ => panic!("Expected Option<...> for field '{}'", stringify!(#field_name)),
                        },
                    }
                }
                ParsedFieldType::OptionStruct { inner_ty } => {
                    quote! {
                        #field_name: match #value_expr {
                            ProjectParamValue::Optional(optional) => {
                                if optional.enabled {
                                    Some(match optional.value.as_ref() {
                                        ProjectParamValue::Struct(s) => #inner_ty::new_from_list(s.fields.clone()),
                                        _ => panic!("Expected Option<struct> for field '{}'", stringify!(#field_name)),
                                    })
                                } else {
                                    None
                                }
                            }
                            _ => panic!("Expected Option<...> for field '{}'", stringify!(#field_name)),
                        },
                    }
                }
                ParsedFieldType::VecLeaf {
                    inner_type_name, ..
                } => {
                    let inner_value_expr = quote! { item };
                    let inner_constructor = make_leaf_constructor_expr(
                        inner_type_name.as_str(),
                        field_name,
                        inner_value_expr,
                    );
                    quote! {
                        #field_name: match #value_expr {
                            ProjectParamValue::Vec(vec_value) => {
                                vec_value.items.iter().map(|item| #inner_constructor).collect()
                            }
                            _ => panic!("Expected Vec<...> for field '{}'", stringify!(#field_name)),
                        },
                    }
                }
                ParsedFieldType::VecStruct { inner_ty } => {
                    quote! {
                        #field_name: match #value_expr {
                            ProjectParamValue::Vec(vec_value) => {
                                vec_value.items.iter().map(|item| {
                                    match item {
                                        ProjectParamValue::Struct(s) => #inner_ty::new_from_list(s.fields.clone()),
                                        _ => panic!("Expected Vec<struct> for field '{}'", stringify!(#field_name)),
                                    }
                                }).collect()
                            }
                            _ => panic!("Expected Vec<...> for field '{}'", stringify!(#field_name)),
                        },
                    }
                }
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
        impl PlotteryParams for #name {
            #get_params_impl
            #new_from_map_impl

            #new_from_list_impl
            #new_with_defaults_impl
        }
    };
    TokenStream::from(expanded)
}
