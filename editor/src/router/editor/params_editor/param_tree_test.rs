#[cfg(test)]
mod tests {
    use plottery_lib::V2;
    use plottery_runtime::{
        AngleUnit, ProjectParam, ProjectParamDataEnum, ProjectParamFile, ProjectParamOptional,
        ProjectParamSeed, ProjectParamStruct, ProjectParamValue as Value, ProjectParamVariant,
        ProjectParamVec,
    };

    use crate::router::editor::params_editor::param_tree::{
        get_default_value, get_param_value_by_path, get_param_value_mut_by_path, no_pick_at,
        reset_value,
    };

    fn path(segments: &[&str]) -> Vec<String> {
        segments.iter().map(|segment| segment.to_string()).collect()
    }

    fn point(x: f32) -> Value {
        Value::Struct(ProjectParamStruct::new(vec![
            ProjectParam::new("pos", Value::V2(V2::new(x, 0.0))).with_no_pick(),
            ProjectParam::new("size", Value::Float(x)),
        ]))
    }

    /// Every compound kind once: an optional struct, a vec of structs, a vec of leaves and an
    /// enum with fields.
    fn params() -> Vec<ProjectParam> {
        let glow = ProjectParamStruct::new(vec![ProjectParam::new("radius", Value::Float(2.0))]);
        vec![
            ProjectParam::new(
                "glow",
                Value::Optional(ProjectParamOptional::new(true, Value::Struct(glow))),
            ),
            ProjectParam::new(
                "points",
                Value::Vec(ProjectParamVec::new(
                    point(0.0),
                    vec![point(1.0), point(2.0)],
                )),
            ),
            ProjectParam::new(
                "anchors",
                Value::Vec(ProjectParamVec::new(
                    Value::V2(V2::zero()),
                    vec![Value::V2(V2::new(1.0, 1.0))],
                )),
            )
            .with_no_pick(),
            ProjectParam::new(
                "shape",
                Value::DataEnum(ProjectParamDataEnum::new(
                    "Spiral",
                    vec![
                        ProjectParamVariant::new("Blank", vec![]),
                        ProjectParamVariant::new(
                            "Spiral",
                            vec![ProjectParam::new("turns", Value::Float(3.0))],
                        ),
                    ],
                )),
            ),
        ]
    }

    #[test]
    fn paths_reach_values_in_every_compound_kind() {
        let mut list = params();
        let cases = [
            (path(&["glow", "radius"]), Value::Float(2.0)),
            (path(&["points", "[1]", "size"]), Value::Float(2.0)),
            (path(&["anchors", "[0]"]), Value::V2(V2::new(1.0, 1.0))),
            (path(&["shape", "Spiral", "turns"]), Value::Float(3.0)),
        ];
        for (path, expected) in cases {
            assert_eq!(
                get_param_value_by_path(&list, &path).as_ref(),
                Some(&expected),
                "{path:?}"
            );
            assert_eq!(
                get_param_value_mut_by_path(&mut list, &path).map(|value| &*value),
                Some(&expected),
                "{path:?}"
            );
        }
        // an Optional at the end of a path is not unwrapped
        assert!(matches!(
            get_param_value_mut_by_path(&mut list, &path(&["glow"])),
            Some(Value::Optional(_))
        ));
    }

    #[test]
    fn paths_that_lead_nowhere_are_none() {
        let mut list = params();
        let dead_ends: [&[&str]; 8] = [
            &[],
            &["nope"],
            &["glow", "nope"],
            &["glow", "radius", "deeper"],
            &["points", "[2]", "size"],
            &["points", "1", "size"],
            &["shape", "Spiral"],
            &["shape", "Nope", "turns"],
        ];
        for segments in dead_ends {
            let path = path(segments);
            assert_eq!(get_param_value_by_path(&list, &path), None, "{path:?}");
            assert_eq!(
                get_param_value_mut_by_path(&mut list, &path),
                None,
                "{path:?}"
            );
        }
    }

    #[test]
    fn default_of_a_vec_item_past_the_end_is_the_item_prototype() {
        assert_eq!(
            get_default_value(&params(), &path(&["points", "[5]", "size"])),
            Some(Value::Float(0.0))
        );
    }

    #[test]
    fn reset_keeps_seed_history_recent_files_and_the_angle_unit() {
        let mut seed = ProjectParamSeed::new(1);
        seed.set(2);
        let mut value = Value::Seed(seed);
        reset_value(&mut value, &Value::Seed(ProjectParamSeed::new(7)));
        let Value::Seed(seed) = value else {
            panic!("not a seed");
        };
        assert_eq!((seed.current, seed.history), (7, vec![2, 1]));

        let mut value = Value::Image(ProjectParamFile::new(Some("a.png".to_string())));
        reset_value(&mut value, &Value::Image(ProjectParamFile::default()));
        let Value::Image(file) = value else {
            panic!("not an image");
        };
        assert_eq!((file.path, file.recent), (None, vec!["a.png".to_string()]));

        let mut value = Value::Angle {
            rad: 1.0,
            unit: AngleUnit::Radians,
        };
        let default = Value::Angle {
            rad: 0.5,
            unit: AngleUnit::Degrees,
        };
        reset_value(&mut value, &default);
        assert_eq!(
            value,
            Value::Angle {
                rad: 0.5,
                unit: AngleUnit::Radians
            }
        );
    }

    #[test]
    fn reset_restores_the_switch_of_an_option_and_skips_another_type() {
        let optional =
            |enabled, x| Value::Optional(ProjectParamOptional::new(enabled, Value::Float(x)));
        let mut value = optional(true, 5.0);
        reset_value(&mut value, &optional(false, 1.0));
        assert_eq!(value, optional(false, 1.0));

        let mut value = Value::Float(5.0);
        reset_value(&mut value, &Value::Bool(true));
        assert_eq!(value, Value::Float(5.0));
    }

    #[test]
    fn no_pick_is_the_flag_of_the_innermost_param() {
        let list = params();
        assert!(no_pick_at(&list, &path(&["points", "[0]", "pos"])));
        assert!(no_pick_at(&list, &path(&["points", "[7]", "pos"])));
        assert!(!no_pick_at(&list, &path(&["points", "[0]", "size"])));
        // a Vec item has the flag of its Vec
        assert!(no_pick_at(&list, &path(&["anchors", "[0]"])));
        assert!(!no_pick_at(&list, &path(&["shape", "Spiral", "turns"])));
        assert!(!no_pick_at(&list, &path(&["nope"])));
    }
}
