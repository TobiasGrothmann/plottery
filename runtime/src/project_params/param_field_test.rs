#[cfg(test)]
mod tests {
    use crate::*;
    use plottery_lib::{
        Angle, BundledFont, ColorRgb, Easing, Margin, NoiseScale, NoiseSettings, NoiseType, Paper,
        Range, Rect, SystemFont, V2i, V2,
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
        svg: SvgPath,
        #[value(NoiseSettings::default().with_type(NoiseType::Worley))]
        noise: NoiseSettings,
        #[value(Easing::power(-0.5))]
        easing: Easing,
        #[value(ColorRgb::new(1.0, 0.0, 0.0))]
        color: ColorRgb,
        #[value(V2i::new(4, 6))]
        cells: V2i,
        #[value(BundledFont::HersheyScript)]
        font: BundledFont,
        default_font: BundledFont,
        #[value(SystemFont::new("Futura").with_weight(700.0))]
        system_font: SystemFont,
        default_system_font: SystemFont,
        #[value(Percent::new(150.0))]
        #[range(10.0, 80.0)]
        share: Percent,
        default_share: Percent,
        area: Rect,
    }

    #[test]
    fn defaults_honor_value_and_range() {
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
        assert_eq!(
            by_name("share"),
            ProjectParamValue::Percent {
                val: 80.0,
                min: 10.0,
                max: 80.0
            }
        );
        assert_eq!(
            by_name("default_share"),
            ProjectParamValue::Percent {
                val: 0.0,
                min: 0.0,
                max: 100.0
            }
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
            svg: SvgPath::new("resources/svgs/a.svg"),
            noise: NoiseSettings::new(NoiseType::Simplex, 3.0, 4, NoiseScale::PerPaper),
            easing: Easing::s_curve(0.4),
            color: ColorRgb::new(0.2, 0.4, 0.6),
            cells: V2i::new(-1, 9),
            font: BundledFont::Zniko,
            default_font: BundledFont::EmsBird,
            system_font: SystemFont::new("Not Installed Here").with_italic(true),
            default_system_font: SystemFont::new("Menlo").with_weight(300.0),
            share: Percent::new(42.5),
            default_share: Percent::new(100.0),
            area: Rect::new(V2::new(1.0, 2.0), V2::new(3.0, 5.0)),
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
        assert_eq!(params.svg, SvgPath::none());
        assert_eq!(params.noise.noise_type, NoiseType::Worley);
        assert_eq!(params.easing, Easing::power(-0.5));
        assert_eq!(params.color, ColorRgb::new(1.0, 0.0, 0.0));
        assert_eq!(params.cells, V2i::new(4, 6));
        assert_eq!(params.font, BundledFont::HersheyScript);
        assert_eq!(params.default_font, BundledFont::EmsReadability);
        assert_eq!(
            params.system_font,
            SystemFont::new("Futura").with_weight(700.0)
        );
        assert_eq!(params.default_system_font, SystemFont::default());
        assert_eq!(params.share, Percent::new(80.0));
        assert_eq!(params.area, Rect::new(V2::zero(), V2::xy(10.0)));
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
    fn unknown_font_is_an_error() {
        let value = ProjectParamValue::Font("Comic".to_string());
        let err = BundledFont::from_param_value(&value).unwrap_err();
        assert!(format!("{err:#}").contains("Comic"), "{err:#}");
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

    #[derive(PlotteryEnum, Debug, Clone, PartialEq)]
    enum Pattern {
        #[default]
        Grid {
            #[value(4)]
            #[range(1, 20)]
            cols: i32,
            /// rows of the grid
            #[value(3)]
            rows: i32,
        },
        Spiral {
            #[value(3.0)]
            turns: f32,
        },
        Custom(Inner),
        Random,
    }

    fn variant_field(value: &ProjectParamValue, variant: &str, field: &str) -> ProjectParam {
        let ProjectParamValue::DataEnum(data_enum) = value else {
            panic!("expected a data enum, found {}", value.type_name());
        };
        let fields = &data_enum.variant(variant).unwrap().fields;
        fields.iter().find(|f| f.name == field).unwrap().clone()
    }

    #[test]
    fn data_enum_defaults_cover_every_variant() {
        let value = Pattern::default_param_value();
        let ProjectParamValue::DataEnum(data_enum) = &value else {
            panic!("expected a data enum, found {}", value.type_name());
        };
        assert_eq!(data_enum.selected, "Grid");
        assert_eq!(
            data_enum.variant_names(),
            ["Grid", "Spiral", "Custom", "Random"]
        );
        assert!(data_enum.variant("Random").unwrap().fields.is_empty());

        let cols = variant_field(&value, "Grid", "cols");
        assert_eq!(
            cols.value,
            ProjectParamValue::IntRanged {
                val: 4,
                min: 1,
                max: 20
            }
        );
        let rows = variant_field(&value, "Grid", "rows");
        assert_eq!(rows.value, ProjectParamValue::Int(3));
        assert_eq!(rows.description.as_deref(), Some("rows of the grid"));
        assert_eq!(
            variant_field(&value, "Spiral", "turns").value,
            ProjectParamValue::Float(3.0)
        );
        assert_eq!(
            variant_field(&value, "Custom", "scale").value,
            ProjectParamValue::Float(1.5)
        );
    }

    #[test]
    fn data_enum_round_trip() {
        let inner = Inner {
            scale: 0.5,
            flag: true,
        };
        let patterns = vec![
            Pattern::Grid { cols: 7, rows: 2 },
            Pattern::Spiral { turns: 1.5 },
            Pattern::Custom(inner),
            Pattern::Random,
        ];
        for pattern in &patterns {
            assert_eq!(
                &Pattern::from_param_value(&pattern.to_param_value()).unwrap(),
                pattern
            );
        }
        assert_eq!(
            Vec::<Pattern>::from_param_value(&patterns.to_param_value()).unwrap(),
            patterns
        );
        let optional = Some(Pattern::Spiral { turns: 2.0 });
        assert_eq!(
            Option::<Pattern>::from_param_value(&optional.to_param_value()).unwrap(),
            optional
        );
    }

    #[test]
    fn data_enum_value_clamps_and_keeps_other_variants_at_default() {
        let grid = Pattern::Grid { cols: 50, rows: 2 }.to_param_value();
        assert_eq!(
            Pattern::from_param_value(&grid).unwrap(),
            Pattern::Grid { cols: 20, rows: 2 }
        );

        let spiral = Pattern::Spiral { turns: 1.5 }.to_param_value();
        let cols = variant_field(&spiral, "Grid", "cols").value;
        assert_eq!(cols.get_i32().unwrap(), 4);
    }

    #[test]
    fn unknown_data_enum_variant_is_an_error() {
        let ProjectParamValue::DataEnum(mut data_enum) = Pattern::default_param_value() else {
            panic!("expected a data enum");
        };
        data_enum.selected = "Squares".to_string();
        let err = Pattern::from_param_value(&ProjectParamValue::DataEnum(data_enum)).unwrap_err();
        assert!(format!("{err:#}").contains("Squares"), "{err:#}");
    }
}
