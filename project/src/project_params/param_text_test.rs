#[cfg(test)]
mod tests {
    use plottery_lib::{ColorRgb, Margin, Paper, PaperSize, V2i, V2};

    use crate::{
        AngleUnit, ProjectParam, ProjectParamEnum, ProjectParamFile, ProjectParamOptional,
        ProjectParamSeed, ProjectParamStruct, ProjectParamValue as Value, ProjectParamVec,
        ProjectParamsListWrapper,
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
}
