#[cfg(test)]
mod test_spatial_grid {
    use itertools::Itertools;
    use rand::{rngs::StdRng, Rng, SeedableRng};

    use crate::{SpatialGrid, V2};

    fn assert_matches_brute_force(
        grid: &SpatialGrid<usize>,
        points: &[Option<V2>],
        rng: &mut StdRng,
    ) {
        for _ in 0..200 {
            let query = V2::new(rng.gen_range(-30.0..40.0), rng.gen_range(-30.0..40.0));
            let radius = rng.gen_range(0.0..4.0);
            let alive = || {
                points
                    .iter()
                    .enumerate()
                    .filter_map(|(i, p)| Some((i, (*p)?)))
            };

            let expected = alive()
                .filter(|(_, p)| p.dist_squared(query) <= radius * radius)
                .map(|(i, _)| i)
                .sorted()
                .collect_vec();
            let found = grid
                .within(query, radius)
                .map(|(_, i)| *i)
                .sorted()
                .collect_vec();
            assert_eq!(found, expected, "within({query:?}, {radius})");

            let expected_nearest = alive().map(|(_, p)| p.dist(query)).min_by(f32::total_cmp);
            let found_nearest = grid.nearest(query).map(|(p, _)| p.dist(query));
            assert_eq!(found_nearest, expected_nearest, "nearest({query:?})");
        }
    }

    #[test]
    fn matches_brute_force() {
        let mut rng = StdRng::seed_from_u64(3);
        let random = (0..400)
            .map(|_| V2::new(rng.gen_range(-5.0..15.0), rng.gen_range(-5.0..15.0)))
            .collect_vec();
        let on_a_line = (0..50).map(|i| V2::new(i as f32 * 0.3, 2.0)).collect_vec();
        let all_equal = vec![V2::new(1.0, 1.0); 20];

        for set in [random, on_a_line, all_equal] {
            let collected: SpatialGrid<usize> = set.iter().copied().zip(0..).collect();
            let fixed_cell = {
                let mut grid = SpatialGrid::new(0.37);
                grid.extend(set.iter().copied().zip(0..));
                grid
            };

            for mut grid in [collected, fixed_cell] {
                let mut points = set.iter().copied().map(Some).collect_vec();
                assert_matches_brute_force(&grid, &points, &mut rng);

                for i in (0..points.len()).step_by(2) {
                    assert!(grid.remove(set[i], &i));
                    points[i] = None;
                }
                assert!(!grid.remove(set[0], &0));
                assert_eq!(grid.len(), set.len() / 2);
                assert_matches_brute_force(&grid, &points, &mut rng);

                for i in (1..points.len()).step_by(2) {
                    assert!(grid.remove(set[i], &i));
                }
                assert!(grid.is_empty());
                assert_eq!(grid.nearest(V2::zero()), None);
            }
        }
    }
}
