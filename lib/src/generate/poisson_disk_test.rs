#[cfg(test)]
mod test_poisson_disk {
    use itertools::Itertools;

    use crate::{maths::seed_for_test, Plottable, PoissonDisk, Rect, SpatialGrid, V2};

    fn bounds() -> Rect {
        Rect::new(V2::zero(), V2::xy(3.0))
    }

    #[test]
    fn uniform_points_keep_min_dist() {
        let _rng = seed_for_test(1);
        let points = PoissonDisk::new(bounds(), 0.1).points();
        assert!(points.len() > 400, "only {} points", points.len());
        assert!(points.iter().all(|p| bounds().contains_point(*p)));
        for (a, b) in points.iter().tuple_combinations() {
            assert!(a.dist(*b) >= 0.1 - 1e-5, "{a:?} and {b:?} too close");
        }
    }

    #[test]
    fn uniform_points_leave_no_gaps() {
        let _rng = seed_for_test(2);
        let r = 0.1;
        let points = PoissonDisk::new(bounds(), r).points();
        let grid: SpatialGrid<()> = points.into_iter().map(|p| (p, ())).collect();
        let inner = bounds().shrink(2.0 * r);
        let steps = 100;
        for (i, j) in (0..=steps).cartesian_product(0..=steps) {
            let probe = inner.bl() + inner.size() * V2::new(i as f32, j as f32) / steps as f32;
            assert!(grid.any_within(probe, 2.0 * r), "gap at {probe:?}");
        }
    }

    #[test]
    fn variable_spacing_keeps_mean_dist() {
        let _rng = seed_for_test(3);
        let dist = |p: V2| 0.1 + 0.15 * p.x / 3.0;
        let points = PoissonDisk::new(bounds(), 0.1)
            .with_dist_fn(0.25, dist)
            .points();
        assert!(points.len() > 100, "only {} points", points.len());
        for (a, b) in points.iter().tuple_combinations() {
            let needed = (dist(*a) + dist(*b)) * 0.5;
            assert!(a.dist(*b) >= needed - 1e-5, "{a:?} and {b:?} too close");
        }
    }

    #[test]
    fn half_the_spacing_gives_four_times_the_points() {
        let _rng = seed_for_test(4);
        let bounds = Rect::new(V2::zero(), V2::new(6.0, 3.0));
        let points = PoissonDisk::new(bounds, 0.1)
            .with_dist_fn(0.2, |p| if p.x < 3.0 { 0.1 } else { 0.2 })
            .points();
        let (left, right) = points.iter().partition::<Vec<&V2>, _>(|p| p.x < 3.0);
        let ratio = left.len() as f32 / right.len() as f32;
        assert!((3.0..5.0).contains(&ratio), "ratio {ratio}");
    }

    #[test]
    fn same_seed_gives_same_points() {
        let sample = |seed| {
            let _rng = seed_for_test(seed);
            PoissonDisk::new(bounds(), 0.2)
                .with_density_fn(0.5, |p| p.y / 3.0)
                .points()
        };
        assert_eq!(sample(5), sample(5));
    }
}
