#[cfg(test)]
mod tests {
    use crate::*;
    use plottery_lib::{
        Angle, ColorRgb, Margin, NoiseScale, NoiseSettings, NoiseType, Paper, Range, V2,
    };

    #[derive(PlotteryEnum, Debug, Clone, Copy, PartialEq)]
    enum Mode {
        Lines,
        #[default]
        Circles,
        Dots,
    }

    #[derive(PlotteryParams, Debug, PartialEq, Clone)]
    struct Inner {
        #[value(1.5)]
        scale: f32,
        flag: bool,
    }

    #[derive(PlotteryParams, Debug, PartialEq)]
    struct Params {
        #[value(Some(3.0))]
        opt_value: Option<f32>,
        #[value(Some(7))]
        #[range(0, 5)]
        opt_ranged: Option<i32>,
        #[value(42.0)]
        #[range(0.0, 10.0)]
        clamped: f32,
        #[value(vec![1, 2, 3])]
        numbers: Vec<i32>,
        #[value(Inner { scale: 0.5, flag: true })]
        inner: Inner,
        mode: Mode,
        #[value(Mode::Dots)]
        mode_with_default: Mode,
        #[value("hello".to_string())]
        text: String,
        #[value(Paper::din_a(3).with_landscape(true))]
        paper: Paper,
        #[value(Range::new(2.0, 50.0))]
        #[range(0.0, 10.0)]
        radius: Range<f32>,
        count: Range<i32>,
        #[value(Angle::from_degrees(45.0))]
        rotation: Angle,
        #[value(V2::new(1.0, 2.0))]
        center: V2,
        #[value(Seed::new(7))]
        seed: Seed,
        image: ImagePath,
        #[value(NoiseSettings::default().with_type(NoiseType::Worley))]
        noise: NoiseSettings,
        #[value(ColorRgb::new(1.0, 0.0, 0.0))]
        color: ColorRgb,
    }

    #[test]
    fn defaults_honour_value_and_range() {
        let defaults = Params::param_defaults_list();
        let by_name = |name: &str| {
            defaults
                .iter()
                .find(|p| p.name == name)
                .unwrap_or_else(|| panic!("missing {name}"))
                .value
                .clone()
        };

        assert_eq!(
            by_name("opt_value"),
            ProjectParamValue::Optional(ProjectParamOptional::new(
                true,
                ProjectParamValue::Float(3.0)
            ))
        );
        assert_eq!(
            by_name("opt_ranged"),
            ProjectParamValue::Optional(ProjectParamOptional::new(
                true,
                ProjectParamValue::IntRanged {
                    val: 5,
                    min: 0,
                    max: 5
                }
            ))
        );
        assert_eq!(
            by_name("clamped"),
            ProjectParamValue::FloatRanged {
                val: 10.0,
                min: 0.0,
                max: 10.0
            }
        );
        assert_eq!(
            by_name("numbers"),
            ProjectParamValue::Vec(ProjectParamVec::new(
                ProjectParamValue::Int(0),
                vec![
                    ProjectParamValue::Int(1),
                    ProjectParamValue::Int(2),
                    ProjectParamValue::Int(3)
                ]
            ))
        );
        assert_eq!(
            by_name("inner"),
            ProjectParamValue::Struct(ProjectParamStruct::new(vec![
                ProjectParam::new("scale", ProjectParamValue::Float(0.5)),
                ProjectParam::new("flag", ProjectParamValue::Bool(true)),
            ]))
        );
        assert_eq!(
            by_name("radius"),
            ProjectParamValue::RangeF32 {
                min: 2.0,
                max: 10.0,
                bounds: Some((0.0, 10.0))
            }
        );
        assert_eq!(
            by_name("mode"),
            ProjectParamValue::Enum(ProjectParamEnum::new(
                "Circles",
                &["Lines", "Circles", "Dots"]
            ))
        );
        assert_eq!(
            by_name("mode_with_default"),
            ProjectParamValue::Enum(ProjectParamEnum::new("Dots", &["Lines", "Circles", "Dots"]))
        );
    }

    #[test]
    fn params_round_trip_through_list() {
        let params = Params {
            opt_value: None,
            opt_ranged: Some(2),
            clamped: 4.0,
            numbers: vec![9, 8],
            inner: Inner {
                scale: 2.0,
                flag: false,
            },
            mode: Mode::Lines,
            mode_with_default: Mode::Circles,
            text: "plot".to_string(),
            paper: Paper::din_b(2).with_margin(Margin::Cm(1.0)),
            radius: Range::new(1.0, 9.0),
            count: Range::new(3, 8),
            rotation: Angle::from_rotations(0.75),
            center: V2::new(-3.0, 4.5),
            seed: Seed::new(-1),
            image: ImagePath::new("resources/images/a.png"),
            noise: NoiseSettings::new(NoiseType::Simplex, 3.0, 4, NoiseScale::PerPaper),
            color: ColorRgb::new(0.2, 0.4, 0.6),
        };

        let round_tripped = Params::new_from_list(params.to_params_list());
        assert_eq!(round_tripped, params);
    }

    #[test]
    fn defaults_round_trip() {
        let params = Params::new_with_defaults();
        assert_eq!(params.opt_value, Some(3.0));
        assert_eq!(params.opt_ranged, Some(5));
        assert_eq!(params.clamped, 10.0);
        assert_eq!(params.numbers, vec![1, 2, 3]);
        assert_eq!(params.mode, Mode::Circles);
        assert_eq!(params.mode_with_default, Mode::Dots);
        assert_eq!(params.text, "hello");
        assert_eq!(params.paper, Paper::din_a(3).with_landscape(true));
        assert_eq!(params.radius, Range::new(2.0, 10.0));
        assert_eq!(params.count, Range::new(0, 1));
        assert_eq!(params.rotation, Angle::from_degrees(45.0));
        assert_eq!(params.center, V2::new(1.0, 2.0));
        assert_eq!(params.seed, Seed::new(7));
        assert_eq!(params.image, ImagePath::none());
        assert_eq!(params.noise.noise_type, NoiseType::Worley);
        assert_eq!(params.color, ColorRgb::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn missing_field_is_an_error() {
        let mut list = Params::param_defaults_list();
        list.retain(|p| p.name != "mode");
        let err = Params::try_new_from_list(list).unwrap_err();
        assert!(err.to_string().contains("mode"), "{err:#}");
    }

    #[test]
    fn unknown_enum_variant_is_an_error() {
        let value = ProjectParamValue::Enum(ProjectParamEnum::new("Squares", &["Lines"]));
        let err = Mode::from_param_value(&value).unwrap_err();
        assert!(format!("{err:#}").contains("Squares"), "{err:#}");
    }

    #[test]
    fn enum_round_trip() {
        for mode in [Mode::Lines, Mode::Circles, Mode::Dots] {
            assert_eq!(
                Mode::from_param_value(&mode.to_param_value()).unwrap(),
                mode
            );
        }
    }
}
