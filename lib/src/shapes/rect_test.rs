#[cfg(test)]
mod test_rect {
    use crate::{
        maths::seed_for_test,
        traits::{ClosestPoint, Scale},
        Angle, Plottable, Rect, Rotate90, SampleSettings, LARGE_EPSILON, V2,
    };

    #[test]
    fn rect_calculations() {
        let r = Rect::new(V2::new(1.0, 2.0), V2::new(4.0, 4.0));

        assert_eq!(r.tl(), V2::new(1.0, 4.0));
        assert_eq!(r.tr(), V2::new(4.0, 4.0));
        assert_eq!(r.br(), V2::new(4.0, 2.0));
        assert_eq!(r.bl(), V2::new(1.0, 2.0));

        assert_eq!(r.center(), V2::new(2.5, 3.0));

        assert_eq!(r.width(), 3.0);
        assert_eq!(r.height(), 2.0);
        assert_eq!(r.area(), 6.0);

        assert_eq!(r.left_center(), V2::new(1.0, 3.0));
        assert_eq!(r.right_center(), V2::new(4.0, 3.0));
        assert_eq!(r.top_center(), V2::new(2.5, 4.0));
        assert_eq!(r.bottom_center(), V2::new(2.5, 2.0));
    }

    #[test]
    fn rect_points() {
        let r = Rect::new_shape(V2::new(1.0, 2.0), V2::new(4.0, 4.0));
        let points: Vec<_> = r.get_points(SampleSettings::default());
        assert_eq!(points.first().unwrap(), points.last().unwrap()); // is closed
        assert_eq!(points.len(), 5);
    }

    #[test]
    fn scale() {
        let r = Rect::new(V2::new(1.0, 2.0), V2::new(4.0, 4.0));
        let r_scaled = r.scale(2.0);
        assert_eq!(r_scaled.bl(), V2::new(2.0, 4.0));
        assert_eq!(r_scaled.tr(), V2::new(8.0, 8.0));
    }

    #[test]
    fn rotate_bl_tr_order() {
        let mut r = Rect::new(V2::new(1.0, 2.0), V2::new(4.0, 4.0));
        r.rotate_180_mut();
        assert!(r.bl().x < r.tr().x);
        assert!(r.bl().y < r.tr().y);

        r.rotate_270_around_mut(V2::new(0.5, 0.1));
        assert!(r.bl().x < r.tr().x);
        assert!(r.bl().y < r.tr().y);

        r.rotate_90_mut();
        assert!(r.bl().x < r.tr().x);
        assert!(r.bl().y < r.tr().y);
    }

    #[test]
    fn closest_point() {
        let r = Rect::new(V2::new(1.0, 2.0), V2::new(4.0, 4.0));

        let point = V2::new(1.0, 2.0);
        assert_eq!(
            r.closest_point(SampleSettings::default(), point),
            Some(point)
        );

        let point = V2::new(1.5, 2.0);
        assert_eq!(
            r.closest_point(SampleSettings::default(), point),
            Some(point)
        );

        let point = V2::new(0.0, 0.0);
        assert_eq!(
            r.closest_point(SampleSettings::default(), point),
            Some(r.bl())
        );

        let point = V2::new(1.2, 3.0);
        assert_eq!(
            r.closest_point(SampleSettings::default(), point),
            Some(V2::new(1.0, 3.0))
        );

        let point = V2::new(0.8, 3.0);
        assert_eq!(
            r.closest_point(SampleSettings::default(), point),
            Some(V2::new(1.0, 3.0))
        );
    }

    #[test]
    fn filled_criss_cross_small_rect_large_pen_skips_inner_fill() {
        let rect = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let path = rect.filled_criss_cross(2.0);
        let points = path.get_points(SampleSettings::default());

        assert_eq!(
            points,
            vec![rect.bl(), rect.tl(), rect.tr(), rect.br(), rect.bl()]
        );
    }

    #[test]
    fn filled_criss_cross_does_not_generate_points_outside_rect() {
        let rect = Rect::new(V2::new(10.0, 20.0), V2::new(11.0, 21.0));
        let path = rect.filled_criss_cross(2.0);

        for point in path.get_points(SampleSettings::default()) {
            assert!(point.x >= rect.bl().x && point.x <= rect.tr().x);
            assert!(point.y >= rect.bl().y && point.y <= rect.tr().y);
        }
    }

    #[test]
    fn helpers() {
        let _rng = seed_for_test(1);
        let r = Rect::new(V2::new(1.0, 2.0), V2::new(4.0, 4.0));
        assert_eq!(r.corners(), [r.bl(), r.tl(), r.tr(), r.br()]);
        assert_eq!(r.edges()[3].to, r.bl());
        assert_eq!(
            r.expand(1.0),
            Rect::new(V2::new(0.0, 1.0), V2::new(5.0, 5.0))
        );
        assert_eq!(r.expand(1.0).shrink(1.0), r);
        assert_eq!(
            Rect::square_around(V2::xy(1.0), 0.5),
            Rect::new(V2::xy(0.5), V2::xy(1.5))
        );
        assert!(r.contains_point(r.random_point()));

        let rotated = r.rotated(Angle::half_rotation());
        assert!(rotated[0].dist(r.tr()) < LARGE_EPSILON);
        assert_eq!(rotated.len(), 5);
    }
}
