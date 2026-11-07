#[cfg(test)]
mod test_bezier {
    use crate::{Bezier, SampleSettings, V2};
    use itertools::Itertools;

    fn bendy() -> Bezier {
        Bezier::cubic(
            V2::new(0.0, 0.0),
            V2::new(0.0, 10.0),
            V2::new(10.0, 10.0),
            V2::new(10.0, 0.0),
        )
    }

    #[test]
    fn ends_and_symmetric_midpoint() {
        let bezier = bendy();
        assert_eq!(bezier.point_at(0.0), bezier.from());
        assert_eq!(bezier.point_at(1.0), bezier.to());
        assert!((bezier.point_at(0.5).x - 5.0).abs() < 1e-5);
        assert_eq!(bezier.degree(), 3);
    }

    #[test]
    fn split_halves_match_point_at() {
        let bezier = bendy();
        let (left, right) = bezier.split(0.3);
        assert_eq!(left.from(), bezier.from());
        assert_eq!(right.to(), bezier.to());
        assert_eq!(left.to(), bezier.point_at(0.3));
        assert_eq!(right.from(), bezier.point_at(0.3));
        assert_eq!(left.point_at(0.5), bezier.point_at(0.15));
        assert_eq!(right.point_at(0.5), bezier.point_at(0.3 + 0.7 * 0.5));
    }

    #[test]
    fn elevated_is_the_same_curve() {
        let quad = Bezier::quadratic(V2::new(0.0, 0.0), V2::new(3.0, 8.0), V2::new(10.0, 1.0));
        let cubic = quad.elevated();
        assert_eq!(cubic.degree(), 3);
        for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
            assert_eq!(quad.point_at(t), cubic.point_at(t));
        }
    }

    #[test]
    fn length_of_collinear_cubic_is_the_chord() {
        let bezier = Bezier::cubic(
            V2::new(0.0, 0.0),
            V2::new(1.0, 0.0),
            V2::new(2.0, 0.0),
            V2::new(3.0, 0.0),
        );
        assert!((bezier.length() - 3.0).abs() < 1e-3);
    }

    #[test]
    fn points_are_evenly_spaced() {
        let bezier = bendy();
        let points = bezier.points(SampleSettings::new(2.0));
        assert_eq!(points[0], bezier.from());
        assert_eq!(points[points.len() - 1], bezier.to());
        let distances: Vec<f32> = points
            .iter()
            .tuple_windows()
            .map(|(a, b)| a.dist(*b))
            .collect();
        let max = distances.iter().cloned().fold(0.0, f32::max);
        let min = distances.iter().cloned().fold(f32::INFINITY, f32::min);
        assert!(max / min < 1.05, "spacing ratio {}", max / min);
    }
}
