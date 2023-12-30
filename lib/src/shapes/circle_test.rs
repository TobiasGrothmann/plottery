#[cfg(test)]
mod test_circle {
    use std::f32::consts::PI;

    use crate::{Circle, Plottable, SampleSettings, V2};

    #[test]
    fn circle_calculations() {
        let center = V2::new(-0.2, 2.0);
        let radius = 3.0;
        let c = Circle::new(center, radius);

        assert_eq!(c.area(), 9.0 * PI);
        assert_eq!(c.circumference(), 6.0 * PI);
    }

    #[test]
    fn circle_points() {
        let center = V2::new(1.0, 2.0);
        let radius = 1.0;
        let c = Circle::new(center.clone(), radius);
        let sample_settings = SampleSettings::default();

        let points = c.get_points(&sample_settings);
        assert!(points.len() > 50); // enough points
        for point in points.iter() {
            assert!((point.dist(&center) - radius).abs() < 0.00001); // radius distance from center
        }
        assert_eq!(points.first().unwrap(), points.last().unwrap()); // is closed
    }
}
