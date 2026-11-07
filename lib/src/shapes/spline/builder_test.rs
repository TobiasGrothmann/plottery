#[cfg(test)]
mod test_builder {
    use crate::{Angle, Path, SampleSettings, Spline, V2};
    use itertools::Itertools;

    fn parallel(a: V2, b: V2) -> bool {
        let (a, b) = (a.normalize(), b.normalize());
        a.dot(b) > 0.0 && (a.x * b.y - a.y * b.x).abs() < 1e-3
    }

    #[test]
    fn lines_equal_path() {
        let points = [V2::new(0.0, 0.0), V2::new(4.0, 1.0), V2::new(2.0, 5.0)];
        let spline = Spline::start(points[0])
            .line_to(points[1])
            .line_to(points[2])
            .build();
        assert_eq!(
            spline.to_path(SampleSettings::default()),
            Path::new_from(points)
        );
    }

    #[test]
    fn arc_around_stays_on_the_circle() {
        let center = V2::new(2.0, 3.0);
        let radius = 5.0;
        let spline = Spline::start(center + V2::new(radius, 0.0))
            .arc_around(center, Angle::from_degrees(90.0))
            .build();
        assert_eq!(spline.len(), 1);
        assert_eq!(spline.end_point().unwrap(), center + V2::new(0.0, radius));
        for t in [0.25, 0.5, 0.75] {
            let error = (spline.point_at(t).dist(center) - radius).abs();
            assert!(error < 1e-3 * radius, "{error}");
        }

        let full = Spline::start(center + V2::new(radius, 0.0))
            .arc_around(center, Angle::from_degrees(-270.0))
            .build();
        assert_eq!(full.len(), 3);
        assert_eq!(full.end_point().unwrap(), center + V2::new(0.0, radius));
    }

    #[test]
    fn arc_to_ends_at_target_with_radius() {
        let to = V2::new(6.0, 0.0);
        let spline = Spline::start(V2::zero()).arc_to(to, 5.0, true).build();
        assert_eq!(spline.end_point().unwrap(), to);
        // clockwise from the origin to the right bulges upwards
        assert!(spline.point_at(0.5).y > 0.0);
        let points = spline.points(SampleSettings::default());
        let center = points.iter().cloned().reduce(|a, b| a + b).unwrap() / points.len() as f32;
        assert!(center.y > 0.0);
    }

    #[test]
    fn tangent_arc_continues_the_tangent() {
        let spline = Spline::start(V2::zero())
            .line_to(V2::new(5.0, 0.0))
            .tangent_arc_to(V2::new(10.0, 5.0))
            .build();
        let segments = spline.segments();
        assert_eq!(segments.len(), 2);
        assert!(parallel(
            segments[0].tangent_at_end(),
            segments[1].tangent_at_start()
        ));
        assert_eq!(spline.end_point().unwrap(), V2::new(10.0, 5.0));
        let error = (spline.segments()[1].point_at(0.5).dist(V2::new(5.0, 5.0)) - 5.0).abs();
        assert!(error < 1e-2, "{error}");
    }

    #[test]
    fn smooth_cubic_mirrors_the_handle() {
        let spline = Spline::start(V2::zero())
            .cubic_to(V2::new(1.0, 2.0), V2::new(3.0, 2.0), V2::new(4.0, 0.0))
            .smooth_cubic_to(V2::new(7.0, -2.0), V2::new(8.0, 0.0))
            .build();
        let second = spline.segments()[1].control_points().collect_vec();
        assert_eq!(second[1], V2::new(5.0, -2.0));
    }

    #[test]
    fn curve_run_between_lines_is_smooth_unless_corner() {
        let build = |corner: bool| {
            let builder = Spline::start(V2::zero())
                .line_to(V2::new(4.0, 0.0))
                .curve_to(V2::new(6.0, 3.0))
                .curve_to(V2::new(9.0, 3.0));
            let builder = if corner { builder.corner() } else { builder };
            builder.line_to(V2::new(13.0, 0.0)).build()
        };
        let smooth = build(false);
        let segments = smooth.segments();
        assert_eq!(segments.len(), 4);
        assert!(parallel(
            segments[0].tangent_at_end(),
            segments[1].tangent_at_start()
        ));
        assert!(parallel(
            segments[2].tangent_at_end(),
            segments[3].tangent_at_start()
        ));

        let cornered = build(true);
        let segments = cornered.segments();
        assert!(parallel(
            segments[0].tangent_at_end(),
            segments[1].tangent_at_start()
        ));
        assert!(!parallel(
            segments[2].tangent_at_end(),
            segments[3].tangent_at_start()
        ));
    }

    #[test]
    fn closed_curve_through_square_is_g1_everywhere() {
        let spline = Spline::start(V2::new(0.0, 0.0))
            .curve_to(V2::new(4.0, 0.0))
            .curve_to(V2::new(4.0, 4.0))
            .curve_to(V2::new(0.0, 4.0))
            .close_curve();
        assert!(spline.is_closed());
        assert_eq!(spline.len(), 4);
        for (a, b) in spline.segments().iter().circular_tuple_windows() {
            assert!(parallel(a.tangent_at_end(), b.tangent_at_start()));
        }
        let points = spline.points(SampleSettings::default());
        assert_eq!(points[0], points[points.len() - 1]);
    }

    #[test]
    fn closed_mixed_spline_keeps_start_and_is_smooth_at_explicit_joins() {
        let spline = Spline::start(V2::new(0.0, 0.0))
            .curve_to(V2::new(4.0, 1.0))
            .curve_to(V2::new(8.0, 0.0))
            .line_to(V2::new(8.0, 5.0))
            .curve_to(V2::new(4.0, 6.0))
            .curve_to(V2::new(0.0, 5.0))
            .close();
        assert!(spline.is_closed());
        assert_eq!(spline.len(), 6);
        assert_eq!(spline.start_point().unwrap(), V2::new(0.0, 0.0));
        for (a, b) in spline.segments().iter().circular_tuple_windows() {
            assert_eq!(a.to(), b.from());
            assert!(parallel(a.tangent_at_end(), b.tangent_at_start()));
        }
    }
}
