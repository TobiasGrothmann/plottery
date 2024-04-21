#[cfg(test)]
mod tests {
    use crate::{ProjectParam, ProjectParamValue, ProjectParamsListWrapper};

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
            ProjectParam::new("b", ProjectParamValue::Int(0)),       // should not change value
            ProjectParam::new("a_2_renamed", ProjectParamValue::Float(0.0)), // should be added
            ProjectParam::new("c", ProjectParamValue::Float(1.0)),   // should change type and value
                                                                     // a_2 and a_3 should be removed
        ]);

        let combined = ProjectParamsListWrapper::new_combined(&old.list, &new.list);
        for (i, param) in combined.list.iter().enumerate() {
            assert_eq!(param, &expected.list[i]);
            assert_eq!(param.value, expected.list[i].value);
        }
    }
}
