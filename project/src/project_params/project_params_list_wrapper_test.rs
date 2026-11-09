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
            ProjectParam::new("b", ProjectParamValue::Int(0)),       // should not change value
            ProjectParam::new("c", ProjectParamValue::Float(1.0)),   // should change type and value
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
}
