#[cfg(test)]
mod test_shape {
    use itertools::Itertools;

    use crate::{
        rect::rect::Rect,
        shape::shape::{SampleSettings, Shape},
        vec2::V2,
        Path,
    };

    #[test]
    fn oversampling() {
        let r = Rect::new(V2::new(0.0, 0.0), V2::new(2.0, 1.0));
        let sample_settings = SampleSettings {
            points_per_unit: 4.0,
        };
        let points = r.get_points_oversampled(&sample_settings);
        assert!(points.len() > 5);
        let distances = points
            .iter()
            .tuple_windows()
            .map(|(from, to)| from.dist(to));
        let max_distance = distances.clone().fold(0.0, |acc, dist| f32::max(acc, dist));

        assert_eq!(max_distance, 1.0 / sample_settings.points_per_unit as f32);

        for (i, point) in points.iter().enumerate() {
            if i == 0 {
                let finds = points.iter().filter(|p| p == &point).collect_vec().len();
                assert_eq!(finds, 2);
            } else {
                assert!(!points[(i + 1)..].contains(point))
            }
        }
    }

    #[test]
    fn oversampling_low_sampling() {
        let r = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let sample_settings = SampleSettings {
            points_per_unit: 1.0,
        };
        let points = r.get_points_oversampled(&sample_settings);
        assert_eq!(points.len(), 5);
        let max_distance = points
            .iter()
            .tuple_windows()
            .map(|(from, to)| from.dist(to))
            .fold(0.0, |acc, dist| f32::max(acc, dist));

        assert_eq!(max_distance, 1.0)
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
    fn masking_1() {
        let mask = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let p = Path::new_from(vec![V2::new(0.5, 0.5), V2::new(1.5, 0.5)]);

        let masked = p.get_masked(Box::new(mask), &SampleSettings::default());
        assert_eq!(masked.inside.len(), 1);
        assert_eq!(masked.outside.len(), 1);
    }

    #[test]
    fn masking_2() {
        let mask = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let p = Path::new_from(vec![V2::new(0.5, 0.5), V2::new(1.5, 1.5)]);

        let masked = p.get_masked(Box::new(mask), &SampleSettings::default());
        assert_eq!(masked.inside.len(), 1);
        assert_eq!(masked.outside.len(), 1);
    }

    #[test]
    fn masking_3() {
        let mask = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let p = Path::new_from(vec![V2::new(1.0, 1.0), V2::new(1.0, 1.5)]);

        let masked = p.get_masked(Box::new(mask), &SampleSettings::default());
        assert_eq!(masked.inside.len(), 0);
        assert_eq!(masked.outside.len(), 1);
    }

    #[test]
    fn masking_4() {
        let mask = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let p = Path::new_from(vec![V2::new(1.0, 1.0), V2::new(0.5, 0.5)]);

        let masked = p.get_masked(Box::new(mask), &SampleSettings::default());
        assert_eq!(masked.inside.len(), 1);
        assert_eq!(masked.outside.len(), 0);
    }

    #[test]
    fn masking_5() {
        let mask = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let p = Path::new_from(vec![V2::new(0.5, 1.2), V2::new(1.2, 0.5)]);

        let masked = p.get_masked(Box::new(mask), &SampleSettings::default());
        assert_eq!(masked.inside.len(), 1);
        assert_eq!(masked.outside.len(), 2);
    }

    #[test]
    fn masking_6() {
        let mask = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let p = Path::new_from(vec![V2::new(0.5, 1.0), V2::new(0.6, 1.0)]);

        let masked = p.get_masked(Box::new(mask), &SampleSettings::default());
        assert_eq!(masked.inside.len(), 0);
        assert_eq!(masked.outside.len(), 1);
    }
}
