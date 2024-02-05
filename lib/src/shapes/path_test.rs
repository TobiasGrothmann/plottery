#[cfg(test)]
mod test_path {
    use crate::{traits::Scale, Path, Plottable, Rect, SampleSettings, V2};

    #[test]
    fn path() {
        let p = Path::new_shape_from(vec![V2::new(0.0, 0.0), V2::new(1.0, 0.0)]);
        let points = p.get_points(&SampleSettings::default());
        assert_eq!(points.len(), 2);
    }

    #[test]
    fn iterators() {
        let p: Path = (0..100)
            .map(|i| V2::new(i as f32 * 0.1, (i as f32).sin()))
            .collect();
        assert_eq!(p.get_points(&SampleSettings::default()).len(), 100);

        for (i, point) in p.get_points(&SampleSettings::default()).iter().enumerate() {
            assert_eq!(point, &V2::new(i as f32 * 0.1, (i as f32).sin()));
        }
    }

    #[test]
    fn shape() {
        let p = Path::new_shape_from(vec![
            V2::new(0.0, 0.0),
            V2::new(1.0, 0.0),
            V2::new(2.0, 1.0),
        ]);
        assert_eq!(p.length(), 1.0 + 2.0_f32.sqrt());
        assert_eq!(p.is_closed(), false);

        let r = Rect::new_shape(V2::new(-1.2, -5.0), V2::new(2.0, 3.1));
        let p = Path::new_shape_from(r.get_points(&SampleSettings::default()));
        assert!((r.length() - p.length()).abs() < 0.00001);
        assert_eq!(p.is_closed(), true);
    }

    #[test]
    fn scale() {
        let p = Path::new_shape_from(vec![V2::new(0.0, 0.1), V2::new(1.0, 0.5)]);
        let p_scaled = p.scale(2.0);
        assert_eq!(p_scaled.get_points(&SampleSettings::default()).len(), 2);
        assert_eq!(
            p_scaled.get_points(&SampleSettings::default())[0],
            V2::new(0.0, 0.2)
        );
        assert_eq!(
            p_scaled.get_points(&SampleSettings::default())[1],
            V2::new(2.0, 1.0)
        );
    }
}
