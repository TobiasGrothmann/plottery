#[cfg(test)]
mod tests {
    use super::super::curve_2d::{Curve2D, Domain};
    use plottery_lib::V2;

    #[test]
    fn test_domain_default() {
        let domain = Domain::default();
        assert_eq!(domain.x_start, 0.0);
        assert_eq!(domain.x_end, 1.0);
        assert_eq!(domain.y_start, 0.0);
        assert_eq!(domain.y_end, 1.0);
    }

    #[test]
    fn test_domain_new() {
        let domain = Domain::new(0.0, 10.0, 0.0, 100.0);
        assert_eq!(domain.x_start, 0.0);
        assert_eq!(domain.x_end, 10.0);
        assert_eq!(domain.y_start, 0.0);
        assert_eq!(domain.y_end, 100.0);
    }

    #[test]
    fn test_curve2d_default() {
        let curve = Curve2D::default();
        assert_eq!(curve.sample(0.0), 0.0);
        assert_eq!(curve.sample(1.0), 1.0);
        assert_eq!(curve.len(), 2);
    }

    #[test]
    fn test_sample_with_range_mapping() {
        // Curve maps from [0, 10] to [0, 100]
        let curve = Curve2D::new(Domain::new(0.0, 10.0, 0.0, 100.0));

        // Input 5.0 is middle of [0, 10], should map to middle of [0, 100] = 50.0
        assert_eq!(curve.sample(5.0), 50.0);

        // Input 0.0 maps to 0.0
        assert_eq!(curve.sample(0.0), 0.0);

        // Input 10.0 maps to 100.0
        assert_eq!(curve.sample(10.0), 100.0);
    }

    #[test]
    fn test_sample_with_custom_ranges() {
        // Curve with custom endpoint values and ranges
        let mut curve = Curve2D::new(Domain::new(-1.0, 1.0, 10.0, 20.0));
        curve.update_endpoint_norm(true, 0.5);
        curve.update_endpoint_norm(false, 0.5);

        // Input -1.0 (x_start) should map to normalized 0.0, then to 0.5 * (20-10) + 10 = 15.0
        assert_eq!(curve.sample(-1.0), 15.0);

        // Input 1.0 (x_end) should map to normalized 1.0, then to 0.5 * (20-10) + 10 = 15.0
        assert_eq!(curve.sample(1.0), 15.0);

        // Input 0.0 (middle) should also map to 15.0
        assert_eq!(curve.sample(0.0), 15.0);
    }

    #[test]
    fn test_add_point_in_normalized_space() {
        let mut curve = Curve2D::new(Domain::new(0.0, 10.0, 0.0, 100.0));
        curve.add_point_norm(V2::new(0.5, 0.8)).unwrap();
        assert_eq!(curve.len(), 3);

        // Sample at x=5.0 which maps to normalized 0.5
        let y = curve.sample(5.0);
        assert_eq!(y, 80.0); // 0.8 in normalized space = 80 in [0,100]
    }

    #[test]
    fn test_iter_points_normalized() {
        let mut curve = Curve2D::new(Domain::new(0.0, 10.0, 0.0, 100.0));
        curve.add_point_norm(V2::new(0.5, 0.5)).unwrap();

        let points: Vec<_> = curve.iter_points_norm().collect();
        assert_eq!(points.len(), 3);
        // Points are in normalized [0,1] space
        assert_eq!(points[0], V2::new(0.0, 0.0));
        assert_eq!(points[1], V2::new(0.5, 0.5));
        assert_eq!(points[2], V2::new(1.0, 1.0));
    }

    #[test]
    fn test_curve_mut() {
        let mut curve = Curve2D::new(Domain::new(0.0, 10.0, 0.0, 100.0));
        curve.update_endpoint_norm(true, 0.5);
        curve.update_endpoint_norm(false, 0.5);

        // Both endpoints at 0.5 normalized = 50.0 in [0,100]
        assert_eq!(curve.sample(0.0), 50.0);
        assert_eq!(curve.sample(10.0), 50.0);
    }
}
