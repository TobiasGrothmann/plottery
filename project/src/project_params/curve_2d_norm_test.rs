#[cfg(test)]
mod tests {
    use super::super::curve_2d_norm::Curve2DNorm;
    use plottery_lib::V2;

    #[test]
    fn test_default() {
        let curve = Curve2DNorm::default();
        assert_eq!(curve.sample(0.0), 0.0);
        assert_eq!(curve.sample(1.0), 1.0);
        assert_eq!(curve.len(), 2);
    }

    #[test]
    fn test_new_clamps_endpoints() {
        let curve = Curve2DNorm::new(-0.5, vec![], 1.5);
        assert_eq!(curve.sample(0.0), 0.0);
        assert_eq!(curve.sample(1.0), 1.0);
    }

    #[test]
    fn test_new_clamps_and_filters_points() {
        let curve = Curve2DNorm::new(
            0.0,
            vec![
                V2::new(-0.1, 0.5),
                V2::new(0.3, 1.5),
                V2::new(0.0, 0.3),
                V2::new(1.0, 0.8),
                V2::new(0.7, -0.2),
            ],
            1.0,
        );

        assert_eq!(curve.len(), 4);
        let points: Vec<_> = curve.iter_points().collect();
        assert!(points.iter().all(|p| p.x >= 0.0 && p.x <= 1.0));
        assert!(points.iter().all(|p| p.y >= 0.0 && p.y <= 1.0));
    }

    #[test]
    fn test_sample_endpoints() {
        let curve = Curve2DNorm::new(0.2, vec![], 0.8);
        assert_eq!(curve.sample(0.0), 0.2);
        assert_eq!(curve.sample(1.0), 0.8);
    }

    #[test]
    fn test_sample_linear_no_points() {
        let curve = Curve2DNorm::new(0.0, vec![], 1.0);
        assert_eq!(curve.sample(0.5), 0.5);
    }

    #[test]
    fn test_sample_with_points() {
        let curve = Curve2DNorm::new(0.0, vec![V2::new(0.5, 0.8)], 1.0);
        assert_eq!(curve.sample(0.25), 0.4);
        assert_eq!(curve.sample(0.75), 0.9);
    }

    #[test]
    fn test_add_point_clamps() {
        let mut curve = Curve2DNorm::default();
        curve.add_point(V2::new(0.5, 2.0)).unwrap();
        assert_eq!(curve.len(), 3);
        let points: Vec<_> = curve.iter_points().collect();
        assert_eq!(points[1].y, 1.0);
    }

    #[test]
    fn test_add_point_rejects_endpoints() {
        let mut curve = Curve2DNorm::default();
        assert!(curve.add_point(V2::new(0.0, 0.5)).is_err());
        assert!(curve.add_point(V2::new(1.0, 0.5)).is_err());
        assert_eq!(curve.len(), 2);
    }

    #[test]
    fn test_remove_point() {
        let mut curve = Curve2DNorm::new(0.0, vec![V2::new(0.3, 0.5), V2::new(0.7, 0.5)], 1.0);
        assert_eq!(curve.len(), 4);
        curve.remove_point_at(1).unwrap(); // Remove first non-endpoint point
        assert_eq!(curve.len(), 3);
        assert!(curve.remove_point_at(10).is_err());
        assert!(curve.remove_point_at(0).is_err()); // Cannot remove endpoint
    }

    #[test]
    fn test_iter_points() {
        let curve = Curve2DNorm::new(0.2, vec![V2::new(0.5, 0.6)], 0.8);
        let points: Vec<_> = curve.iter_points().collect();
        assert_eq!(points.len(), 3);
        assert_eq!(points[0], V2::new(0.0, 0.2));
        assert_eq!(points[1], V2::new(0.5, 0.6));
        assert_eq!(points[2], V2::new(1.0, 0.8));
    }
}
