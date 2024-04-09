#[cfg(test)]
mod test_shape {
    use crate::{traits::ClosestPoint, Path, SampleSettings, V2};

    #[test]
    fn closest_point() {
        let p = Path::new_shape_from(vec![
            V2::new(0.0, 0.0),
            V2::new(3.0, 3.0),
            V2::new(6.0, 0.0),
        ]);

        let point = V2::new(3.0, 3.0);
        assert_eq!(
            p.closest_point(&SampleSettings::default(), &point),
            Some(point)
        );

        let point = V2::new(5.0, 5.0);
        assert_eq!(
            p.closest_point(&SampleSettings::default(), &point),
            Some(V2::new(3.0, 3.0))
        );

        let point = V2::new(1.5, 1.5);
        assert_eq!(
            p.closest_point(&SampleSettings::default(), &point),
            Some(point)
        );

        let point = V2::new(0.0, 3.0); // (1.5, 4.5) has the same distance, but first point is chosen
        assert_eq!(
            p.closest_point(&SampleSettings::default(), &point),
            Some(V2::new(1.5, 1.5))
        );

        let point = V2::new(3.0, 0.0);
        assert_eq!(
            p.closest_point(&SampleSettings::default(), &point),
            Some(V2::new(1.5, 1.5))
        );

        let point = V2::new(100.0, 0.0);
        assert_eq!(
            p.closest_point(&SampleSettings::default(), &point),
            Some(V2::new(6.0, 0.0))
        );
    }
}
