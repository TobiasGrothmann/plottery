#[cfg(test)]
mod tests {
    use crate::project_param_data_enum::{ProjectParamDataEnum, ProjectParamVariant};
    use crate::{
        project_param::ProjectParam, project_param_struct::ProjectParamStruct,
        project_param_value::ProjectParamValue, project_param_vec::ProjectParamVec,
        project_params_list_wrapper::ProjectParamsListWrapper,
    };

    fn params() -> ProjectParamsListWrapper {
        ProjectParamsListWrapper::new(vec![
            ProjectParam::new("a", ProjectParamValue::Bool(true)),
            ProjectParam::new("b", ProjectParamValue::Float(2.5)).with_section("numbers"),
            ProjectParam::new(
                "c",
                ProjectParamValue::Struct(ProjectParamStruct::new(vec![ProjectParam::new(
                    "items",
                    ProjectParamValue::Vec(ProjectParamVec::new(
                        ProjectParamValue::Int(0),
                        vec![ProjectParamValue::Int(1), ProjectParamValue::Int(2)],
                    )),
                )])),
            )
            .with_description("nested"),
            ProjectParam::new(
                "d",
                ProjectParamValue::DataEnum(ProjectParamDataEnum::new(
                    "Spiral",
                    vec![
                        ProjectParamVariant::new("Blank", vec![]),
                        ProjectParamVariant::new(
                            "Spiral",
                            vec![ProjectParam::new("turns", ProjectParamValue::Float(3.0))],
                        ),
                    ],
                )),
            ),
        ])
    }

    #[test]
    fn round_trip() {
        let bytes = params().to_file_bytes().unwrap();
        assert_eq!(&bytes[..6], b"PLTP\x03\x00");
        let (read, skipped) = ProjectParamsListWrapper::from_file_bytes(&bytes).unwrap();
        assert_eq!(read.list, params().list);
        assert!(skipped.is_empty());
    }

    #[test]
    fn old_version_is_an_error() {
        let mut old = params().to_file_bytes().unwrap();
        old[4] = 1;
        let err = ProjectParamsListWrapper::from_file_bytes(&old).unwrap_err();
        assert!(err.to_string().contains("old format version 1"));
    }

    #[test]
    fn undecodable_records_are_skipped_and_named() {
        let mut bytes = params().to_file_bytes().unwrap();
        // header 6, count 1, then per record: name (1 + 1), type tag 1, value (1 + n), section 1, description 1
        let a_value = 6 + 1 + 2 + 1 + 1;
        let b_type_tag = a_value + 1 + 2 + 2;
        assert_eq!(bytes[a_value], 1);
        bytes[a_value] = 7; // not a bool
        bytes[b_type_tag] = 0x7f; // unknown type

        let (read, skipped) = ProjectParamsListWrapper::from_file_bytes(&bytes).unwrap();
        assert_eq!(skipped, ["a", "b"]);
        assert_eq!(read.list, params().list[2..]);
    }

    #[test]
    fn newer_version_and_garbage_are_errors() {
        let mut newer = params().to_file_bytes().unwrap();
        newer[4] = 4;
        let err = ProjectParamsListWrapper::from_file_bytes(&newer).unwrap_err();
        assert!(err.to_string().contains("format version 4"));

        let mut truncated = params().to_file_bytes().unwrap();
        truncated.truncate(30);
        assert!(ProjectParamsListWrapper::from_file_bytes(&truncated).is_err());
        assert!(ProjectParamsListWrapper::from_file_bytes(&[1, 2, 3]).is_err());
    }
}
