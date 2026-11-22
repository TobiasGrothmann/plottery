#[cfg(test)]
mod tests {
    use syn::{parse_quote, DeriveInput};

    use crate::{plottery_enum_impl, plottery_params_impl};

    fn params_error(input: DeriveInput) -> String {
        plottery_params_impl(&input).unwrap_err().to_string()
    }

    fn enum_error(input: DeriveInput) -> String {
        plottery_enum_impl(&input).unwrap_err().to_string()
    }

    #[test]
    fn valid_inputs_expand() {
        let params: DeriveInput = parse_quote! {
            struct Params {
                /// how many
                #[section("Grid")]
                #[range(1, 10)]
                #[value(4)]
                count: i32,
                #[no_pick]
                center: V2,
                maybe: Option<Vec<f32>>,
            }
        };
        assert!(plottery_params_impl(&params).is_ok());

        let unit_enum: DeriveInput = parse_quote! {
            enum Pattern { #[default] Grid, Spiral }
        };
        assert!(plottery_enum_impl(&unit_enum).is_ok());

        let data_enum: DeriveInput = parse_quote! {
            enum Pattern {
                #[default]
                Grid,
                Spiral { #[range(1.0, 5.0)] turns: f32 },
                Custom(CustomParams),
            }
        };
        assert!(plottery_enum_impl(&data_enum).is_ok());
    }

    #[test]
    fn field_attribute_errors() {
        let unknown_list = params_error(parse_quote! {
            struct Params { #[rnge(0.0, 1.0)] size: f32 }
        });
        assert!(unknown_list.starts_with("Unknown attribute 'rnge'."));

        let unknown_path = params_error(parse_quote! {
            struct Params { #[nopick] center: V2 }
        });
        assert!(unknown_path.starts_with("Unknown attribute 'nopick'."));

        let one_bound = params_error(parse_quote! {
            struct Params { #[range(1.0)] size: f32 }
        });
        assert_eq!(
            one_bound,
            "Invalid number of arguments for attribute 'range'. Expected 2, found 1"
        );

        let no_expression = params_error(parse_quote! {
            struct Params { #[value(=)] size: f32 }
        });
        assert_eq!(
            no_expression,
            "Failed to parse arguments of attribute 'value'"
        );

        let section_ident = params_error(parse_quote! {
            struct Params { #[section(Frames)] size: f32 }
        });
        assert!(section_ident.starts_with("#[section] expects one string"));
    }

    #[test]
    fn unsupported_wrappers_name_the_field() {
        for (ty, text) in [
            (quote::quote!(Option<Option<f32>>), "Option<Option<T>>"),
            (quote::quote!(Vec<Option<f32>>), "Vec<Option<T>>"),
            (quote::quote!(Vec<Vec<f32>>), "Vec<Vec<T>>"),
        ] {
            let error = params_error(parse_quote! {
                struct Params { size: f32, nested: #ty }
            });
            assert_eq!(
                error,
                format!("Field 'nested': {text} is not supported for PlotteryParams.")
            );
        }
    }

    #[test]
    fn params_needs_a_plain_struct_with_named_fields() {
        let on_enum = params_error(parse_quote! { enum Params { A, B } });
        assert!(on_enum.contains("Use #[derive(PlotteryEnum)] instead."));

        let generic = params_error(parse_quote! { struct Params<T> { size: T } });
        assert_eq!(
            generic,
            "PlotteryParams cannot be derived for generic structs."
        );

        let named_fields = "PlotteryParams requires a struct with named fields.";
        assert_eq!(
            params_error(parse_quote! { struct Params(f32); }),
            named_fields
        );
        assert_eq!(params_error(parse_quote! { struct Params; }), named_fields);
    }

    #[test]
    fn enum_needs_exactly_one_default() {
        let none = enum_error(parse_quote! { enum Pattern { Grid, Spiral } });
        assert_eq!(
            none,
            "PlotteryEnum: mark one variant of 'Pattern' with #[default]."
        );

        let two = enum_error(parse_quote! {
            enum Pattern { #[default] Grid, #[default] Spiral }
        });
        assert_eq!(
            two,
            "PlotteryEnum: only one variant may be marked #[default]."
        );

        let on_struct = enum_error(parse_quote! { struct Pattern { size: f32 } });
        assert_eq!(on_struct, "PlotteryEnum can only be derived for enums.");
    }

    #[test]
    fn enum_variant_errors() {
        let two_unnamed = enum_error(parse_quote! {
            enum Pattern { #[default] Grid, Spiral(f32, f32) }
        });
        assert!(two_unnamed.starts_with("PlotteryEnum: variant 'Spiral' needs named fields"));

        let bad_field = enum_error(parse_quote! {
            enum Pattern { #[default] Grid, Spiral { #[rnge(1.0, 5.0)] turns: f32 } }
        });
        assert!(bad_field.starts_with("Unknown attribute 'rnge'."));
    }
}
