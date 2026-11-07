#[cfg(test)]
mod test_spline {
    use crate::{
        Angle, Bezier, BoundingBox, Line, Plottable, Rotate, SampleSettings, Segment, Spline, V2,
    };
    use itertools::Itertools;

    fn line_and_curve() -> Spline {
        let mut spline = Spline::default();
        spline.push(Segment::Line(Line::new(
            V2::new(0.0, 0.0),
            V2::new(10.0, 0.0),
        )));
        spline.push(Segment::Bezier(Bezier::cubic(
            V2::new(10.0, 0.0),
            V2::new(15.0, 5.0),
            V2::new(15.0, 10.0),
            V2::new(10.0, 15.0),
        )));
        spline
    }

    #[test]
    fn points_have_no_duplicate_join() {
        let points = line_and_curve().points(SampleSettings::new(1.0));
        assert_eq!(points[0], V2::new(0.0, 0.0));
        assert_eq!(points[points.len() - 1], V2::new(10.0, 15.0));
        assert!(points
            .iter()
            .tuple_windows()
            .all(|(a, b)| a.dist(*b) > 1e-4));
    }

    #[test]
    fn point_at_half_of_two_equal_lines_is_the_join() {
        let mut spline = Spline::default();
        spline.push(Segment::Line(Line::new(
            V2::new(0.0, 0.0),
            V2::new(5.0, 0.0),
        )));
        spline.push(Segment::Line(Line::new(
            V2::new(5.0, 0.0),
            V2::new(5.0, 5.0),
        )));
        assert_eq!(spline.point_at(0.5), V2::new(5.0, 0.0));
        assert_eq!(spline.point_at(0.0), V2::new(0.0, 0.0));
        assert_eq!(spline.point_at(1.0), V2::new(5.0, 5.0));
    }

    #[test]
    fn rotation_round_trip_keeps_control_points() {
        let spline = line_and_curve();
        let angle = Angle::from_degrees(33.0);
        let round_trip = spline.rotate(angle).rotate(-angle);
        for (a, b) in spline.control_points().zip(round_trip.control_points()) {
            assert!(a.dist(b) < 1e-4);
        }
    }

    #[test]
    fn bounding_box_contains_all_sampled_points() {
        let spline = line_and_curve();
        let bounds = spline.bounding_box().unwrap().expand(1e-4);
        for point in spline.points(SampleSettings::default()) {
            assert!(bounds.contains_point(point), "{point:?} outside {bounds:?}");
        }
    }
}
