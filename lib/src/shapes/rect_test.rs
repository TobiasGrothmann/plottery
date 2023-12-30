#[cfg(test)]
mod test_rect {
    use crate::{Plottable, Rect, SampleSettings, V2};

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
        let r = Rect::new(V2::new(1.0, 2.0), V2::new(4.0, 4.0));
        let points: Vec<_> = r.get_points(&SampleSettings::default());
        assert_eq!(points.first().unwrap(), points.last().unwrap()); // is closed
        assert_eq!(points.len(), 5);
    }
}
