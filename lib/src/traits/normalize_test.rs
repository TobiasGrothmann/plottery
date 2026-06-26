#[cfg(test)]
mod test_normalize {
    use crate::{
        traits::{normalize::Alignment, Normalize},
        BoundingBox, Rect, Translate, V2,
    };

    #[test]
    fn normalize_simple_0() {
        let target = Rect::new(V2::new(1.0, 1.0), V2::new(2.0, 2.0));
        let rect = Rect::new(V2::new(-1.0, -1.0), V2::new(5.0, 5.0));

        // all normalizations should result in the target rectangle
        let normalized = rect
            .normalize_inside(&target, Alignment::Left)
            .unwrap()
            .bounding_box()
            .unwrap();
        assert_eq!(normalized.bl(), target.bl());
        assert_eq!(normalized.tr(), target.tr());

        let normalized = rect
            .normalize_inside(&target, Alignment::Top)
            .unwrap()
            .bounding_box()
            .unwrap();
        assert_eq!(normalized.bl(), target.bl());
        assert_eq!(normalized.tr(), target.tr());

        let normalized = rect
            .normalize_inside(&target, Alignment::Right)
            .unwrap()
            .bounding_box()
            .unwrap();
        assert_eq!(normalized.bl(), target.bl());
        assert_eq!(normalized.tr(), target.tr());

        let normalized = rect
            .normalize_inside(&target, Alignment::Bottom)
            .unwrap()
            .bounding_box()
            .unwrap();
        assert_eq!(normalized.bl(), target.bl());
        assert_eq!(normalized.tr(), target.tr());

        let normalized = rect
            .normalize_inside(&target, Alignment::Center)
            .unwrap()
            .bounding_box()
            .unwrap();
        assert_eq!(normalized.bl(), target.bl());
        assert_eq!(normalized.tr(), target.tr());
    }

    #[test]
    fn normalize_simple_1() {
        let target = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let rect = Rect::new(V2::new(0.0, 0.0), V2::new(6.0, 5.0)); // rect ist wider than target
        let normalized = rect.normalize_inside(&target, Alignment::Bottom).unwrap();

        assert_eq!(normalized.bounding_box().unwrap().bl(), target.bl());
        assert_eq!(normalized.bounding_box().unwrap().tr().x, target.tr().x);
        assert_ne!(normalized.bounding_box().unwrap().tr().y, target.tr().y); // result is less high than target
    }

    #[test]
    fn normalize_alignment_tall_shape_left() {
        let target = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let rect = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 2.0)); // rect is taller than target
        let normalized = rect.normalize_inside(&target, Alignment::Left).unwrap();
        let normalized_bounds = normalized.bounding_box().unwrap();

        assert_eq!(normalized_bounds.width(), 0.5);
        assert_eq!(normalized_bounds.height(), target.height());
        assert_eq!(normalized_bounds.bl(), V2::new(0.0, 0.0));
        assert_eq!(normalized_bounds.tr(), V2::new(0.5, 1.0));
    }

    #[test]
    fn normalize_alignment_tall_shape_center() {
        let target = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let rect = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 2.0));
        let normalized = rect.normalize_inside(&target, Alignment::Center).unwrap();
        let normalized_bounds = normalized.bounding_box().unwrap();

        assert_eq!(normalized_bounds.width(), 0.5);
        assert_eq!(normalized_bounds.height(), target.height());
        assert_eq!(normalized_bounds.bl(), V2::new(0.25, 0.0));
        assert_eq!(normalized_bounds.tr(), V2::new(0.75, 1.0));

        let normalized_bottom_bounds = rect
            .normalize_inside(&target, Alignment::Bottom)
            .unwrap()
            .bounding_box()
            .unwrap();
        let normalized_top_bounds = rect
            .normalize_inside(&target, Alignment::Top)
            .unwrap()
            .bounding_box()
            .unwrap();

        // top and bottom should result in centering the shape, too
        assert_eq!(normalized_bounds.bl(), normalized_bottom_bounds.bl());
        assert_eq!(normalized_bounds.tr(), normalized_bottom_bounds.tr());
        assert_eq!(normalized_bounds.bl(), normalized_top_bounds.bl());
        assert_eq!(normalized_bounds.tr(), normalized_top_bounds.tr());
    }

    #[test]
    fn normalize_alignment_tall_shape_right() {
        let target = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let rect = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 2.0));
        let normalized = rect.normalize_inside(&target, Alignment::Right).unwrap();
        let normalized_bounds = normalized.bounding_box().unwrap();

        assert_eq!(normalized_bounds.width(), 0.5);
        assert_eq!(normalized_bounds.height(), target.height());
        assert_eq!(normalized_bounds.bl(), V2::new(0.5, 0.0));
        assert_eq!(normalized_bounds.tr(), V2::new(1.0, 1.0));
    }

    #[test]
    fn noramlize_different_positions() {
        let mut target = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let mut rect = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 2.0));

        let target_offset = V2::new(0.1, -20.0);
        let rect_offset = V2::new(-40.0, 10.0);
        target.translate_mut(target_offset);
        rect.translate_mut(rect_offset);

        let normalized = rect.normalize_inside(&target, Alignment::Right).unwrap();
        let mut normalized_bounds = normalized.bounding_box().unwrap();
        normalized_bounds.translate_mut(target_offset * -1.0);

        assert_eq!(normalized_bounds.width(), 0.5);
        assert_eq!(normalized_bounds.height(), target.height());
        assert_eq!(normalized_bounds.bl(), V2::new(0.5, 0.0));
        assert_eq!(normalized_bounds.tr(), V2::new(1.0, 1.0));
    }

    #[test]
    fn normalize_alignment_wide_shape_bottom() {
        let target = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let rect = Rect::new(V2::new(0.0, 0.0), V2::new(2.0, 1.0));
        let normalized = rect.normalize_inside(&target, Alignment::Bottom).unwrap();
        let normalized_bounds = normalized.bounding_box().unwrap();

        assert_eq!(normalized_bounds.width(), target.width());
        assert_eq!(normalized_bounds.height(), 0.5);
        assert_eq!(normalized_bounds.bl(), V2::new(0.0, 0.0));
        assert_eq!(normalized_bounds.tr(), V2::new(1.0, 0.5));
    }

    #[test]
    fn normalize_alignment_wide_shape_center() {
        let target = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let rect = Rect::new(V2::new(0.0, 0.0), V2::new(2.0, 1.0));
        let normalized = rect.normalize_inside(&target, Alignment::Center).unwrap();
        let normalized_bounds = normalized.bounding_box().unwrap();

        assert_eq!(normalized_bounds.width(), target.width());
        assert_eq!(normalized_bounds.height(), 0.5);
        assert_eq!(normalized_bounds.bl(), V2::new(0.0, 0.25));
        assert_eq!(normalized_bounds.tr(), V2::new(1.0, 0.75));
    }

    #[test]
    fn normalize_alignment_wide_shape_top() {
        let target = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let rect = Rect::new(V2::new(0.0, 0.0), V2::new(2.0, 1.0));
        let normalized = rect.normalize_inside(&target, Alignment::Top).unwrap();
        let normalized_bounds = normalized.bounding_box().unwrap();

        assert_eq!(normalized_bounds.width(), target.width());
        assert_eq!(normalized_bounds.height(), 0.5);
        assert_eq!(normalized_bounds.bl(), V2::new(0.0, 0.5));
        assert_eq!(normalized_bounds.tr(), V2::new(1.0, 1.0));
    }

    #[test]
    fn normalize_mode_around_tall_shape() {
        let target = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let rect = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 2.0));

        let normalized = rect.normalize_around(&target, Alignment::Center).unwrap();
        let normalized_bounds = normalized.bounding_box().unwrap();

        assert_eq!(normalized_bounds.bl(), V2::new(0.0, -0.5));
        assert_eq!(normalized_bounds.tr(), V2::new(1.0, 1.5));
    }

    #[test]
    fn normalize_mode_around_wide_shape() {
        let target = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let rect = Rect::new(V2::new(0.0, 0.0), V2::new(2.0, 1.0));

        let normalized = rect.normalize_around(&target, Alignment::Center).unwrap();
        let normalized_bounds = normalized.bounding_box().unwrap();

        assert_eq!(normalized_bounds.bl(), V2::new(-0.5, 0.0));
        assert_eq!(normalized_bounds.tr(), V2::new(1.5, 1.0));
    }

    #[test]
    fn normalize_around_tall_shape_alignment() {
        let target = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let rect = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 2.0));

        let bottom = rect
            .normalize_around(&target, Alignment::Bottom)
            .unwrap()
            .bounding_box()
            .unwrap();
        let center = rect
            .normalize_around(&target, Alignment::Center)
            .unwrap()
            .bounding_box()
            .unwrap();
        let top = rect
            .normalize_around(&target, Alignment::Top)
            .unwrap()
            .bounding_box()
            .unwrap();

        assert_eq!(bottom.bl(), V2::new(0.0, 0.0));
        assert_eq!(bottom.tr(), V2::new(1.0, 2.0));

        assert_eq!(center.bl(), V2::new(0.0, -0.5));
        assert_eq!(center.tr(), V2::new(1.0, 1.5));

        assert_eq!(top.bl(), V2::new(0.0, -1.0));
        assert_eq!(top.tr(), V2::new(1.0, 1.0));
    }

    #[test]
    fn normalize_around_wide_shape_alignment() {
        let target = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
        let rect = Rect::new(V2::new(0.0, 0.0), V2::new(2.0, 1.0));

        let left = rect
            .normalize_around(&target, Alignment::Left)
            .unwrap()
            .bounding_box()
            .unwrap();
        let center = rect
            .normalize_around(&target, Alignment::Center)
            .unwrap()
            .bounding_box()
            .unwrap();
        let right = rect
            .normalize_around(&target, Alignment::Right)
            .unwrap()
            .bounding_box()
            .unwrap();

        assert_eq!(left.bl(), V2::new(0.0, 0.0));
        assert_eq!(left.tr(), V2::new(2.0, 1.0));

        assert_eq!(center.bl(), V2::new(-0.5, 0.0));
        assert_eq!(center.tr(), V2::new(1.5, 1.0));

        assert_eq!(right.bl(), V2::new(-1.0, 0.0));
        assert_eq!(right.tr(), V2::new(1.0, 1.0));
    }
}
