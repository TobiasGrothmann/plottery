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

/// Derives `ParamField` for an enum of unit variants. Exactly one variant must be marked `#[default]`.
#[proc_macro_derive(PlotteryEnum, attributes(default))]
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
                "Unknown attribute '{}'. Only #[value(...)], #[range(...)] and #[section(...)] are supported for PlotteryParams fields.",
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

/// Builds the `ProjectParamValue` for `value` (or the type default when `None`), honouring `#[range]`.
fn field_value_tokens(
    ty: &syn::Type,
    value: Option<proc_macro2::TokenStream>,
    range: &Option<(proc_macro2::TokenStream, proc_macro2::TokenStream)>,
) -> proc_macro2::TokenStream {
    match (value, range) {
        (None, None) => quote! {
            <#ty as ::plottery_project::ParamField>::default_param_value()
        },
        (None, Some((min, max))) => quote! {
            <#ty as ::plottery_project::RangedParamField>::default_ranged_param_value(#min, #max)
        },
        (Some(value), None) => quote! {
            <#ty as ::plottery_project::ParamField>::to_param_value(&#value)
        },
        (Some(value), Some((min, max))) => quote! {
            <#ty as ::plottery_project::RangedParamField>::to_ranged_param_value(&#value, #min, #max)
        },
    }
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

    let mut defaults = Vec::new();
    let mut to_params = Vec::new();
    let mut from_map = Vec::new();
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
            .map(|value| quote!({ let v: #ty = #value; v }));
        let default_tokens = field_value_tokens(ty, default_value, &attributes.range);
        defaults.push(quote! {
            ::plottery_project::ProjectParam::new(#field_name, #default_tokens) #metadata,
        });

        let self_tokens = field_value_tokens(ty, Some(quote!(self.#ident)), &attributes.range);
        to_params.push(quote! {
            ::plottery_project::ProjectParam::new(#field_name, #self_tokens) #metadata,
        });

        from_map.push(quote! {
            #ident: ::plottery_project::param_field_from_map(params, #field_name)?,
        });
    }

    let expanded = quote! {
        impl ::plottery_project::PlotteryParams for #name {
            fn param_defaults_list() -> ::std::vec::Vec<::plottery_project::ProjectParam> {
                vec![ #(#defaults)* ]
            }

            fn to_params_list(&self) -> ::std::vec::Vec<::plottery_project::ProjectParam> {
                vec![ #(#to_params)* ]
            }

            fn try_new_from_map(
                params: &::std::collections::HashMap<String, ::plottery_project::ProjectParam>,
            ) -> ::std::result::Result<Self, ::plottery_project::ParamFieldError> {
                Ok(Self { #(#from_map)* })
            }
        }

        impl ::plottery_project::ParamField for #name {
            fn default_param_value() -> ::plottery_project::ProjectParamValue {
                ::plottery_project::ProjectParamValue::Struct(
                    ::plottery_project::ProjectParamStruct::new(
                        <Self as ::plottery_project::PlotteryParams>::param_defaults_list(),
                    ),
                )
            }

            fn to_param_value(&self) -> ::plottery_project::ProjectParamValue {
                ::plottery_project::ProjectParamValue::Struct(
                    ::plottery_project::ProjectParamStruct::new(
                        <Self as ::plottery_project::PlotteryParams>::to_params_list(self),
                    ),
                )
            }

            fn from_param_value(
                value: &::plottery_project::ProjectParamValue,
            ) -> ::std::result::Result<Self, ::plottery_project::ParamFieldError> {
                match value {
                    ::plottery_project::ProjectParamValue::Struct(s) => {
                        <Self as ::plottery_project::PlotteryParams>::try_new_from_list(s.fields.clone())
                    }
                    other => Err(::plottery_project::param_type_mismatch("struct", other)),
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

    let mut variant_idents = Vec::new();
    let mut variant_names = Vec::new();
    let mut default_variant = None;

    for variant in &data.variants {
        if !matches!(variant.fields, Fields::Unit) {
            panic!(
                "PlotteryEnum only supports unit variants, but '{}' has fields.",
                variant.ident
            );
        }
        if variant
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("default"))
        {
            if default_variant.is_some() {
                panic!("PlotteryEnum: only one variant may be marked #[default].");
            }
            default_variant = Some(variant.ident.clone());
        }
        variant_names.push(variant.ident.to_string());
        variant_idents.push(variant.ident.clone());
    }

    let default_variant = default_variant.unwrap_or_else(|| {
        panic!(
            "PlotteryEnum: mark one variant of '{}' with #[default].",
            name
        )
    });
    let default_name = default_variant.to_string();

    let expanded = quote! {
        impl ::plottery_project::ParamField for #name {
            fn default_param_value() -> ::plottery_project::ProjectParamValue {
                ::plottery_project::ProjectParamValue::Enum(
                    ::plottery_project::ProjectParamEnum::new(#default_name, &[#(#variant_names),*]),
                )
            }

            fn to_param_value(&self) -> ::plottery_project::ProjectParamValue {
                let selected = match self {
                    #(Self::#variant_idents => #variant_names,)*
                };
                ::plottery_project::ProjectParamValue::Enum(
                    ::plottery_project::ProjectParamEnum::new(selected, &[#(#variant_names),*]),
                )
            }

            fn from_param_value(
                value: &::plottery_project::ProjectParamValue,
            ) -> ::std::result::Result<Self, ::plottery_project::ParamFieldError> {
                match value {
                    ::plottery_project::ProjectParamValue::Enum(e) => match e.selected.as_str() {
                        #(#variant_names => Ok(Self::#variant_idents),)*
                        other => Err(::plottery_project::param_type_mismatch(
                            concat!("enum ", stringify!(#name)),
                            value,
                        )
                        .context(format!("unknown variant '{}'", other))),
                    },
                    other => Err(::plottery_project::param_type_mismatch(
                        concat!("enum ", stringify!(#name)),
                        other,
                    )),
                }
            }
        }
    };
    TokenStream::from(expanded)
}
