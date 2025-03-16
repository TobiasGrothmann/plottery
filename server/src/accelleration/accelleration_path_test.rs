#[cfg(test)]
mod tests {
    use plottery_lib::{FloatInterpolation, Path, Plottable, Rect, SampleSettings, V2};

    use crate::accelleration::accelleration_path::AccellerationPath;

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

        assert_eq!(acc_path.points.len(), 5);

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

        // long segment
        assert_eq!(acc_path.points[1].speed, 1.0);
        assert_eq!(acc_path.points[2].speed, 1.0);
        // right before end
        assert!(acc_path.points[3].speed <= 0.5);
        assert!(acc_path.points[3].speed >= 0.01);
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

        assert_eq!(acc_path.points.len(), 5);

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

        // right after start
        assert!(acc_path.points[1].speed <= 0.5);
        assert!(acc_path.points[1].speed >= 0.01);
        // long segment
        assert_eq!(acc_path.points[2].speed, 1.0);
        assert_eq!(acc_path.points[3].speed, 1.0);
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
}
