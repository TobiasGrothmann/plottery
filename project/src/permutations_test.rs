#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::f32::consts::TAU;

    use plottery_runtime::{
        ProjectParam, ProjectParamDataEnum, ProjectParamEnum, ProjectParamOptional,
        ProjectParamSeed, ProjectParamStruct, ProjectParamValue as V, ProjectParamVariant,
        ProjectParamsListWrapper,
    };

    use crate::variations_test::tests::{params, paths, ranged, ranges, value};
    use crate::{permutation_info, permutations, PermutationInfo, RollRanges};

    /// The permutations, checked against the count that doesn't build them.
    fn permute(
        start: &ProjectParamsListWrapper,
        names: &[&str],
        ranges: &RollRanges,
        samples: usize,
        keep: f32,
    ) -> (PermutationInfo, Vec<ProjectParamsListWrapper>) {
        let (info, sets) = permutations(start, &paths(names), ranges, samples, keep);
        assert_eq!(
            permutation_info(start, &paths(names), ranges, samples, keep),
            info
        );
        assert_eq!(info.rolls, sets.len() as u128);
        assert!(!info.estimated);
        if keep == 100.0 {
            assert_eq!(info.grid, info.rolls);
        }
        (info, sets)
    }

    fn float(set: &ProjectParamsListWrapper, path: &str) -> f32 {
        value(set, path).get_f32().unwrap()
    }

    fn int(set: &ProjectParamsListWrapper, path: &str) -> i32 {
        match value(set, path) {
            V::IntRanged { val, .. } => val,
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_grid_reaches_both_ends_without_the_start_params() {
        let start = params();
        let (info, sets) = permute(&start, &["size", "count"], &RollRanges::new(), 4, 100.0);
        assert_eq!(
            info.steps,
            [("size".to_string(), 4), ("count".to_string(), 4)]
        );
        // size 1 and count 3 is roll 0
        assert_eq!(info.grid, 15);
        assert_eq!(info.summary(), "15 permutations");

        let points: Vec<(f32, i32)> = sets
            .iter()
            .map(|set| (float(set, "size"), int(set, "count")))
            .collect();
        assert_eq!(points.first(), Some(&(0.5, 1)));
        assert_eq!(points[1], (0.5, 2));
        assert_eq!(points.last(), Some(&(2.0, 4)));
        assert!(!points.contains(&(1.0, 3)));
        let distinct: BTreeSet<(u32, i32)> = points
            .iter()
            .map(|(size, count)| (size.to_bits(), *count))
            .collect();
        assert_eq!(distinct.len(), 15);
    }

    #[test]
    fn discrete_params_take_all_their_choices_and_seeds_take_the_samples() {
        let start = ProjectParamsListWrapper::new(vec![
            ProjectParam::new("mirror", V::Bool(false)),
            ProjectParam::new(
                "pen",
                V::Enum(ProjectParamEnum::new("Fine", &["Fine", "Bold", "Brush"])),
            ),
            ProjectParam::new("seed", V::Seed(ProjectParamSeed::new(7))),
        ]);
        let (info, sets) = permute(
            &start,
            &["mirror", "pen", "seed"],
            &RollRanges::new(),
            5,
            100.0,
        );
        let steps: Vec<usize> = info.steps.iter().map(|(_, steps)| *steps).collect();
        assert_eq!(steps, [2, 3, 5]);
        assert_eq!(info.grid, 29);

        let seeds: BTreeSet<i32> = sets
            .iter()
            .map(|set| match value(set, "seed") {
                V::Seed(seed) => seed.current,
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(seeds, (7..12).collect());
        let pens: BTreeSet<String> = sets
            .iter()
            .map(|set| match value(set, "pen") {
                V::Enum(pen) => pen.selected,
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(pens.len(), 3);
    }

    #[test]
    fn an_angle_without_a_range_goes_round_once() {
        let (info, sets) = permute(&params(), &["turn"], &RollRanges::new(), 4, 100.0);
        // 0 is the start
        assert_eq!(info.grid, 3);
        let turns: Vec<f32> = sets
            .iter()
            .map(|set| match value(set, "turn") {
                V::Angle { rad, .. } => rad,
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(turns, [TAU / 4.0, TAU / 2.0, TAU * 3.0 / 4.0]);
    }

    #[test]
    fn ints_and_collapsed_ranges_dont_repeat() {
        let start = params();
        // size's roll range lies above its #[range] and ends up as its upper end alone
        let ranges = ranges(&[("size", 5.0, 9.0), ("free", 3.0, 3.0)]);
        let (info, sets) = permute(&start, &["size", "count", "free"], &ranges, 10, 100.0);
        let steps: Vec<usize> = info.steps.iter().map(|(_, steps)| *steps).collect();
        assert_eq!(steps, [1, 4, 1]);
        assert_eq!(info.grid, 4);
        for set in &sets {
            assert_eq!(float(set, "size"), 2.0);
            assert_eq!(float(set, "free"), 3.0);
        }
        let counts: Vec<i32> = sets.iter().map(|set| int(set, "count")).collect();
        assert_eq!(counts, [1, 2, 3, 4]);
    }

    #[test]
    fn a_roll_range_narrows_the_grid() {
        let ranges = ranges(&[("free", 10.0, 30.0), ("size", 0.0, 1.5)]);
        let (_, sets) = permute(&params(), &["free", "size"], &ranges, 3, 100.0);
        let bits = |values: &mut dyn Iterator<Item = f32>| -> BTreeSet<u32> {
            values.map(|v| v.to_bits()).collect()
        };
        let frees = bits(&mut sets.iter().map(|set| float(set, "free")));
        let sizes = bits(&mut sets.iter().map(|set| float(set, "size")));
        assert_eq!(frees, bits(&mut [10.0, 20.0, 30.0].into_iter()));
        assert_eq!(sizes, bits(&mut [0.5, 1.0, 1.5].into_iter()));
    }

    #[test]
    fn ranges_go_from_one_sample_to_a_later_one() {
        let (info, sets) = permute(&params(), &["spread"], &RollRanges::new(), 3, 100.0);
        assert_eq!(info.grid, 3);
        let spreads: Vec<(i32, i32)> = sets
            .iter()
            .map(|set| match value(set, "spread") {
                V::RangeI32 { min, max, .. } => (min, max),
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(spreads, [(0, 5), (0, 10), (5, 10)]);
    }

    #[test]
    fn the_keep_chance_is_the_same_every_time_and_keeps_a_part_of_a_higher_one() {
        let start = params();
        let names = ["size", "count"];
        let kept = |keep| -> Vec<(u32, i32)> {
            let (info, sets) = permute(&start, &names, &RollRanges::new(), 50, keep);
            assert_eq!(info.grid, 200);
            sets.iter()
                .map(|set| (float(set, "size").to_bits(), int(set, "count")))
                .collect()
        };
        let low = kept(30.0);
        assert_eq!(low, kept(30.0));
        assert!((30..=90).contains(&low.len()), "{}", low.len());
        let high = kept(60.0);
        assert!(high.len() > low.len());
        assert!(low.iter().all(|point| high.contains(point)));

        let (info, _) = permute(&start, &names, &RollRanges::new(), 50, 30.0);
        assert_eq!(info.summary(), format!("{} permutations", low.len()));
    }

    #[test]
    fn the_order_the_params_are_named_in_doesnt_matter() {
        let start = params();
        let lists = |names: &[&str]| -> Vec<_> {
            let (_, sets) = permute(&start, names, &RollRanges::new(), 5, 50.0);
            sets.into_iter().map(|set| set.list).collect()
        };
        assert_eq!(
            lists(&["count", "size", "seed"]),
            lists(&["seed", "size", "count"])
        );
    }

    #[test]
    fn a_large_grid_is_rolled_whole() {
        let start = ProjectParamsListWrapper::new(vec![
            ranged("a", 0.5, 0.0, 1.0),
            ranged("b", 0.5, 0.0, 1.0),
            ranged("c", 0.5, 0.0, 1.0),
        ]);
        let (info, sets) = permute(&start, &["a", "b", "c"], &RollRanges::new(), 20, 100.0);
        assert_eq!(info.grid, 8000);
        assert_eq!(sets.len(), 8000);
        let last = sets.last().unwrap();
        assert_eq!(
            (float(last, "a"), float(last, "b"), float(last, "c")),
            (1.0, 1.0, 1.0)
        );
    }

    #[test]
    fn a_grid_too_large_to_count_is_estimated() {
        let start = ProjectParamsListWrapper::new(
            ["a", "b", "c", "d"]
                .map(|name| ranged(name, 0.5, 0.0, 1.0))
                .to_vec(),
        );
        let names = paths(&["a", "b", "c", "d"]);
        let info = permutation_info(&start, &names, &RollRanges::new(), 100, 1.0);
        assert_eq!(info.grid, 100_000_000);
        assert!(info.estimated);
        assert_eq!(info.rolls, 1_000_000);
        assert_eq!(info.summary(), "≈1000000 permutations");
        // all of them are counted without going through them
        let info = permutation_info(&start, &names, &RollRanges::new(), 100, 100.0);
        assert_eq!((info.rolls, info.estimated), (100_000_000, false));
    }

    #[test]
    fn fields_of_another_variant_make_no_extra_rolls() {
        let mut start = params();
        let V::DataEnum(shape) = start.value_at_path_mut("shape").unwrap() else {
            panic!()
        };
        shape.variants.push(ProjectParamVariant::new(
            "Ring",
            vec![ranged("radius", 3.0, 0.0, 10.0)],
        ));
        let (info, sets) = permute(
            &start,
            &["shape", "shape.radius"],
            &RollRanges::new(),
            3,
            100.0,
        );
        // Circle with 3 radii (one is the start), Square, Ring
        assert_eq!(info.grid, 4);
        let shapes: Vec<ProjectParamDataEnum> = sets
            .iter()
            .map(|set| match value(set, "shape") {
                V::DataEnum(shape) => shape,
                other => panic!("{other:?}"),
            })
            .collect();
        let selected = |name: &str| {
            shapes
                .iter()
                .filter(|shape| shape.selected == name)
                .cloned()
                .collect::<Vec<_>>()
        };
        assert_eq!(selected("Square").len(), 1);
        assert_eq!(selected("Ring").len(), 1);
        // the same name as Circle's field, still not Circle's values
        assert_eq!(
            selected("Ring")[0].selected_fields()[0]
                .value
                .get_f32()
                .unwrap(),
            3.0
        );
        let circle_radii: Vec<f32> = selected("Circle")
            .iter()
            .map(|circle| circle.selected_fields()[0].value.get_f32().unwrap())
            .collect();
        assert_eq!(circle_radii, [1.5, 2.0]);
    }

    #[test]
    fn fields_of_an_option_that_is_off_make_no_extra_rolls() {
        let start = ProjectParamsListWrapper::new(vec![ProjectParam::new(
            "glow",
            V::Optional(ProjectParamOptional::new(
                false,
                V::Struct(ProjectParamStruct::new(vec![ranged(
                    "radius", 1.0, 0.0, 5.0,
                )])),
            )),
        )]);
        let (info, sets) = permute(
            &start,
            &["glow", "glow.radius"],
            &RollRanges::new(),
            3,
            100.0,
        );
        // off is the start, on with 3 radii
        assert_eq!(info.grid, 3);
        for set in &sets {
            let V::Optional(glow) = value(set, "glow") else {
                panic!()
            };
            assert!(glow.enabled);
        }
        let radii: Vec<f32> = sets.iter().map(|set| float(set, "glow.radius")).collect();
        assert_eq!(radii, [0.0, 2.5, 5.0]);

        // the option stays off, so its radius doesn't matter
        let (info, sets) = permute(&start, &["glow.radius"], &RollRanges::new(), 3, 100.0);
        assert_eq!((info.grid, sets.len()), (0, 0));
    }
}
