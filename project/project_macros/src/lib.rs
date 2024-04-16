use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(PlotteryParamsDefinition)]
pub fn plottery_params(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    plottery_params_impl(&ast)
}

fn plottery_params_impl(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let name = &ast.ident;

    let data = match &ast.data {
        syn::Data::Struct(data) => data,
        syn::Data::Enum(_) => panic!("PlotteryProjectParams can only be derived for structs"),
        syn::Data::Union(_) => panic!("PlotteryProjectParams can only be derived for structs"),
    };

    // GET PARAMS

    let param_list = data
        .fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap().to_string();
            quote! {
                ProjectParam::new(#field_name, ProjectParamValue::Float(0.0)),
            }
        })
        .collect::<Vec<_>>();

    let get_params_impl = quote! {
        fn get_params(&self) -> Vec<ProjectParam> {
            vec![
                #(#param_list)*
            ]
        }
    };

    // NEW FROM LIST

    let constructor_list = data
        .fields
        .iter()
        .map(|field| {
            let field_name = &field.ident;
            quote! {
                #field_name: params.get(stringify!(#field_name)).unwrap().value.get_float().unwrap(),
            }
        })
        .collect::<Vec<_>>();

    let new_from_list_impl = quote! {
        fn new_from_list(params: &std::collections::HashMap<String, ProjectParam>) -> Self {
            Self {
                #(#constructor_list)*
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
