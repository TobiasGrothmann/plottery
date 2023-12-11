#[cfg(test)]
mod test_path {
    use crate::{
        path::path::Path,
        shape::shape::{SampleSettings, Shape},
        vec2::V2,
    };

    #[test]
    fn path() {
        let p = Path::new_from(vec![V2::new(0.0, 0.0), V2::new(1.0, 0.0)]);
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
}
