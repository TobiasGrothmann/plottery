#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use plottery_project::{RollRange, RollRanges};
    use plottery_runtime::{ProjectParam, ProjectParamValue as V, ProjectParamsListWrapper};

    use super::super::variations_state::{roll_ranges, RangeInput};

    #[test]
    fn ranges_fill_up_from_the_own_range_and_plain_numbers_wait_for_both_ends() {
        let size = V::FloatRanged {
            val: 1.0,
            min: 0.5,
            max: 2.0,
        };
        let params = ProjectParamsListWrapper::new(vec![
            ProjectParam::new("free", V::Float(1.0)),
            ProjectParam::new("size", size),
            ProjectParam::new("flag", V::Bool(true)),
        ]);
        let paths: Vec<String> = ["free", "size", "flag"].map(String::from).to_vec();
        let input = |from, to| RangeInput { from, to };
        let roll = |inputs: &[(&str, RangeInput)]| {
            let inputs: BTreeMap<String, RangeInput> = inputs
                .iter()
                .map(|(path, input)| (path.to_string(), *input))
                .collect();
            roll_ranges(&params, &paths, &inputs)
        };
        let range = |path: &str, from, to| -> RollRanges {
            [(path.to_string(), RollRange { from, to })].into()
        };

        assert_eq!(roll(&[]), (RollRanges::new(), vec!["free".to_string()]));
        assert_eq!(
            roll(&[
                ("free", input(Some(1.0), None)),
                ("size", input(None, Some(1.5)))
            ]),
            (range("size", 0.5, 1.5), vec!["free".to_string()])
        );
        assert_eq!(
            roll(&[
                ("free", input(Some(1.0), Some(3.0))),
                ("flag", input(Some(0.0), Some(1.0)))
            ]),
            (range("free", 1.0, 3.0), vec![])
        );
    }
}
