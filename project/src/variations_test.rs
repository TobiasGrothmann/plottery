#[cfg(test)]
mod tests {
    use std::f32::consts::TAU;

    use plottery_runtime::{
        AngleUnit, ProjectParam, ProjectParamDataEnum, ProjectParamSeed, ProjectParamStruct,
        ProjectParamValue as V, ProjectParamVariant, ProjectParamVec, ProjectParamsListWrapper,
    };
    use rand::{rngs::StdRng, SeedableRng};

    use crate::{
        can_roll_paths, check_rollable, holds_rollable, roll_param_sets, rollable_paths, RollRange,
        RollRanges, ROLL_SEED_RANGE,
    };

    fn ranged(name: &str, val: f32, min: f32, max: f32) -> ProjectParam {
        ProjectParam::new(name, V::FloatRanged { val, min, max })
    }

    fn params() -> ProjectParamsListWrapper {
        ProjectParamsListWrapper::new(vec![
            ProjectParam::new("seed", V::Seed(ProjectParamSeed::new(1))),
            ranged("size", 1.0, 0.5, 2.0),
            ProjectParam::new(
                "count",
                V::IntRanged {
                    val: 3,
                    min: 1,
                    max: 4,
                },
            ),
            ProjectParam::new(
                "spread",
                V::RangeI32 {
                    min: 2,
                    max: 3,
                    bounds: Some((0, 10)),
                },
            ),
            ProjectParam::new("free", V::Float(1.0)),
            ProjectParam::new("title", V::String("hi".to_string())),
            ProjectParam::new(
                "turn",
                V::Angle {
                    rad: 0.0,
                    unit: AngleUnit::Degrees,
                },
            ),
            ProjectParam::new(
                "glow",
                V::Struct(ProjectParamStruct::new(vec![ranged(
                    "radius", 1.0, 0.0, 5.0,
                )])),
            ),
            ProjectParam::new(
                "shape",
                V::DataEnum(ProjectParamDataEnum::new(
                    "Circle",
                    vec![
                        ProjectParamVariant::new("Circle", vec![ranged("radius", 1.0, 1.0, 2.0)]),
                        ProjectParamVariant::new("Square", vec![ranged("side", 5.0, 5.0, 6.0)]),
                    ],
                )),
            ),
        ])
    }

    fn paths(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    fn value(params: &ProjectParamsListWrapper, path: &str) -> V {
        params.clone().value_at_path_mut(path).unwrap().clone()
    }

    #[test]
    fn rolls_stay_in_range_and_reach_both_ends() {
        let start = params();
        let selected = check_rollable(
            &start,
            &paths(&["seed", "count", "spread", "turn", "size"]),
            &RollRanges::new(),
        )
        .unwrap();
        let sets = roll_param_sets(
            &start,
            &selected,
            &RollRanges::new(),
            400,
            &mut StdRng::seed_from_u64(1),
        );
        assert_eq!(sets.len(), 401);
        assert_eq!(sets[0].list, start.list);

        let mut counts = Vec::new();
        for set in &sets[1..] {
            let V::IntRanged { val, .. } = value(set, "count") else {
                panic!()
            };
            counts.push(val);
            let V::RangeI32 { min, max, .. } = value(set, "spread") else {
                panic!()
            };
            assert!(0 <= min && min <= max && max <= 10);
            let V::FloatRanged { val, .. } = value(set, "size") else {
                panic!()
            };
            assert!((0.5..=2.0).contains(&val));
            let V::Angle { rad, .. } = value(set, "turn") else {
                panic!()
            };
            assert!((0.0..TAU).contains(&rad));
            let V::Seed(seed) = value(set, "seed") else {
                panic!()
            };
            assert!(ROLL_SEED_RANGE.contains(&seed.current));
        }
        assert!(counts.contains(&1) && counts.contains(&4));
        assert!(counts.iter().all(|count| (1..=4).contains(count)));
    }

    #[test]
    fn same_seed_gives_same_rolls() {
        let start = params();
        let selected = rollable_paths(&start.list);
        let roll = |seed| -> Vec<_> {
            roll_param_sets(
                &start,
                &selected,
                &RollRanges::new(),
                10,
                &mut StdRng::seed_from_u64(seed),
            )
            .into_iter()
            .map(|set| set.list)
            .collect()
        };
        assert_eq!(roll(7), roll(7));
        assert_ne!(roll(7), roll(8));
    }

    #[test]
    fn rollable_paths_are_the_rollable_leaves_and_pass_the_check() {
        let start = params();
        let rollable = rollable_paths(&start.list);
        assert_eq!(
            rollable,
            paths(&[
                "seed",
                "size",
                "count",
                "spread",
                "turn",
                "glow.radius",
                "shape",
                "shape.radius"
            ])
        );
        assert_eq!(
            check_rollable(&start, &rollable, &RollRanges::new())
                .unwrap()
                .len(),
            rollable.len()
        );
    }

    #[test]
    fn unrollable_params_fail_with_the_rollable_list() {
        let start = params();
        let error = |names: &[&str]| {
            check_rollable(&start, &paths(names), &RollRanges::new())
                .unwrap_err()
                .to_string()
        };
        assert!(error(&["title"])
            .starts_with("'title' is a string and can't be rolled. Rollable: seed, size,"));
        assert!(
            error(&["free"]).starts_with("'free' is a f32 without a #[range] and can't be rolled")
        );
        assert!(error(&["glow"]).starts_with("'glow' is a struct and can't be rolled"));
        assert!(error(&["nope"]).starts_with("unknown param 'nope'"));
        assert!(error(&[]).starts_with("Name the params to roll. Rollable: seed"));
    }

    fn ranges(list: &[(&str, f32, f32)]) -> RollRanges {
        list.iter()
            .map(|(path, from, to)| {
                (
                    path.to_string(),
                    RollRange {
                        from: *from,
                        to: *to,
                    },
                )
            })
            .collect()
    }

    #[test]
    fn numbers_roll_in_their_roll_range() {
        let start = params();
        // to before from, wider than the #[range] of size, between whole numbers for count
        let ranges = ranges(&[
            ("free", 30.0, 10.0),
            ("size", 1.5, 9.0),
            ("count", 1.6, 3.4),
        ]);
        let selected = check_rollable(&start, &paths(&["free", "size", "count"]), &ranges).unwrap();
        let sets = roll_param_sets(
            &start,
            &selected,
            &ranges,
            400,
            &mut StdRng::seed_from_u64(5),
        );
        assert_eq!(sets[0].list, start.list);

        let floats = |path: &str| -> Vec<f32> {
            sets[1..]
                .iter()
                .map(|set| match value(set, path) {
                    V::Float(val) | V::FloatRanged { val, .. } => val,
                    other => panic!("{other:?}"),
                })
                .collect()
        };
        let spans = |values: &[f32], lo: f32, hi: f32| {
            values.iter().all(|val| (lo..=hi).contains(val))
                && values.iter().any(|val| *val < lo + (hi - lo) * 0.1)
                && values.iter().any(|val| *val > hi - (hi - lo) * 0.1)
        };
        assert!(spans(&floats("free"), 10.0, 30.0));
        assert!(spans(&floats("size"), 1.5, 2.0));

        let mut counts: Vec<i32> = sets[1..]
            .iter()
            .map(|set| match value(set, "count") {
                V::IntRanged { val, .. } => val,
                other => panic!("{other:?}"),
            })
            .collect();
        counts.sort();
        counts.dedup();
        assert_eq!(counts, [2, 3]);
    }

    #[test]
    fn a_plain_number_is_rollable_with_a_finite_roll_range() {
        let start = params();
        let free = paths(&["free"]);
        assert!(!rollable_paths(&start.list).contains(&free[0]));
        assert!(can_roll_paths(&start.list).contains(&free[0]));
        assert!(check_rollable(&start, &free, &ranges(&[("free", 0.0, 1.0)])).is_ok());
        // a range of another param or of something that takes none doesn't help
        assert!(check_rollable(&start, &free, &ranges(&[("size", 0.0, 1.0)])).is_err());
        let title = check_rollable(&start, &paths(&["title"]), &ranges(&[("title", 0.0, 1.0)]));
        assert!(title.is_err());
        let error = check_rollable(&start, &free, &ranges(&[("free", 0.0, f32::INFINITY)]))
            .unwrap_err()
            .to_string();
        assert_eq!(
            error,
            "The range to roll 'free' in needs two numbers, not 0 and inf"
        );
    }

    #[test]
    fn check_dedups_and_puts_parents_first() {
        let checked = check_rollable(
            &params(),
            &paths(&["shape.radius", "shape", "seed", "shape"]),
            &RollRanges::new(),
        )
        .unwrap();
        assert_eq!(checked, paths(&["shape", "seed", "shape.radius"]));
    }

    #[test]
    fn a_field_of_one_list_item_rolls_alone() {
        let shape = |radius| {
            V::Struct(ProjectParamStruct::new(vec![ranged(
                "radius", radius, 0.0, 5.0,
            )]))
        };
        let start = ProjectParamsListWrapper::new(vec![ProjectParam::new(
            "shapes",
            V::Vec(ProjectParamVec::new(
                shape(1.0),
                vec![shape(1.0), shape(1.0)],
            )),
        )]);
        assert!(holds_rollable(&value(&start, "shapes")));

        let selected =
            check_rollable(&start, &paths(&["shapes.1.radius"]), &RollRanges::new()).unwrap();
        let sets = roll_param_sets(
            &start,
            &selected,
            &RollRanges::new(),
            20,
            &mut StdRng::seed_from_u64(4),
        );
        let radius = |set, i| value(set, &format!("shapes.{i}.radius")).get_f32().unwrap();
        assert!(sets[1..].iter().all(|set| radius(set, 0) == 1.0));
        assert!(sets[1..]
            .iter()
            .all(|set| (0.0..=5.0).contains(&radius(set, 1))));
        assert!(sets[1..].iter().any(|set| radius(set, 1) != 1.0));

        let error = |path| {
            check_rollable(&start, &paths(&[path]), &RollRanges::new())
                .unwrap_err()
                .to_string()
        };
        assert!(error("shapes").starts_with("'shapes' is a vec<struct> and can't be rolled"));
        assert!(error("shapes.1").starts_with("'shapes.1' is a struct and can't be rolled"));
    }

    #[test]
    fn fields_of_a_variant_that_got_deselected_are_skipped() {
        let start = params();
        let selected = check_rollable(
            &start,
            &paths(&["shape", "shape.radius"]),
            &RollRanges::new(),
        )
        .unwrap();
        let sets = roll_param_sets(
            &start,
            &selected,
            &RollRanges::new(),
            100,
            &mut StdRng::seed_from_u64(3),
        );
        let circle_radius = |set: &ProjectParamsListWrapper| {
            let V::DataEnum(shape) = value(set, "shape") else {
                panic!()
            };
            let circle = shape.variant("Circle").unwrap();
            (
                shape.selected.clone(),
                circle.fields[0].value.get_f32().unwrap(),
            )
        };
        let rolled: Vec<(String, f32)> = sets[1..].iter().map(circle_radius).collect();
        assert!(rolled.iter().any(|(selected, _)| selected == "Square"));
        assert!(rolled
            .iter()
            .filter(|(selected, _)| selected == "Square")
            .all(|(_, radius)| *radius == 1.0));
        assert!(rolled
            .iter()
            .any(|(selected, radius)| selected == "Circle" && *radius != 1.0));
    }
}
