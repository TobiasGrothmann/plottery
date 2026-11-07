#[cfg(test)]
mod test_svg_path_data {
    use crate::{composition::svg_path_data::parse_svg_path_data, Segment, V2};

    #[test]
    fn line_quad_and_close_make_one_closed_spline() {
        let splines = parse_svg_path_data("M0 0 L10 0 Q15 5 10 10 Z");
        assert_eq!(splines.len(), 1);
        let spline = &splines[0];
        assert!(spline.is_closed());
        assert_eq!(spline.len(), 3);
        assert!(matches!(spline.segments()[0], Segment::Line(_)));
        assert!(matches!(spline.segments()[1], Segment::Bezier(_)));
        assert!(matches!(spline.segments()[2], Segment::Line(_)));
        assert_eq!(spline.segments()[2].to(), V2::new(0.0, 0.0));
    }

    #[test]
    fn two_moves_make_two_open_splines() {
        let splines = parse_svg_path_data("M0 0 L1 0 M5 5 L6 6");
        assert_eq!(splines.len(), 2);
        assert!(splines.iter().all(|spline| !spline.is_closed()));
        assert_eq!(splines[1].start_point(), Some(V2::new(5.0, 5.0)));
        assert_eq!(splines[1].end_point(), Some(V2::new(6.0, 6.0)));
    }

    #[test]
    fn smooth_cubic_mirrors_previous_control() {
        let splines = parse_svg_path_data("M0 0 C1 1 2 1 3 0 S5 -1 6 0");
        assert_eq!(splines.len(), 1);
        let Segment::Bezier(second) = &splines[0].segments()[1] else {
            panic!("expected a bezier");
        };
        assert_eq!(second.controls()[1], V2::new(4.0, -1.0));
        assert_eq!(second.to(), V2::new(6.0, 0.0));
    }

    #[test]
    fn relative_line_after_close_starts_at_subpath_start() {
        let splines = parse_svg_path_data("M1 1 l2 0 l0 2 z l-1 0");
        assert_eq!(splines.len(), 2);
        assert!(splines[0].is_closed());
        assert_eq!(splines[1].start_point(), Some(V2::new(1.0, 1.0)));
        assert_eq!(splines[1].end_point(), Some(V2::new(0.0, 1.0)));
    }
}
