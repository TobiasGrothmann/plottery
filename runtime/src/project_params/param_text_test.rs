#[cfg(test)]
mod tests {
    use plottery_lib::{
        ColorRgb, Margin, NoiseScale, NoiseSettings, NoiseType, Paper, PaperSize, Rect, SystemFont,
        V2i, V2,
    };

    use crate::{
        AngleUnit, ParamField, ProjectParam, ProjectParamDataEnum, ProjectParamEnum,
        ProjectParamFile, ProjectParamOptional, ProjectParamSeed, ProjectParamStruct,
        ProjectParamValue as Value, ProjectParamVariant, ProjectParamVec, ProjectParamsListWrapper,
    };

    /// Sets `text` on `start`, expects `expected`, and that `expected` prints as `text`.
    fn assert_round_trip(mut start: Value, text: &str, expected: Value) {
        start.set_from_text(text).unwrap();
        assert_eq!(start, expected, "setting '{text}'");
        assert_eq!(expected.to_text(), text);
    }

    #[test]
    fn round_trip() {
        let enum_value =
            |selected| Value::Enum(ProjectParamEnum::new(selected, &["Lines", "Dots"]));
        let paper = |size, landscape| Value::Paper(Paper::new(size, landscape, Margin::Cm(2.0)));
        let cases = [
            (Value::Float(1.0), "-12.5", Value::Float(-12.5)),
            (
                Value::FloatRanged {
                    val: 1.0,
                    min: 0.0,
                    max: 100.0,
                },
                "12.5",
                Value::FloatRanged {
                    val: 12.5,
                    min: 0.0,
                    max: 100.0,
                },
            ),
            (
                Value::Percent {
                    val: 1.0,
                    min: 0.0,
                    max: 100.0,
                },
                "12.5%",
                Value::Percent {
                    val: 12.5,
                    min: 0.0,
                    max: 100.0,
                },
            ),
            (Value::Int(1), "-3", Value::Int(-3)),
            (
                Value::IntRanged {
                    val: 1,
                    min: 0,
                    max: 5,
                },
                "5",
                Value::IntRanged {
                    val: 5,
                    min: 0,
                    max: 5,
                },
            ),
            (Value::Bool(false), "true", Value::Bool(true)),
            (enum_value("Lines"), "Dots", enum_value("Dots")),
            (
                Value::String("a".into()),
                " two words ",
                Value::String(" two words ".into()),
            ),
            (
                Value::Angle {
                    rad: 0.0,
                    unit: AngleUnit::Degrees,
                },
                "45deg",
                Value::Angle {
                    rad: std::f32::consts::FRAC_PI_4,
                    unit: AngleUnit::Degrees,
                },
            ),
            (
                Value::V2(V2::zero()),
                "1.5,-2",
                Value::V2(V2::new(1.5, -2.0)),
            ),
            (
                Value::V2i(V2i::new(0, 0)),
                "3,4",
                Value::V2i(V2i::new(3, 4)),
            ),
            (
                Value::Rect(Rect::default()),
                "1,2.5,10,15",
                Value::Rect(Rect::new(V2::new(1.0, 2.5), V2::new(10.0, 15.0))),
            ),
            (
                Value::RangeF32 {
                    min: 0.0,
                    max: 1.0,
                    bounds: Some((0.0, 10.0)),
                },
                "2.5..5",
                Value::RangeF32 {
                    min: 2.5,
                    max: 5.0,
                    bounds: Some((0.0, 10.0)),
                },
            ),
            (
                Value::RangeI32 {
                    min: 0,
                    max: 1,
                    bounds: None,
                },
                "-3..-1",
                Value::RangeI32 {
                    min: -3,
                    max: -1,
                    bounds: None,
                },
            ),
            (
                Value::Color(ColorRgb::black()),
                "#3366cc",
                Value::Color(ColorRgb::from_hex("#3366cc").unwrap()),
            ),
            (
                paper(PaperSize::DinA(4), false),
                "A3 landscape",
                paper(PaperSize::DinA(3), true),
            ),
            (
                paper(PaperSize::DinA(4), true),
                "30x40",
                paper(PaperSize::Custom(V2::new(30.0, 40.0)), false),
            ),
            (
                Value::Image(ProjectParamFile::new(None)),
                "resources/images/a.png",
                Value::Image(ProjectParamFile::new(Some("resources/images/a.png".into()))),
            ),
            (
                Value::Optional(ProjectParamOptional::new(false, Value::Float(1.0))),
                "2",
                Value::Optional(ProjectParamOptional::new(true, Value::Float(2.0))),
            ),
            (
                Value::Optional(ProjectParamOptional::new(true, Value::Float(1.0))),
                "off",
                Value::Optional(ProjectParamOptional::new(false, Value::Float(1.0))),
            ),
            (
                Value::SystemFont(SystemFont::new("Futura").with_italic(true)),
                "Helvetica Neue",
                Value::SystemFont(SystemFont::new("Helvetica Neue")),
            ),
            (
                Value::SystemFont(SystemFont::default()),
                "Helvetica Neue:700",
                Value::SystemFont(SystemFont::new("Helvetica Neue").with_weight(700.0)),
            ),
            (
                Value::SystemFont(SystemFont::default()),
                "Helvetica Neue:300:italic",
                Value::SystemFont(
                    SystemFont::new("Helvetica Neue")
                        .with_weight(300.0)
                        .with_italic(true),
                ),
            ),
        ];
        for (start, text, expected) in cases {
            assert_round_trip(start, text, expected);
        }
    }

    #[test]
    fn angle_input_in_another_unit_keeps_param_unit() {
        let mut angle = Value::Angle {
            rad: 0.0,
            unit: AngleUnit::Degrees,
        };
        angle.set_from_text("0.25rot").unwrap();
        assert_eq!(angle.to_text(), "90deg");
        angle.set_from_text("30").unwrap();
        assert_eq!(angle.to_text(), "30deg");
    }

    #[test]
    fn percent_sign_is_optional() {
        let mut percent = Value::Percent {
            val: 0.0,
            min: 0.0,
            max: 100.0,
        };
        percent.set_from_text("30").unwrap();
        assert_eq!(percent.to_text(), "30%");
    }

    #[test]
    fn rect_swaps_crossed_corners_and_needs_four_numbers() {
        let mut rect = Value::Rect(Rect::default());
        rect.set_from_text("10,15,1,2").unwrap();
        assert_eq!(rect.to_text(), "1,2,10,15");

        for text in ["1,2,3", "1,2,3,4,5", "1,2,x,4", ""] {
            assert_eq!(
                rect.set_from_text(text).unwrap_err().to_string(),
                format!("'{text}' is not a rect, use left,bottom,right,top like 1,2,10,15")
            );
        }
    }

    #[test]
    fn seed_keeps_history() {
        let mut seed = Value::Seed(ProjectParamSeed::new(1));
        seed.set_from_text("1234").unwrap();
        let Value::Seed(seed) = seed else {
            unreachable!()
        };
        assert_eq!((seed.current, seed.history), (1234, vec![1]));
    }

    #[test]
    fn invalid_values_are_errors() {
        let error =
            |mut value: Value, text: &str| value.set_from_text(text).unwrap_err().to_string();
        for text in ["Futura:heavy", "Futura:0", ":700", ""] {
            assert_eq!(
                error(Value::SystemFont(SystemFont::default()), text),
                format!("'{text}' is not a font, use family[:weight][:italic], e.g. Futura:700")
            );
        }
        assert_eq!(
            error(
                Value::FloatRanged {
                    val: 1.0,
                    min: 0.0,
                    max: 100.0
                },
                "120"
            ),
            "120 is outside 0..100"
        );
        assert_eq!(
            error(
                Value::FloatRanged {
                    val: 1.0,
                    min: 0.0,
                    max: 100.0
                },
                "NaN"
            ),
            "NaN is outside 0..100"
        );
        assert_eq!(
            error(
                Value::Percent {
                    val: 1.0,
                    min: 0.0,
                    max: 100.0
                },
                "120%"
            ),
            "120 is outside 0..100"
        );
        assert_eq!(
            error(
                Value::RangeI32 {
                    min: 1,
                    max: 2,
                    bounds: Some((0, 10))
                },
                "5..11"
            ),
            "11 is outside 0..10"
        );
        assert_eq!(
            error(
                Value::RangeF32 {
                    min: 1.0,
                    max: 2.0,
                    bounds: None
                },
                "3..1"
            ),
            "3 is larger than 1"
        );
        assert_eq!(
            error(
                Value::Enum(ProjectParamEnum::new("Lines", &["Lines", "Dots"])),
                "dots"
            ),
            "'dots' is not a variant, use one of Lines | Dots"
        );
        assert!(error(Value::Int(0), "1.5").contains("not a valid i32"));
        assert!(error(Value::Paper(Paper::default()), "A11").contains("not a paper"));
        assert!(error(Value::Easing(Default::default()), "linear").contains("not settable"));
    }

    fn nested_params() -> ProjectParamsListWrapper {
        let glow = ProjectParamStruct::new(vec![ProjectParam::new("radius", Value::Float(1.0))]);
        ProjectParamsListWrapper::new(vec![
            ProjectParam::new("size", Value::Float(1.0)),
            ProjectParam::new(
                "glow",
                Value::Optional(ProjectParamOptional::new(
                    false,
                    Value::Struct(glow.clone()),
                )),
            ),
            ProjectParam::new("inner", Value::Struct(glow)),
            ProjectParam::new(
                "points",
                Value::Vec(ProjectParamVec::new(
                    Value::V2(V2::zero()),
                    vec![Value::V2(V2::zero()), Value::V2(V2::zero())],
                )),
            ),
        ])
    }

    #[test]
    fn set_dotted_paths() {
        let mut params = nested_params();
        params.set("glow", "on").unwrap();
        params.set("glow.radius", "2").unwrap();
        params.set("inner.radius", "3").unwrap();
        params.set("points.1", "4,5").unwrap();

        let mut expected = nested_params();
        let glow = ProjectParamStruct::new(vec![ProjectParam::new("radius", Value::Float(2.0))]);
        expected.list[1].value =
            Value::Optional(ProjectParamOptional::new(true, Value::Struct(glow)));
        expected.list[2].value = Value::Struct(ProjectParamStruct::new(vec![ProjectParam::new(
            "radius",
            Value::Float(3.0),
        )]));
        expected.list[3].value = Value::Vec(ProjectParamVec::new(
            Value::V2(V2::zero()),
            vec![Value::V2(V2::zero()), Value::V2(V2::new(4.0, 5.0))],
        ));
        let values = |p: &ProjectParamsListWrapper| {
            p.list.iter().map(|p| p.value.clone()).collect::<Vec<_>>()
        };
        assert_eq!(values(&params), values(&expected));
    }

    #[test]
    fn set_noise_fields() {
        let mut params = ProjectParamsListWrapper::new(vec![ProjectParam::new(
            "noise",
            NoiseSettings::default_param_value(),
        )]);
        params.set("noise.noise_type", "Simplex").unwrap();
        params.set("noise.frequency", "2").unwrap();
        params.set("noise.scale", "PerPaper").unwrap();
        params.set("noise.octaves", "3").unwrap();

        assert_eq!(
            NoiseSettings::from_param_value(&params.list[0].value).unwrap(),
            NoiseSettings::new(NoiseType::Simplex, 2.0, 3, NoiseScale::PerPaper)
        );
        assert!(params.set("noise.octaves", "0").is_err());
    }

    #[test]
    fn set_unknown_paths() {
        let mut params = nested_params();
        let error = |params: &mut ProjectParamsListWrapper, path| {
            params.set(path, "1").unwrap_err().to_string()
        };
        assert_eq!(
            error(&mut params, "nope"),
            "unknown param 'nope', params are: size, glow, inner, points"
        );
        assert_eq!(
            error(&mut params, "glow.nope"),
            "'glow' has no field 'nope', fields are: radius"
        );
        assert_eq!(
            error(&mut params, "points.2"),
            "'points' has 2 items, use an index from 0 to 1"
        );
        assert_eq!(
            error(&mut params, "size.x"),
            "'size' is a f32 without fields"
        );
        assert!(error(&mut params, "glow").starts_with("glow: use on or off"));
    }

    fn enum_params() -> ProjectParamsListWrapper {
        let pattern = Value::DataEnum(ProjectParamDataEnum::new(
            "Grid",
            vec![
                ProjectParamVariant::new("Grid", vec![ProjectParam::new("cols", Value::Int(4))]),
                ProjectParamVariant::new(
                    "Spiral",
                    vec![ProjectParam::new("turns", Value::Float(3.0))],
                ),
                ProjectParamVariant::new("Random", vec![]),
            ],
        ));
        ProjectParamsListWrapper::new(vec![
            ProjectParam::new("pattern", pattern.clone()),
            ProjectParam::new(
                "overlay",
                Value::Optional(ProjectParamOptional::new(false, pattern)),
            ),
        ])
    }

    /// `selected` and the value of `field` in `variant`, selected or not.
    fn variant_field(value: &Value, variant: &str, field: &str) -> (String, Value) {
        let data_enum = match value {
            Value::DataEnum(data_enum) => data_enum,
            Value::Optional(optional) => match optional.value.as_ref() {
                Value::DataEnum(data_enum) => data_enum,
                other => panic!("expected a data enum, found {}", other.type_name()),
            },
            other => panic!("expected a data enum, found {}", other.type_name()),
        };
        let fields = &data_enum.variant(variant).unwrap().fields;
        let field = fields.iter().find(|f| f.name == field).unwrap();
        (data_enum.selected.clone(), field.value.clone())
    }

    #[test]
    fn set_data_enum_variant_and_its_fields() {
        let mut params = enum_params();
        params.set("pattern.cols", "2").unwrap();
        params.set("pattern", "Spiral").unwrap();
        params.set("pattern.turns", "4").unwrap();
        params.set("overlay", "Spiral").unwrap();
        params.set("overlay.turns", "5").unwrap();

        let pattern = &params.list[0].value;
        assert_eq!(pattern.to_text(), "Spiral");
        assert_eq!(
            variant_field(pattern, "Spiral", "turns"),
            ("Spiral".to_string(), Value::Float(4.0))
        );
        // the variant that is no longer selected keeps its value
        assert_eq!(variant_field(pattern, "Grid", "cols").1, Value::Int(2));

        let overlay = &params.list[1].value;
        assert_eq!(overlay.to_text(), "Spiral");
        assert_eq!(
            variant_field(overlay, "Spiral", "turns"),
            ("Spiral".to_string(), Value::Float(5.0))
        );
    }

    #[test]
    fn data_enum_path_errors_name_the_variant() {
        let mut params = enum_params();
        let error = |params: &mut ProjectParamsListWrapper, path, text| {
            params.set(path, text).unwrap_err().to_string()
        };
        assert_eq!(
            error(&mut params, "pattern.turns", "1"),
            "'turns' is a field of Spiral, but 'pattern' is Grid, set pattern=Spiral first"
        );
        assert_eq!(
            error(&mut params, "pattern.nope", "1"),
            "'pattern' has no field 'nope', fields are: cols"
        );
        assert_eq!(
            error(&mut params, "pattern", "Dots"),
            "pattern: 'Dots' is not a variant, use one of Grid | Spiral | Random"
        );
        params.set("pattern", "Random").unwrap();
        assert_eq!(
            error(&mut params, "pattern.nope", "1"),
            "'pattern' is Random, which has no fields"
        );
    }
}
