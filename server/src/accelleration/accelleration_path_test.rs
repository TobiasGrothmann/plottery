#[cfg(test)]
mod tests {
    use plottery_lib::{FloatInterpolation, Path, Plottable, Rect, SampleSettings, V2};

    use crate::accelleration::accelleration_path::{AccellerationPath, V2Speed};

    fn is_inbetween_on_segment(point: V2, a: V2, b: V2) -> bool {
        if point == a || point == b {
            return false;
        }

        let ab = b - a;
        let ap = point - a;

        // collinear check via 2D cross product
        let cross = ab.x * ap.y - ab.y * ap.x;
        if cross.abs() > 1e-5 {
            return false;
        }

        // point must lie within segment bounds
        let dot = ap.dot(ab);
        let ab_len_sq = ab.dot(ab);
        dot >= -1e-6 && dot <= ab_len_sq + 1e-6
    }

    fn has_inserted_inbetween_for_segment(path_points: &[V2Speed], a: V2, b: V2) -> bool {
        path_points
            .iter()
            .any(|ps| is_inbetween_on_segment(ps.point, a, b))
    }

    fn max_inbetween_speed_on_segment(path_points: &[V2Speed], a: V2, b: V2) -> Option<f32> {
        path_points
            .iter()
            .filter(|ps| is_inbetween_on_segment(ps.point, a, b))
            .map(|ps| ps.speed)
            .max_by(|x, y| x.total_cmp(y))
    }

    fn speed_at_point(path_points: &[V2Speed], point: V2) -> Option<f32> {
        path_points
            .iter()
            .find(|ps| ps.point == point)
            .map(|ps| ps.speed)
    }

    fn assert_path_matches(actual: &[V2Speed], expected: &[(V2, f32)]) {
        assert_eq!(actual.len(), expected.len());

        for (i, (actual_point, (expected_point, expected_speed))) in
            actual.iter().zip(expected.iter()).enumerate()
        {
            assert!(
                (actual_point.point.x - expected_point.x).abs() < 1e-6,
                "point[{}].x mismatch: actual={} expected={}",
                i,
                actual_point.point.x,
                expected_point.x
            );
            assert!(
                (actual_point.point.y - expected_point.y).abs() < 1e-6,
                "point[{}].y mismatch: actual={} expected={}",
                i,
                actual_point.point.y,
                expected_point.y
            );
            assert!(
                (actual_point.speed - expected_speed).abs() < 1e-6,
                "point[{}].speed mismatch: actual={} expected={}",
                i,
                actual_point.speed,
                expected_speed
            );
        }
    }

    #[test]
    fn full_speed_all_segments() {
        let path = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));

        let accell_dist = 0.2;
        let edge_slow_down_power = 0.5;

        let points = path.get_points(SampleSettings::default());
        let acc_path = AccellerationPath::new(&points, accell_dist, edge_slow_down_power);

        assert_eq!(acc_path.points.len(), 13);
        // start and end points didn't change
        assert_eq!(
            acc_path.points.first().unwrap().point,
            points.first().unwrap()
        );
        assert_eq!(
            acc_path.points.last().unwrap().point,
            points.last().unwrap()
        );

        for (i, point_speed) in acc_path.points.iter().enumerate() {
            if i == 0 || i == acc_path.points.len() - 1 {
                // first and last point
                assert_eq!(point_speed.speed, 0.0);
            } else {
                // other points
                assert!(point_speed.speed >= 0.1);
                assert!(point_speed.speed <= 1.0);
            }
        }
    }

    #[test]
    fn long_short() {
        let path = Path::new_from(vec![
            V2::new(0.0, 0.0),
            V2::new(1.0, 0.0),
            V2::new(1.0, 0.1),
        ]);

        let accell_dist = 0.3;
        let edge_slow_down_power = 0.5;

        let points = path.get_points(SampleSettings::default());
        let acc_path = AccellerationPath::new(&points, accell_dist, edge_slow_down_power);

        assert!(acc_path.points.len() >= 5);

        // start and end points didn't change
        assert_eq!(
            acc_path.points.first().unwrap().point,
            points.first().unwrap()
        );
        assert_eq!(
            acc_path.points.last().unwrap().point,
            points.last().unwrap()
        );

        // start and end speed 0.0
        assert_eq!(acc_path.points.first().unwrap().speed, 0.0);
        assert_eq!(acc_path.points.last().unwrap().speed, 0.0);

        // long segment should reach full speed
        let long_seg_peak =
            max_inbetween_speed_on_segment(&acc_path.points, V2::new(0.0, 0.0), V2::new(1.0, 0.0))
                .expect("expected in-between point on long segment");
        assert!((long_seg_peak - 1.0).abs() < 1e-6);

        // short segment should still get an in-between peak if it can exceed both endpoints
        let corner_speed = speed_at_point(&acc_path.points, V2::new(1.0, 0.0)).unwrap();
        let short_seg_peak =
            max_inbetween_speed_on_segment(&acc_path.points, V2::new(1.0, 0.0), V2::new(1.0, 0.1))
                .expect("expected in-between point on short segment");
        assert!(short_seg_peak > corner_speed + 1e-6);
    }

    #[test]
    fn short_long() {
        let path = Path::new_from(vec![
            V2::new(0.0, 0.0),
            V2::new(0.1, 0.0),
            V2::new(0.1, 1.0),
        ]);

        let accell_dist = 0.3;
        let edge_slow_down_power = 0.5;

        let points = path.get_points(SampleSettings::default());
        let acc_path = AccellerationPath::new(&points, accell_dist, edge_slow_down_power);

        assert!(acc_path.points.len() >= 5);

        // start and end points didn't change
        assert_eq!(
            acc_path.points.first().unwrap().point,
            points.first().unwrap()
        );
        assert_eq!(
            acc_path.points.last().unwrap().point,
            points.last().unwrap()
        );

        // start and end speed 0.0
        assert_eq!(acc_path.points.first().unwrap().speed, 0.0);
        assert_eq!(acc_path.points.last().unwrap().speed, 0.0);

        // short segment should still get an in-between peak if it can exceed both endpoints
        let corner_speed = speed_at_point(&acc_path.points, V2::new(0.1, 0.0)).unwrap();
        let short_seg_peak =
            max_inbetween_speed_on_segment(&acc_path.points, V2::new(0.0, 0.0), V2::new(0.1, 0.0))
                .expect("expected in-between point on short segment");
        assert!(short_seg_peak > corner_speed + 1e-6);

        // long segment should reach full speed
        let long_seg_peak =
            max_inbetween_speed_on_segment(&acc_path.points, V2::new(0.1, 0.0), V2::new(0.1, 1.0))
                .expect("expected in-between point on long segment");
        assert!((long_seg_peak - 1.0).abs() < 1e-6);
    }

    #[test]
    fn very_short_segments() {
        let points: Vec<V2> = 0.0.lerp_iter(1.0, 0.01).map(|x| V2::new(x, 0.0)).collect();

        let accell_dist = 0.3;
        let edge_slow_down_power = 0.5;
        let acc_path = AccellerationPath::new(&points, accell_dist, edge_slow_down_power);

        for point in acc_path.points.iter() {
            println!("point: {:?}", point);
        }

        assert!(points.len() + 10 >= acc_path.points.len());
    }

    #[test]
    fn inserts_peak_when_segment_can_accelerate_above_endpoints() {
        // Single segment with both endpoints at speed 0.0.
        // With accell_dist=2.0 and length=1.0, peak speed should be 0.25 at midpoint.
        let points = vec![V2::new(0.0, 0.0), V2::new(1.0, 0.0)];
        let accell_dist = 2.0;
        let edge_slow_down_power = 1.0;

        let acc_path = AccellerationPath::new(&points, accell_dist, edge_slow_down_power);

        assert_eq!(acc_path.points.len(), 3);
        assert_eq!(acc_path.points[0].point, V2::new(0.0, 0.0));
        assert_eq!(acc_path.points[2].point, V2::new(1.0, 0.0));
        assert!((acc_path.points[1].point.x - 0.5).abs() < 1e-6);
        assert!(acc_path.points[1].point.y.abs() < 1e-6);

        assert!(acc_path.points[1].speed > acc_path.points[0].speed + 1e-6);
        assert!(acc_path.points[1].speed > acc_path.points[2].speed + 1e-6);
        assert!((acc_path.points[1].speed - 0.25).abs() < 1e-6);
    }

    #[test]
    fn single_segment_accell_dist_ratio_cases() {
        // Segment from a->b with n = |b-a| = 1.0
        let points = vec![V2::new(0.0, 0.0), V2::new(1.0, 0.0)];
        let edge_slow_down_power = 1.0;

        // accell_dist = 0.2 * n
        let path_02 = AccellerationPath::new(&points, 0.2, edge_slow_down_power);
        assert_path_matches(
            &path_02.points,
            &[
                (V2::new(0.0, 0.0), 0.0),
                (V2::new(0.2, 0.0), 1.0),
                (V2::new(0.8, 0.0), 1.0),
                (V2::new(1.0, 0.0), 0.0),
            ],
        );

        // accell_dist = 0.5 * n
        let path_05 = AccellerationPath::new(&points, 0.5, edge_slow_down_power);
        assert_path_matches(
            &path_05.points,
            &[
                (V2::new(0.0, 0.0), 0.0),
                (V2::new(0.5, 0.0), 1.0),
                (V2::new(1.0, 0.0), 0.0),
            ],
        );

        // accell_dist = 0.8 * n
        let path_08 = AccellerationPath::new(&points, 0.8, edge_slow_down_power);
        assert_path_matches(
            &path_08.points,
            &[
                (V2::new(0.0, 0.0), 0.0),
                (V2::new(0.5, 0.0), 0.625),
                (V2::new(1.0, 0.0), 0.0),
            ],
        );

        // accell_dist = 1.0 * n
        let path_10 = AccellerationPath::new(&points, 1.0, edge_slow_down_power);
        assert_path_matches(
            &path_10.points,
            &[
                (V2::new(0.0, 0.0), 0.0),
                (V2::new(0.5, 0.0), 0.5),
                (V2::new(1.0, 0.0), 0.0),
            ],
        );

        // accell_dist = 1.5 * n
        let path_15 = AccellerationPath::new(&points, 1.5, edge_slow_down_power);
        assert_path_matches(
            &path_15.points,
            &[
                (V2::new(0.0, 0.0), 0.0),
                (V2::new(0.5, 0.0), 1.0 / 3.0),
                (V2::new(1.0, 0.0), 0.0),
            ],
        );
    }

    #[test]
    fn out_and_back_has_symmetric_inserted_peak_points() {
        // Reproduces: (0,0) -> (1,0) -> (0,0)
        // Both segments are equal and should both get in-between peak points.
        let points = vec![V2::new(0.0, 0.0), V2::new(1.0, 0.0), V2::new(0.0, 0.0)];
        let accell_dist = 2.0;
        let edge_slow_down_power = 1.0;

        let acc_path = AccellerationPath::new(&points, accell_dist, edge_slow_down_power);

        assert!(has_inserted_inbetween_for_segment(
            &acc_path.points,
            V2::new(0.0, 0.0),
            V2::new(1.0, 0.0)
        ));
        assert!(has_inserted_inbetween_for_segment(
            &acc_path.points,
            V2::new(1.0, 0.0),
            V2::new(0.0, 0.0)
        ));

        let outbound_peak = acc_path
            .points
            .iter()
            .filter(|ps| ps.point.x > 0.0 && ps.point.x < 1.0)
            .map(|ps| ps.speed)
            .fold(0.0_f32, f32::max);

        // for the return segment (1->0), x is still between 0 and 1, so compare global maxes by halves
        let left_half_peak = acc_path
            .points
            .iter()
            .filter(|ps| ps.point.x <= 0.5 + 1e-6)
            .map(|ps| ps.speed)
            .fold(0.0_f32, f32::max);
        let right_half_peak = acc_path
            .points
            .iter()
            .filter(|ps| ps.point.x >= 0.5 - 1e-6)
            .map(|ps| ps.speed)
            .fold(0.0_f32, f32::max);

        assert!(outbound_peak > 0.0);
        assert!((left_half_peak - right_half_peak).abs() < 1e-4);
    }
}
