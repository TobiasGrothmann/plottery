#[cfg(test)]
mod tests {
    use crate::*;

    #[derive(PlotteryParams, Debug, PartialEq)]
    struct PointItem {
        #[value(1.0)]
        x: f32,
    }

    #[derive(PlotteryParams, Debug, PartialEq)]
    struct Params {
        values: Vec<i32>,
        points: Vec<PointItem>,
    }

    #[test]
    fn vec_defaults_are_empty_with_item_prototypes() {
        let defaults = Params::param_defaults_list();
        assert_eq!(defaults.len(), 2);

        match &defaults[0].value {
            ProjectParamValue::Vec(vec_value) => {
                assert!(matches!(
                    vec_value.item_prototype.as_ref(),
                    ProjectParamValue::Int(0)
                ));
                assert!(vec_value.items.is_empty());
            }
            other => panic!("expected Vec for values, got {:?}", other),
        }

        match &defaults[1].value {
            ProjectParamValue::Vec(vec_value) => {
                assert!(matches!(
                    vec_value.item_prototype.as_ref(),
                    ProjectParamValue::Struct(_)
                ));
                assert!(vec_value.items.is_empty());
            }
            other => panic!("expected Vec for points, got {:?}", other),
        }
    }

    #[test]
    fn vec_new_from_list_reconstructs_values() {
        let params_list = vec![
            ProjectParam::new(
                "values",
                ProjectParamValue::Vec(ProjectParamVec::new(
                    ProjectParamValue::Int(0),
                    vec![ProjectParamValue::Int(5), ProjectParamValue::Int(9)],
                )),
            ),
            ProjectParam::new(
                "points",
                ProjectParamValue::Vec(ProjectParamVec::new(
                    ProjectParamValue::Struct(ProjectParamStruct::new(
                        PointItem::param_defaults_list(),
                    )),
                    vec![ProjectParamValue::Struct(ProjectParamStruct::new(vec![
                        ProjectParam::new("x", ProjectParamValue::Float(2.5)),
                    ]))],
                )),
            ),
        ];

        let parsed = Params::new_from_list(params_list);

        assert_eq!(parsed.values, vec![5, 9]);
        assert_eq!(parsed.points, vec![PointItem { x: 2.5 }]);
    }
}
