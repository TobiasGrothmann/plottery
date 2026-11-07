#[cfg(test)]
mod test_hobby {
    use crate::{
        shapes::spline::hobby::{hobby_open, EndCondition},
        Angle, Rotate, Segment, Spline, V2,
    };
    use itertools::Itertools;

    fn parallel(a: V2, b: V2) -> bool {
        let (a, b) = (a.normalize(), b.normalize());
        a.dot(b) > 0.0 && (a.x * b.y - a.y * b.x).abs() < 1e-3
    }

    fn zigzag() -> Vec<V2> {
        vec![
            V2::new(0.0, 0.0),
            V2::new(3.0, 4.0),
            V2::new(7.0, 1.0),
            V2::new(10.0, 6.0),
            V2::new(14.0, 2.0),
        ]
    }

    #[test]
    fn collinear_points_give_collinear_controls() {
        let points = vec![V2::new(0.0, 0.0), V2::new(3.0, 3.0), V2::new(8.0, 8.0)];
        let spline = Spline::through(points);
        for control in spline.control_points() {
            assert!((control.x - control.y).abs() < 1e-4, "{control:?}");
        }
    }

    #[test]
    fn through_passes_through_knots_and_is_g1() {
        let points = zigzag();
        let spline = Spline::through(points.clone());
        assert_eq!(spline.len(), points.len() - 1);
        for (knot, point) in spline.knots().zip(points.iter()) {
            assert_eq!(knot, *point);
        }
        for (a, b) in spline.segments().iter().tuple_windows() {
            assert!(parallel(a.tangent_at_end(), b.tangent_at_start()));
        }
    }

    #[test]
    fn three_points_on_a_circle_give_a_semicircle() {
        // (0,0), (1,1), (2,0) lie on the unit circle around (1,0): the ends must leave vertically
        // with the quarter-circle handle length 4/3 * tan(π/8) ≈ 0.5523
        let spline = Spline::through([V2::new(0.0, 0.0), V2::new(1.0, 1.0), V2::new(2.0, 0.0)]);
        let first = &spline.segments()[0];
        assert!(parallel(first.tangent_at_start(), V2::new(0.0, 1.0)));
        assert!(parallel(first.tangent_at_end(), V2::new(1.0, 0.0)));
        let handles: Vec<f32> = spline.handles().map(|h| h.length()).collect();
        for length in handles {
            assert!((length - 0.5523).abs() < 1e-3, "{length}");
        }
        for t in [0.2, 0.5, 0.8] {
            let error = (spline.point_at(t).dist(V2::new(1.0, 0.0)) - 1.0).abs();
            assert!(error < 3e-3, "{error}");
        }
    }

    #[test]
    fn closed_square_is_four_fold_symmetric() {
        let square = [
            V2::new(-1.0, -1.0),
            V2::new(1.0, -1.0),
            V2::new(1.0, 1.0),
            V2::new(-1.0, 1.0),
        ];
        let spline = Spline::through_closed(square);
        assert!(spline.is_closed());
        assert_eq!(spline.len(), 4);
        let segments = spline.segments();
        for (a, b) in segments.iter().circular_tuple_windows() {
            let (Segment::Bezier(a), Segment::Bezier(b)) = (a, b) else {
                panic!("expected cubics");
            };
            for (ca, cb) in a.controls().iter().zip(b.controls()) {
                assert!(ca.rotate(Angle::quarter_rotation()).dist(*cb) < 1e-3);
            }
        }
    }

    #[test]
    fn direction_end_condition_is_honoured() {
        let points = zigzag();
        let tensions = vec![1.0; points.len()];
        let up = Angle::from_degrees(90.0);
        let left = Angle::from_degrees(180.0);
        let beziers = hobby_open(
            &points,
            &tensions,
            EndCondition::Direction(up),
            EndCondition::Direction(left),
        );
        assert!(parallel(beziers[0].tangent_at(0.0), V2::polar(up, 1.0)));
        assert!(parallel(
            beziers[beziers.len() - 1].tangent_at(1.0),
            V2::polar(left, 1.0)
        ));
    }

    #[test]
    fn higher_tension_gives_shorter_handles() {
        let loose = Spline::through_with_tension(zigzag(), 1.0);
        let tight = Spline::through_with_tension(zigzag(), 2.0);
        for (a, b) in loose.handles().zip(tight.handles()) {
            assert!(b.length() < a.length());
        }
    }
}
