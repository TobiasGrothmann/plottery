#[cfg(test)]
mod test_shape {
    use itertools::Itertools;

    use crate::{Circle, Path, Plottable, Rect, SampleSettings, V2};

    #[test]
    fn oversampling() {
        let r = Rect::new_shape(V2::new(0.0, 0.0), V2::new(2.0, 1.0));
        let sample_settings = SampleSettings {
            points_per_unit: 4.0,
        };
        let points = r.get_points_oversampled(sample_settings);
        assert!(points.len() > 5);
        let distances = points
            .iter()
            .tuple_windows()
            .map(|(from, to)| from.dist(*to));
        let max_distance = distances.clone().fold(0.0, f32::max);

        assert_eq!(max_distance, 1.0 / sample_settings.points_per_unit);

        for (i, point) in points.iter().enumerate() {
            if i == 0 {
                let finds = points.iter().filter(|p| p == point).collect_vec().len();
                assert_eq!(finds, 2);
            } else {
                assert!(!points[(i + 1)..].contains(point))
            }
        }
    }

    #[test]
    fn oversampling_low_sampling() {
        let r = Rect::new_shape(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let sample_settings = SampleSettings {
            points_per_unit: 1.0,
        };
        let points = r.get_points_oversampled(sample_settings);
        assert_eq!(points.len(), 5);
        let max_distance = points
            .iter()
            .tuple_windows()
            .map(|(from, to)| from.dist(*to))
            .fold(0.0, f32::max);

        assert_eq!(max_distance, 1.0)
    }

    #[test]
    fn oversampling_with_distance() {
        let r = Rect::new_shape(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let sample_settings = SampleSettings {
            points_per_unit: 50.0,
        };
        let points = r.get_points_and_dist_oversampled(sample_settings);

        assert!(points.len() > 45);
        assert_eq!(points.last().unwrap().1, 4.0);

        let mut last_dist = -1.0;
        for (_point, dist) in points {
            assert!(dist > last_dist);
            last_dist = dist;
        }
    }

    #[test]
    fn sample_settings() {
        let sample_settings = SampleSettings {
            points_per_unit: 2.0,
        };
        assert_eq!(sample_settings.get_num_points_for_length(2.0), 4);
        assert_eq!(sample_settings.get_num_points_for_length(2.01), 5);
        assert_eq!(sample_settings.get_num_points_for_length(1.0), 2);
        assert_eq!(sample_settings.get_num_points_for_length(0.9), 2);
        assert_eq!(sample_settings.get_num_points_for_length(0.6), 2);
        assert_eq!(sample_settings.get_num_points_for_length(0.3), 1);
    }

    #[test]
    fn masking_0() {
        let mask = Rect::new_shape(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let p = Path::new_shape_from(vec![V2::new(0.5, 0.5), V2::new(1.5, 0.5)]);

        let masked = p.mask_geo(&mask, SampleSettings::default());
        assert_eq!(masked.inside.len(), 1);
        assert_eq!(masked.outside.len(), 1);
    }

    #[test]
    fn masking_1() {
        let mask = Rect::new_shape(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let p = Path::new_shape_from(vec![V2::new(0.5, 0.5), V2::new(1.5, 1.5)]);

        let masked = p.mask_geo(&mask, SampleSettings::default());
        assert_eq!(masked.inside.len(), 1);
        assert_eq!(masked.outside.len(), 1);
    }

    #[test]
    fn masking_2() {
        let mask = Rect::new_shape(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let p = Path::new_shape_from(vec![V2::new(1.0, 1.0), V2::new(1.0, 1.5)]);

        let masked = p.mask_geo(&mask, SampleSettings::default());
        assert_eq!(masked.inside.len(), 0);
        assert_eq!(masked.outside.len(), 1);
    }

    #[test]
    fn masking_3() {
        let mask = Rect::new_shape(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let p = Path::new_shape_from(vec![V2::new(1.0, 1.0), V2::new(0.5, 0.5)]);

        let masked = p.mask_geo(&mask, SampleSettings::default());
        assert_eq!(masked.inside.len(), 1);
        assert_eq!(masked.outside.len(), 0);
    }

    #[test]
    fn masking_4() {
        let mask = Rect::new_shape(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let p = Path::new_shape_from(vec![V2::new(0.5, 1.2), V2::new(1.2, 0.5)]);

        let masked = p.mask_geo(&mask, SampleSettings::default());
        assert_eq!(masked.inside.len(), 1);
        assert_eq!(
            masked.inside.shapes[0]
                .get_points(SampleSettings::default())
                .len(),
            2
        );
        assert_eq!(masked.outside.len(), 2);
    }

    #[test]
    fn masking_5_circle() {
        let center = V2::new(3.0, 3.0);
        let radius = 0.5;
        let mask = Circle::new_shape(center, radius);
        let mut p: Path = Path::new();

        for _ in 0..200 {
            p.push(
                center - V2::xy(radius * 2.0)
                    + V2::new(
                        radius * rand::random::<f32>() * 4.0,
                        radius * rand::random::<f32>() * 4.0,
                    ),
            );
        }

        let masked = p.mask_geo(&mask.clone(), SampleSettings::default());

        for shape_inside in masked.inside.shapes {
            for point in shape_inside.get_points(SampleSettings::default()) {
                assert!(center.dist(point) - 0.001 <= radius);
            }
        }
        for shape_outside in masked.outside.shapes {
            for point in shape_outside.get_points(SampleSettings::default()) {
                assert!(center.dist(point) + 0.001 >= radius);
            }
        }
    }
}
