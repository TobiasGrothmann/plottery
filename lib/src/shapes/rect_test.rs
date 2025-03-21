#[cfg(test)]
mod test_rect {
    use crate::{
        traits::{ClosestPoint, Scale},
        Plottable, Rect, Rotate90, SampleSettings, V2,
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

        assert_eq!(r.left_mid(), V2::new(1.0, 3.0));
        assert_eq!(r.right_mid(), V2::new(4.0, 3.0));
        assert_eq!(r.top_mid(), V2::new(2.5, 4.0));
        assert_eq!(r.bot_mid(), V2::new(2.5, 2.0));
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
}
