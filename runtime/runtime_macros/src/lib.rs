use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    self, punctuated::Punctuated, Expr, Field, Fields, GenericArgument, Meta, PathArguments, Token,
};

/// Derives `PlotteryParams` and `ParamField` for a struct of named fields.
///
/// Every field type must implement `ParamField`. `#[value(expr)]` sets the default,
/// `#[range(min, max)]` restricts the value (needs `RangedParamField`),
/// `#[section("Name")]` starts an editor section that runs until the next one,
/// and `///` doc comments become the param's description.
#[proc_macro_derive(PlotteryParams, attributes(value, range, section))]
pub fn plottery_params(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Failed to parse macro.");
    plottery_params_impl(&ast)
}

/// Derives `ParamField` for an enum. Exactly one variant must be marked `#[default]`.
///
/// Variants can have fields, shown below the selector while their variant is selected: either
/// named fields with the attributes of `PlotteryParams` fields, or one `PlotteryParams` struct
/// as `Variant(Struct)`. The values of the other variants are kept.
#[proc_macro_derive(PlotteryEnum, attributes(default, value, range, section))]
pub fn plottery_enum(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Failed to parse macro.");
    plottery_enum_impl(&ast)
}

struct FieldAttributes {
    value: Option<proc_macro2::TokenStream>,
    range: Option<(proc_macro2::TokenStream, proc_macro2::TokenStream)>,
    section: Option<String>,
    description: Option<String>,
}

fn parse_attribute_args(list: &syn::MetaList, expected: usize) -> Vec<Expr> {
    let name = list.path.to_token_stream().to_string();
    let args = list
        .parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)
        .unwrap_or_else(|_| panic!("Failed to parse arguments of attribute '{}'", name));
    if args.len() != expected {
        panic!(
            "Invalid number of arguments for attribute '{}'. Expected {}, found {}",
            name,
            expected,
            args.len()
        );
    }
    args.into_iter().collect()
}

fn parse_field_attributes(field: &Field) -> FieldAttributes {
    let mut attributes = FieldAttributes {
        value: None,
        range: None,
        section: None,
        description: None,
    };
    let mut doc_lines = Vec::new();

    for attr in &field.attrs {
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(syn::MetaNameValue {
                value:
                    Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(line),
                        ..
                    }),
                ..
            }) = &attr.meta
            {
                doc_lines.push(line.value().trim().to_string());
            }
            continue;
        }
        let Meta::List(list) = &attr.meta else {
            panic!(
                "Unknown attribute '{}'. Expected #[value(...)], #[range(...)] or #[section(...)].",
                attr.path().to_token_stream()
            );
        };
        if list.path.is_ident("value") {
            let args = parse_attribute_args(list, 1);
            let value = &args[0];
            attributes.value = Some(quote!(#value));
        } else if list.path.is_ident("range") {
            let args = parse_attribute_args(list, 2);
            let (min, max) = (&args[0], &args[1]);
            attributes.range = Some((quote!(#min), quote!(#max)));
        } else if list.path.is_ident("section") {
            let title = list.parse_args::<syn::LitStr>().unwrap_or_else(|_| {
                panic!("#[section] expects one string, e.g. #[section(\"Frames\")]")
            });
            attributes.section = Some(title.value());
        } else {
            panic!(
                "Unknown attribute '{}'. Only #[value(...)], #[range(...)] and #[section(...)] are supported for fields.",
                list.path.to_token_stream()
            );
        }
    }

    let description = doc_lines
        .iter()
        .filter(|line| !line.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    attributes.description = Some(description).filter(|d| !d.is_empty());
    attributes
}

fn single_generic_arg(ty: &syn::Type) -> Option<(String, &syn::Type)> {
    let syn::Type::Path(path) = ty else {
        return None;
    };
    let segment = path.path.segments.last()?;
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    if args.args.len() != 1 {
        return None;
    }
    match args.args.first()? {
        GenericArgument::Type(inner) => Some((segment.ident.to_string(), inner)),
        _ => None,
    }
}

/// Rejects wrappers the editor cannot render, before the trait bounds produce a less helpful error.
fn check_field_type(field: &Field) {
    let Some((outer, inner)) = single_generic_arg(&field.ty) else {
        return;
    };
    let Some((inner_name, _)) = single_generic_arg(inner) else {
        return;
    };
    let field_name = field.ident.as_ref().expect("Named field").to_string();
    match (outer.as_str(), inner_name.as_str()) {
        ("Option", "Option") => panic!(
            "Field '{}': Option<Option<T>> is not supported for PlotteryParams.",
            field_name
        ),
        ("Vec", "Option") => panic!(
            "Field '{}': Vec<Option<T>> is not supported for PlotteryParams.",
            field_name
        ),
        ("Vec", "Vec") => panic!(
            "Field '{}': Vec<Vec<T>> is not supported for PlotteryParams.",
            field_name
        ),
        _ => {}
    }
}

/// Builds the `ProjectParamValue` for the reference `value` (or the type default when `None`), honoring `#[range]`.
fn field_value_tokens(
    ty: &syn::Type,
    value: Option<proc_macro2::TokenStream>,
    range: &Option<(proc_macro2::TokenStream, proc_macro2::TokenStream)>,
) -> proc_macro2::TokenStream {
    match (value, range) {
        (None, None) => quote! {
            <#ty as ::plottery_runtime::ParamField>::default_param_value()
        },
        (None, Some((min, max))) => quote! {
            <#ty as ::plottery_runtime::RangedParamField>::default_ranged_param_value(#min, #max)
        },
        (Some(value), None) => quote! {
            <#ty as ::plottery_runtime::ParamField>::to_param_value(#value)
        },
        (Some(value), Some((min, max))) => quote! {
            <#ty as ::plottery_runtime::RangedParamField>::to_ranged_param_value(#value, #min, #max)
        },
    }
}

/// Generated code per named field, each ending in a comma.
struct FieldTokens {
    /// `ProjectParam`s holding the defaults.
    defaults: Vec<proc_macro2::TokenStream>,
    /// `ProjectParam`s holding the current values.
    to_params: Vec<proc_macro2::TokenStream>,
    /// `field: value` initializers reading a map named `params`.
    from_map: Vec<proc_macro2::TokenStream>,
}

/// `access` gives a reference to a field's current value.
fn named_field_tokens(
    fields: &syn::FieldsNamed,
    access: impl Fn(&syn::Ident) -> proc_macro2::TokenStream,
) -> FieldTokens {
    let mut tokens = FieldTokens {
        defaults: Vec::new(),
        to_params: Vec::new(),
        from_map: Vec::new(),
    };
    let mut current_section: Option<String> = None;

    for field in &fields.named {
        check_field_type(field);
        let ident = field.ident.as_ref().expect("Named field");
        let field_name = ident.to_string();
        let ty = &field.ty;
        let attributes = parse_field_attributes(field);
        if attributes.section.is_some() {
            current_section = attributes.section.clone();
        }
        let metadata = {
            let section = current_section.iter().map(|s| quote!(.with_section(#s)));
            let description = attributes
                .description
                .iter()
                .map(|d| quote!(.with_description(#d)));
            quote!(#(#section)* #(#description)*)
        };

        let default_value = attributes
            .value
            .map(|value| quote!(&{ let v: #ty = #value; v }));
        let default_tokens = field_value_tokens(ty, default_value, &attributes.range);
        tokens.defaults.push(quote! {
            ::plottery_runtime::ProjectParam::new(#field_name, #default_tokens) #metadata,
        });

        let current_tokens = field_value_tokens(ty, Some(access(ident)), &attributes.range);
        tokens.to_params.push(quote! {
            ::plottery_runtime::ProjectParam::new(#field_name, #current_tokens) #metadata,
        });

        tokens.from_map.push(quote! {
            #ident: ::plottery_runtime::param_field_from_map(params, #field_name)?,
        });
    }
    tokens
}

fn plottery_params_impl(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let data = match &ast.data {
        syn::Data::Struct(data) => data,
        syn::Data::Enum(_) => {
            panic!(
                "PlotteryParams cannot be derived for enums. Use #[derive(PlotteryEnum)] instead."
            )
        }
        syn::Data::Union(_) => panic!("PlotteryParams cannot be derived for unions."),
    };
    if !ast.generics.params.is_empty() {
        panic!("PlotteryParams cannot be derived for generic structs.");
    }
    let Fields::Named(fields) = &data.fields else {
        panic!("PlotteryParams requires a struct with named fields.");
    };

    let FieldTokens {
        defaults,
        to_params,
        from_map,
    } = named_field_tokens(fields, |ident| quote!(&self.#ident));

    let expanded = quote! {
        impl ::plottery_runtime::PlotteryParams for #name {
            fn param_defaults_list() -> ::std::vec::Vec<::plottery_runtime::ProjectParam> {
                vec![ #(#defaults)* ]
            }

            fn to_params_list(&self) -> ::std::vec::Vec<::plottery_runtime::ProjectParam> {
                vec![ #(#to_params)* ]
            }

            fn try_new_from_map(
                params: &::std::collections::HashMap<String, ::plottery_runtime::ProjectParam>,
            ) -> ::std::result::Result<Self, ::plottery_runtime::ParamFieldError> {
                Ok(Self { #(#from_map)* })
            }
        }

        impl ::plottery_runtime::ParamField for #name {
            fn default_param_value() -> ::plottery_runtime::ProjectParamValue {
                ::plottery_runtime::ProjectParamValue::Struct(
                    ::plottery_runtime::ProjectParamStruct::new(
                        <Self as ::plottery_runtime::PlotteryParams>::param_defaults_list(),
                    ),
                )
            }

            fn to_param_value(&self) -> ::plottery_runtime::ProjectParamValue {
                ::plottery_runtime::ProjectParamValue::Struct(
                    ::plottery_runtime::ProjectParamStruct::new(
                        <Self as ::plottery_runtime::PlotteryParams>::to_params_list(self),
                    ),
                )
            }

            fn from_param_value(
                value: &::plottery_runtime::ProjectParamValue,
            ) -> ::std::result::Result<Self, ::plottery_runtime::ParamFieldError> {
                match value {
                    ::plottery_runtime::ProjectParamValue::Struct(s) => {
                        <Self as ::plottery_runtime::PlotteryParams>::try_new_from_list(s.fields.clone())
                    }
                    other => Err(::plottery_runtime::param_type_mismatch("struct", other)),
                }
            }
        }
    };
    TokenStream::from(expanded)
}

fn plottery_enum_impl(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let syn::Data::Enum(data) = &ast.data else {
        panic!("PlotteryEnum can only be derived for enums.");
    };
    if !ast.generics.params.is_empty() {
        panic!("PlotteryEnum cannot be derived for generic enums.");
    }

    let default_variants: Vec<_> = data
        .variants
        .iter()
        .filter(|variant| variant.attrs.iter().any(|a| a.path().is_ident("default")))
        .collect();
    let default_name = match default_variants.as_slice() {
        [variant] => variant.ident.to_string(),
        [] => panic!(
            "PlotteryEnum: mark one variant of '{}' with #[default].",
            name
        ),
        _ => panic!("PlotteryEnum: only one variant may be marked #[default]."),
    };

    let expanded = if data
        .variants
        .iter()
        .all(|v| matches!(v.fields, Fields::Unit))
    {
        unit_enum_tokens(name, data, &default_name)
    } else {
        data_enum_tokens(name, data, &default_name)
    };
    TokenStream::from(expanded)
}

fn unit_enum_tokens(
    name: &syn::Ident,
    data: &syn::DataEnum,
    default_name: &str,
) -> proc_macro2::TokenStream {
    let variant_idents: Vec<_> = data.variants.iter().map(|v| &v.ident).collect();
    let variant_names: Vec<_> = variant_idents.iter().map(|i| i.to_string()).collect();

    quote! {
        impl ::plottery_runtime::ParamField for #name {
            fn default_param_value() -> ::plottery_runtime::ProjectParamValue {
                ::plottery_runtime::ProjectParamValue::Enum(
                    ::plottery_runtime::ProjectParamEnum::new(#default_name, &[#(#variant_names),*]),
                )
            }

            fn to_param_value(&self) -> ::plottery_runtime::ProjectParamValue {
                let selected = match self {
                    #(Self::#variant_idents => #variant_names,)*
                };
                ::plottery_runtime::ProjectParamValue::Enum(
                    ::plottery_runtime::ProjectParamEnum::new(selected, &[#(#variant_names),*]),
                )
            }

            fn from_param_value(
                value: &::plottery_runtime::ProjectParamValue,
            ) -> ::std::result::Result<Self, ::plottery_runtime::ParamFieldError> {
                match value {
                    ::plottery_runtime::ProjectParamValue::Enum(e) => match e.selected.as_str() {
                        #(#variant_names => Ok(Self::#variant_idents),)*
                        other => Err(::plottery_runtime::param_type_mismatch(
                            concat!("enum ", stringify!(#name)),
                            value,
                        )
                        .context(format!("unknown variant '{}'", other))),
                    },
                    other => Err(::plottery_runtime::param_type_mismatch(
                        concat!("enum ", stringify!(#name)),
                        other,
                    )),
                }
            }
        }
    }
}

/// An enum with at least one variant that has fields, stored as `ProjectParamValue::DataEnum`.
fn data_enum_tokens(
    name: &syn::Ident,
    data: &syn::DataEnum,
    default_name: &str,
) -> proc_macro2::TokenStream {
    let mut default_variants = Vec::new();
    let mut to_param_arms = Vec::new();
    let mut from_param_arms = Vec::new();

    for variant in &data.variants {
        let ident = &variant.ident;
        let variant_name = ident.to_string();
        // Per variant: its default fields, the pattern binding its fields, their current values
        // and the expression building it from a map named `params`.
        let (defaults, pattern, current, construct) = match &variant.fields {
            Fields::Unit => (
                quote!(::std::vec::Vec::new()),
                quote!(Self::#ident),
                quote!(::std::vec::Vec::new()),
                quote!(Self::#ident),
            ),
            Fields::Named(fields) => {
                let FieldTokens {
                    defaults,
                    to_params,
                    from_map,
                } = named_field_tokens(fields, |field| quote!(#field));
                let field_idents = fields.named.iter().map(|f| &f.ident);
                (
                    quote!(vec![ #(#defaults)* ]),
                    quote!(Self::#ident { #(#field_idents),* }),
                    quote!(vec![ #(#to_params)* ]),
                    quote!(Self::#ident { #(#from_map)* }),
                )
            }
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let ty = &fields.unnamed[0].ty;
                (
                    quote!(<#ty as ::plottery_runtime::PlotteryParams>::param_defaults_list()),
                    quote!(Self::#ident(inner)),
                    quote!(<#ty as ::plottery_runtime::PlotteryParams>::to_params_list(inner)),
                    quote!(Self::#ident(
                        <#ty as ::plottery_runtime::PlotteryParams>::try_new_from_map(params)?
                    )),
                )
            }
            Fields::Unnamed(_) => panic!(
                "PlotteryEnum: variant '{}' needs named fields or exactly one PlotteryParams struct, e.g. {}(MyParams).",
                ident, ident
            ),
        };
        default_variants.push(quote! {
            ::plottery_runtime::ProjectParamVariant::new(#variant_name, #defaults),
        });
        to_param_arms.push(quote!(#pattern => (#variant_name, #current),));
        from_param_arms.push(quote!(#variant_name => Ok(#construct),));
    }

    quote! {
        impl ::plottery_runtime::ParamField for #name {
            fn default_param_value() -> ::plottery_runtime::ProjectParamValue {
                ::plottery_runtime::ProjectParamValue::DataEnum(
                    ::plottery_runtime::ProjectParamDataEnum::new(
                        #default_name,
                        vec![ #(#default_variants)* ],
                    ),
                )
            }

            fn to_param_value(&self) -> ::plottery_runtime::ProjectParamValue {
                let (selected, fields): (&str, ::std::vec::Vec<::plottery_runtime::ProjectParam>) =
                    match self {
                        #(#to_param_arms)*
                    };
                match <Self as ::plottery_runtime::ParamField>::default_param_value() {
                    ::plottery_runtime::ProjectParamValue::DataEnum(data_enum) => {
                        ::plottery_runtime::ProjectParamValue::DataEnum(
                            data_enum.with_selected(selected, fields),
                        )
                    }
                    other => other,
                }
            }

            fn from_param_value(
                value: &::plottery_runtime::ProjectParamValue,
            ) -> ::std::result::Result<Self, ::plottery_runtime::ParamFieldError> {
                match value {
                    ::plottery_runtime::ProjectParamValue::DataEnum(data_enum) => {
                        #[allow(unused_variables)]
                        let params = &data_enum.selected_fields_map();
                        match data_enum.selected.as_str() {
                            #(#from_param_arms)*
                            other => Err(::plottery_runtime::param_type_mismatch(
                                concat!("enum ", stringify!(#name)),
                                value,
                            )
                            .context(format!("unknown variant '{}'", other))),
                        }
                    }
                    other => Err(::plottery_runtime::param_type_mismatch(
                        concat!("enum ", stringify!(#name)),
                        other,
                    )),
                }
            }
        }
    }
}
