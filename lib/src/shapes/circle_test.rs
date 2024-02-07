#[cfg(test)]
mod test_circle {
    use std::f32::consts::PI;

    use crate::{
        traits::{normalize::Alignment, Normalize, Scale, Scale2D},
        BoundingBox, Circle, Path, Plottable, SampleSettings, V2,
    };

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
        let c = Circle::new_shape(center.clone(), radius);
        let sample_settings = SampleSettings::default();

        let points = c.get_points(&sample_settings);
        assert!(points.len() > 50); // enough points
        for point in points.iter() {
            assert!((point.dist(&center) - radius).abs() < 0.00001); // radius distance from center
        }
        assert_eq!(points.first().unwrap(), points.last().unwrap()); // is closed
    }

    #[test]
    fn scale() {
        let c = Circle::new(V2::new(1.0, 2.0), 3.0);
        let c_scaled = c.scale(2.0);
        assert_eq!(c_scaled.center, V2::new(2.0, 4.0));
        assert_eq!(c_scaled.radius, 6.0);
    }

    #[test]
    fn scale_shape() {
        let c = Circle::new_shape(V2::new(1.0, 2.0), 3.0);
        let c_scaled = c.scale(2.0);
        assert_eq!(c.length() * 2.0, c_scaled.length());

        let c_p = Path::new_shape_from(c.get_points(&SampleSettings::default()));
        let c_scaled_p = Path::new_shape_from(c_scaled.get_points(&SampleSettings::default()));
        assert!((c_p.length() * 2.0 - c_scaled_p.length()).abs() < 0.001);
    }

    #[test]
    fn scale_shape_2d() {
        let c = Circle::new_shape(V2::new(1.0, 2.0), 3.0);
        let mut c_scaled = c.scale_2d(&V2::new(2.0, 3.0));

        match c_scaled {
            crate::Shape::Circle(_) => panic!("Expected Path, got circle {:?}", c_scaled),
            crate::Shape::Rect(_) => panic!("Expected Path, got rect {:?}", c_scaled),
            crate::Shape::Path(_) => {}
        }
        assert!(c.length() < c_scaled.length());

        c_scaled.scale_2d_mut(&V2::new(1.0 / 2.0, 1.0 / 3.0)); // scale back to original
        assert!((c.length() - c_scaled.length()).abs() < 0.001);
    }

    #[test]
    fn normalize() {
        let c = Circle::new(V2::new(1.0, 2.0), 3.0);
        let target = crate::Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let normalized = c.normalize(&target, Alignment::Center).unwrap();
        let normalized_bounds = normalized.bounding_box().unwrap();

        assert_eq!(normalized_bounds.bl(), V2::new(0.0, 0.0));
        assert_eq!(normalized_bounds.tr(), V2::new(1.0, 1.0));
    }
}
