#[cfg(test)]
mod tests {
    use crate::{
        angle_unit::AngleUnit, project_param::ProjectParam, project_param_enum::ProjectParamEnum,
        project_param_optional::ProjectParamOptional, project_param_struct::ProjectParamStruct,
        project_param_value::ProjectParamValue, project_param_vec::ProjectParamVec,
        project_params_list_wrapper::ProjectParamsListWrapper,
    };

    #[test]
    fn test_combine_params() {
        let old = ProjectParamsListWrapper::new(vec![
            ProjectParam::new("a_1", ProjectParamValue::Float(0.0)),
            ProjectParam::new("a_2", ProjectParamValue::Float(0.0)),
            ProjectParam::new("a_3", ProjectParamValue::Float(0.0)),
            ProjectParam::new("b", ProjectParamValue::Int(0)),
            ProjectParam::new("c", ProjectParamValue::Int(0)),
        ]);

        let new = ProjectParamsListWrapper::new(vec![
            ProjectParam::new("a_1", ProjectParamValue::Float(0.0)), // stays
            ProjectParam::new("a_2_renamed", ProjectParamValue::Float(0.0)), // renamed
            // a_3 is removed
            ProjectParam::new("b", ProjectParamValue::Int(1)), // changes value
            ProjectParam::new("c", ProjectParamValue::Float(1.0)), // changes type
        ]);

        let expected = ProjectParamsListWrapper::new(vec![
            ProjectParam::new("a_1", ProjectParamValue::Float(0.0)), // should stay
            ProjectParam::new("a_2_renamed", ProjectParamValue::Float(0.0)), // added in struct order
            ProjectParam::new("b", ProjectParamValue::Int(0)), // should not change value
            ProjectParam::new("c", ProjectParamValue::Float(1.0)), // should change type and value
                                                               // a_2 and a_3 should be removed
        ]);

        let combined = ProjectParamsListWrapper::new_combined(&old.list, &new.list);
        assert_eq!(combined.list.len(), expected.list.len());
        for (i, param) in combined.list.iter().enumerate() {
            assert_eq!(param, &expected.list[i]);
            assert_eq!(param.value, expected.list[i].value);
        }
    }

    #[test]
    fn test_combine_optional_params() {
        let old = ProjectParamsListWrapper::new(vec![
            ProjectParam::new(
                "opt_int",
                ProjectParamValue::Optional(ProjectParamOptional::new(
                    true,
                    ProjectParamValue::Int(42),
                )),
            ),
            ProjectParam::new(
                "opt_ranged",
                ProjectParamValue::Optional(ProjectParamOptional::new(
                    false,
                    ProjectParamValue::IntRanged {
                        val: 6,
                        min: 0,
                        max: 10,
                    },
                )),
            ),
            ProjectParam::new(
                "opt_changed",
                ProjectParamValue::Optional(ProjectParamOptional::new(
                    true,
                    ProjectParamValue::Int(9),
                )),
            ),
        ]);

        let new = ProjectParamsListWrapper::new(vec![
            ProjectParam::new(
                "opt_int",
                ProjectParamValue::Optional(ProjectParamOptional::new(
                    false,
                    ProjectParamValue::Int(0),
                )),
            ),
            ProjectParam::new(
                "opt_ranged",
                ProjectParamValue::Optional(ProjectParamOptional::new(
                    false,
                    ProjectParamValue::IntRanged {
                        val: 0,
                        min: 0,
                        max: 10,
                    },
                )),
            ),
            ProjectParam::new(
                "opt_changed",
                ProjectParamValue::Optional(ProjectParamOptional::new(
                    false,
                    ProjectParamValue::Float(1.0),
                )),
            ),
        ]);

        let expected = ProjectParamsListWrapper::new(vec![
            ProjectParam::new(
                "opt_int",
                ProjectParamValue::Optional(ProjectParamOptional::new(
                    true,
                    ProjectParamValue::Int(42),
                )),
            ),
            ProjectParam::new(
                "opt_ranged",
                ProjectParamValue::Optional(ProjectParamOptional::new(
                    false,
                    ProjectParamValue::IntRanged {
                        val: 6,
                        min: 0,
                        max: 10,
                    },
                )),
            ),
            ProjectParam::new(
                "opt_changed",
                ProjectParamValue::Optional(ProjectParamOptional::new(
                    false,
                    ProjectParamValue::Float(1.0),
                )),
            ),
        ]);

        let combined = ProjectParamsListWrapper::new_combined(&old.list, &new.list);
        assert_eq!(combined.list, expected.list);
    }

    #[test]
    fn test_combine_vec_params_keeps_items_when_schema_matches() {
        let old =
            ProjectParamsListWrapper::new(vec![ProjectParam::new(
                "points",
                ProjectParamValue::Vec(ProjectParamVec::new(
                    ProjectParamValue::Struct(ProjectParamStruct::new(vec![ProjectParam::new(
                        "x",
                        ProjectParamValue::Float(0.0),
                    )])),
                    vec![
                        ProjectParamValue::Struct(ProjectParamStruct::new(vec![
                            ProjectParam::new("x", ProjectParamValue::Float(0.25)),
                        ])),
                        ProjectParamValue::Struct(ProjectParamStruct::new(vec![
                            ProjectParam::new("x", ProjectParamValue::Float(0.75)),
                        ])),
                    ],
                )),
            )]);

        let new = ProjectParamsListWrapper::new(vec![ProjectParam::new(
            "points",
            ProjectParamValue::Vec(ProjectParamVec::new(
                ProjectParamValue::Struct(ProjectParamStruct::new(vec![ProjectParam::new(
                    "x",
                    ProjectParamValue::Float(0.0),
                )])),
                vec![],
            )),
        )]);

        let combined = ProjectParamsListWrapper::new_combined(&old.list, &new.list);
        assert_eq!(combined.list, old.list);
    }

    #[test]
    fn test_combine_vec_params_resets_items_when_schema_changes() {
        let old = ProjectParamsListWrapper::new(vec![ProjectParam::new(
            "points",
            ProjectParamValue::Vec(ProjectParamVec::new(
                ProjectParamValue::Struct(ProjectParamStruct::new(vec![ProjectParam::new(
                    "x",
                    ProjectParamValue::Float(0.0),
                )])),
                vec![ProjectParamValue::Struct(ProjectParamStruct::new(vec![
                    ProjectParam::new("x", ProjectParamValue::Float(0.25)),
                ]))],
            )),
        )]);

        let new = ProjectParamsListWrapper::new(vec![ProjectParam::new(
            "points",
            ProjectParamValue::Vec(ProjectParamVec::new(
                ProjectParamValue::Struct(ProjectParamStruct::new(vec![
                    ProjectParam::new("x", ProjectParamValue::Float(0.0)),
                    ProjectParam::new("y", ProjectParamValue::Float(0.0)),
                ])),
                vec![],
            )),
        )]);

        let combined = ProjectParamsListWrapper::new_combined(&old.list, &new.list);
        assert_eq!(combined.list, new.list);
    }

    #[test]
    fn test_combine_enum_keeps_selection_if_variant_still_exists() {
        let old = vec![ProjectParam::new(
            "mode",
            ProjectParamValue::Enum(ProjectParamEnum::new("Dots", &["Lines", "Dots"])),
        )];
        let new = vec![ProjectParam::new(
            "mode",
            ProjectParamValue::Enum(ProjectParamEnum::new(
                "Lines",
                &["Lines", "Dots", "Squares"],
            )),
        )];

        let combined = ProjectParamsListWrapper::new_combined(&old, &new);
        assert_eq!(
            combined.list[0].value,
            ProjectParamValue::Enum(ProjectParamEnum::new("Dots", &["Lines", "Dots", "Squares"]))
        );
    }

    #[test]
    fn test_combine_enum_resets_selection_if_variant_was_removed() {
        let old = vec![ProjectParam::new(
            "mode",
            ProjectParamValue::Enum(ProjectParamEnum::new("Dots", &["Lines", "Dots"])),
        )];
        let new = vec![ProjectParam::new(
            "mode",
            ProjectParamValue::Enum(ProjectParamEnum::new("Lines", &["Lines", "Squares"])),
        )];

        let combined = ProjectParamsListWrapper::new_combined(&old, &new);
        assert_eq!(combined.list, new);
    }

    #[test]
    fn test_combine_angle_keeps_value_and_unit() {
        let old = vec![ProjectParam::new(
            "rotation",
            ProjectParamValue::Angle {
                rad: 1.0,
                unit: AngleUnit::Rotations,
            },
        )];
        let new = vec![ProjectParam::new(
            "rotation",
            ProjectParamValue::Angle {
                rad: 0.5,
                unit: AngleUnit::Degrees,
            },
        )];

        let combined = ProjectParamsListWrapper::new_combined(&old, &new);
        assert_eq!(combined.list, old);
    }

    #[test]
    fn test_combine_font_keeps_known_and_resets_removed_name() {
        let font = |name: &str| ProjectParamValue::Font(name.to_string());
        let old = vec![
            ProjectParam::new("kept", font("EmsBird")),
            ProjectParam::new("removed", font("NoLongerBundled")),
        ];
        let new = vec![
            ProjectParam::new("kept", font("EmsReadability")),
            ProjectParam::new("removed", font("EmsReadability")),
        ];

        let combined = ProjectParamsListWrapper::new_combined(&old, &new);
        let values: Vec<_> = combined.list.into_iter().map(|p| p.value).collect();
        assert_eq!(values, vec![font("EmsBird"), font("EmsReadability")]);
    }

    fn float_ranged(val: f32, min: f32, max: f32) -> ProjectParamValue {
        ProjectParamValue::FloatRanged { val, min, max }
    }

    fn int_ranged(val: i32, min: i32, max: i32) -> ProjectParamValue {
        ProjectParamValue::IntRanged { val, min, max }
    }

    fn combined_values(
        old: Vec<(&str, ProjectParamValue)>,
        new: Vec<(&str, ProjectParamValue)>,
    ) -> Vec<ProjectParamValue> {
        let list = |values: Vec<(&str, ProjectParamValue)>| -> Vec<ProjectParam> {
            values
                .into_iter()
                .map(|(name, value)| ProjectParam::new(name, value))
                .collect()
        };
        ProjectParamsListWrapper::new_combined(&list(old), &list(new))
            .list
            .into_iter()
            .map(|param| param.value)
            .collect()
    }

    #[test]
    fn test_combine_keeps_value_that_fits_the_new_range() {
        let optional = |value| ProjectParamValue::Optional(ProjectParamOptional::new(true, value));
        let combined = combined_values(
            vec![
                ("float", float_ranged(4.0, 0.0, 10.0)),
                ("int", int_ranged(4, 0, 10)),
                ("gains_range", ProjectParamValue::Float(4.0)),
                ("loses_range", int_ranged(4, 0, 10)),
                ("optional", optional(float_ranged(4.0, 0.0, 10.0))),
                (
                    "range",
                    ProjectParamValue::RangeI32 {
                        min: 2,
                        max: 4,
                        bounds: Some((0, 10)),
                    },
                ),
            ],
            vec![
                ("float", float_ranged(1.0, 2.0, 5.0)),
                ("int", int_ranged(1, -5, 5)),
                ("gains_range", float_ranged(1.0, 0.0, 5.0)),
                ("loses_range", ProjectParamValue::Int(1)),
                ("optional", optional(float_ranged(1.0, 2.0, 5.0))),
                (
                    "range",
                    ProjectParamValue::RangeI32 {
                        min: 0,
                        max: 1,
                        bounds: Some((1, 5)),
                    },
                ),
            ],
        );
        assert_eq!(
            combined,
            vec![
                float_ranged(4.0, 2.0, 5.0),
                int_ranged(4, -5, 5),
                float_ranged(4.0, 0.0, 5.0),
                ProjectParamValue::Int(4),
                optional(float_ranged(4.0, 2.0, 5.0)),
                ProjectParamValue::RangeI32 {
                    min: 2,
                    max: 4,
                    bounds: Some((1, 5)),
                },
            ]
        );
    }

    #[test]
    fn test_combine_clamps_value_outside_the_new_range() {
        let optional = |value| ProjectParamValue::Optional(ProjectParamOptional::new(true, value));
        let combined = combined_values(
            vec![
                ("float", float_ranged(9.0, 0.0, 10.0)),
                ("int", int_ranged(-3, -10, 10)),
                ("gains_range", ProjectParamValue::Int(40)),
                ("optional", optional(int_ranged(9, 0, 10))),
                (
                    "range",
                    ProjectParamValue::RangeF32 {
                        min: 0.5,
                        max: 9.0,
                        bounds: None,
                    },
                ),
            ],
            vec![
                ("float", float_ranged(1.0, 0.0, 5.0)),
                ("int", int_ranged(1, 0, 5)),
                ("gains_range", int_ranged(1, 0, 5)),
                ("optional", optional(int_ranged(1, 0, 5))),
                (
                    "range",
                    ProjectParamValue::RangeF32 {
                        min: 2.0,
                        max: 3.0,
                        bounds: Some((1.0, 5.0)),
                    },
                ),
            ],
        );
        assert_eq!(
            combined,
            vec![
                float_ranged(5.0, 0.0, 5.0),
                int_ranged(0, 0, 5),
                int_ranged(5, 0, 5),
                optional(int_ranged(5, 0, 5)),
                ProjectParamValue::RangeF32 {
                    min: 1.0,
                    max: 5.0,
                    bounds: Some((1.0, 5.0)),
                },
            ]
        );
    }

    #[test]
    fn test_combine_resets_ranged_value_when_type_changes() {
        let new = vec![
            ("int_to_float", float_ranged(1.0, 0.0, 10.0)),
            ("float_to_int", ProjectParamValue::Int(1)),
            (
                "float_to_range",
                ProjectParamValue::RangeF32 {
                    min: 1.0,
                    max: 2.0,
                    bounds: Some((0.0, 10.0)),
                },
            ),
        ];
        let combined = combined_values(
            vec![
                ("int_to_float", int_ranged(4, 0, 10)),
                ("float_to_int", float_ranged(4.0, 0.0, 10.0)),
                ("float_to_range", float_ranged(4.0, 0.0, 10.0)),
            ],
            new.clone(),
        );
        let defaults: Vec<_> = new.into_iter().map(|(_, value)| value).collect();
        assert_eq!(combined, defaults);
    }

    #[test]
    fn test_combine_vec_items_follow_a_changed_range_of_their_fields() {
        let item = |x| vec![ProjectParam::new("x", x)];
        let list = |prototype, items: Vec<ProjectParamValue>| {
            let item_struct = |x| ProjectParamValue::Struct(ProjectParamStruct::new(item(x)));
            vec![ProjectParam::new(
                "points",
                ProjectParamValue::Vec(ProjectParamVec::new(
                    item_struct(prototype),
                    items.into_iter().map(item_struct).collect(),
                )),
            )]
        };
        let old = list(
            float_ranged(0.0, 0.0, 10.0),
            vec![float_ranged(3.0, 0.0, 10.0), float_ranged(9.0, 0.0, 10.0)],
        );
        let new = list(float_ranged(1.0, 0.0, 5.0), vec![]);

        let combined = ProjectParamsListWrapper::new_combined(&old, &new);
        let ProjectParamValue::Vec(points) = &combined.list[0].value else {
            panic!("expected a vec");
        };
        // Structs compare by field types only, so the values are read out.
        let xs = |value: &ProjectParamValue| match value {
            ProjectParamValue::Struct(item) => item.fields[0].value.clone(),
            other => other.clone(),
        };
        assert_eq!(xs(&points.item_prototype), float_ranged(1.0, 0.0, 5.0));
        assert_eq!(
            points.items.iter().map(xs).collect::<Vec<_>>(),
            vec![float_ranged(3.0, 0.0, 5.0), float_ranged(5.0, 0.0, 5.0)]
        );
    }
}
