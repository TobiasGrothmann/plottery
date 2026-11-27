#[cfg(test)]
mod tests {
    use plottery_lib::{
        write_versioned, ColorRgb, Easing, NoiseSettings, NoiseType, Rect, V2i, V2,
    };
    use serde::Serialize;

    use crate::project_param_data_enum::{ProjectParamDataEnum, ProjectParamVariant};
    use crate::project_params::params_file_v2::{
        DataEnumV2, NoiseScaleV2, NoiseTypeV2, NoiseV2, OptionalV2, ParamV2, StructV2, ValueV2,
        VariantV2, VecV2,
    };
    use crate::ParamField;
    use crate::{
        project_param::ProjectParam, project_param_optional::ProjectParamOptional,
        project_param_struct::ProjectParamStruct, project_param_value::ProjectParamValue as V,
        project_param_vec::ProjectParamVec, project_params_list_wrapper::ProjectParamsListWrapper,
        AngleUnit,
    };

    /// A record as format version 2 wrote it.
    #[derive(Serialize)]
    struct RecordV2 {
        name: String,
        type_tag: u32,
        value: Vec<u8>,
        section: Option<String>,
        description: Option<String>,
    }

    fn record(name: &str, value: &ValueV2) -> RecordV2 {
        let binary = postcard::to_stdvec(value).unwrap();
        let (type_tag, value) = postcard::take_from_bytes::<u32>(&binary).unwrap();
        RecordV2 {
            name: name.to_string(),
            type_tag,
            value: value.to_vec(),
            section: None,
            description: None,
        }
    }

    fn file_v2(records: &[RecordV2]) -> Vec<u8> {
        write_versioned(b"PLTP", 2, &records).unwrap()
    }

    fn field_v2(name: &str, value: ValueV2) -> ParamV2 {
        ParamV2 {
            name: name.to_string(),
            value,
            section: Some("look".to_string()),
            description: Some("doc".to_string()),
        }
    }

    fn field(name: &str, value: V) -> ProjectParam {
        ProjectParam::new(name, value)
            .with_section("look")
            .with_description("doc")
    }

    fn angle_ranged_v2() -> ValueV2 {
        ValueV2::AngleRanged {
            rad: 0.5,
            unit: AngleUnit::Degrees,
            min: 0.0,
            max: 1.0,
        }
    }

    fn angle_ranged() -> V {
        V::AngleRanged {
            rad: 0.5,
            unit: AngleUnit::Degrees,
            min: 0.0,
            max: 1.0,
        }
    }

    #[test]
    fn values_after_the_removed_noise_come_back_as_they_were_also_nested() {
        let color = ColorRgb::new(0.1, 0.2, 0.3);
        let rect = Rect::new(V2::new(0.0, 0.0), V2::new(2.0, 3.0));
        let easing = Easing::default();
        let bytes = file_v2(&[
            record("size", &ValueV2::Float(2.5)),
            record("easing", &ValueV2::Easing(easing)),
            record("ink", &ValueV2::Color(color)),
            record(
                "glow",
                &ValueV2::Struct(StructV2 {
                    fields: vec![
                        field_v2("tint", ValueV2::Color(color)),
                        field_v2("turn", angle_ranged_v2()),
                    ],
                }),
            ),
            record(
                "shape",
                &ValueV2::DataEnum(DataEnumV2 {
                    selected: "Box".to_string(),
                    variants: vec![
                        VariantV2 {
                            name: "Dot".to_string(),
                            fields: vec![],
                        },
                        VariantV2 {
                            name: "Box".to_string(),
                            fields: vec![field_v2("frame", ValueV2::Rect(rect))],
                        },
                    ],
                }),
            ),
            record(
                "cells",
                &ValueV2::Vec(VecV2 {
                    item_prototype: Box::new(ValueV2::V2i(V2i::new(0, 0))),
                    items: vec![ValueV2::V2i(V2i::new(1, 2))],
                }),
            ),
            record(
                "fade",
                &ValueV2::Optional(OptionalV2 {
                    enabled: true,
                    value: Box::new(ValueV2::Percent {
                        val: 40.0,
                        min: 0.0,
                        max: 100.0,
                    }),
                }),
            ),
        ]);

        let (read, skipped) = ProjectParamsListWrapper::from_file_bytes(&bytes).unwrap();
        assert!(skipped.is_empty());
        let expected = vec![
            ProjectParam::new("size", V::Float(2.5)),
            ProjectParam::new("easing", V::Easing(easing)),
            ProjectParam::new("ink", V::Color(color)),
            ProjectParam::new(
                "glow",
                V::Struct(ProjectParamStruct::new(vec![
                    field("tint", V::Color(color)),
                    field("turn", angle_ranged()),
                ])),
            ),
            ProjectParam::new(
                "shape",
                V::DataEnum(ProjectParamDataEnum::new(
                    "Box",
                    vec![
                        ProjectParamVariant::new("Dot", vec![]),
                        ProjectParamVariant::new("Box", vec![field("frame", V::Rect(rect))]),
                    ],
                )),
            ),
            ProjectParam::new(
                "cells",
                V::Vec(ProjectParamVec::new(
                    V::V2i(V2i::new(0, 0)),
                    vec![V::V2i(V2i::new(1, 2))],
                )),
            ),
            ProjectParam::new(
                "fade",
                V::Optional(ProjectParamOptional::new(
                    true,
                    V::Percent {
                        val: 40.0,
                        min: 0.0,
                        max: 100.0,
                    },
                )),
            ),
        ];
        assert_eq!(read.list, expected);

        // saved again, it is a current file
        let saved = read.to_file_bytes().unwrap();
        assert_eq!(&saved[..6], b"PLTP\x03\x00");
        let (again, _) = ProjectParamsListWrapper::from_file_bytes(&saved).unwrap();
        assert_eq!(again.list, expected);
    }

    #[test]
    fn a_noise_param_becomes_the_noise_settings_struct() {
        let noise = ValueV2::Noise(NoiseV2 {
            noise_type: NoiseTypeV2::Worley,
            frequency: 3.5,
            octaves: 4,
            scale: NoiseScaleV2::PerPaper,
        });
        let bytes = file_v2(&[record("grain", &noise)]);
        let (read, skipped) = ProjectParamsListWrapper::from_file_bytes(&bytes).unwrap();
        assert!(skipped.is_empty());
        let settings = NoiseSettings::from_param_value(&read.list[0].value).unwrap();
        assert_eq!(settings, NoiseSettings::new(NoiseType::Worley, 3.5, 4));
        // the shape a project's noise settings have, so its saved values merge in
        assert!(read.list[0]
            .value
            .same_schema(&NoiseSettings::default_param_value()));
    }
}
