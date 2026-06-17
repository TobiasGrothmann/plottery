#[cfg(test)]
mod test_shape {
    use crate::{Circle, Containment, Path, Rect, Shape, V2};

    #[test]
    fn intersects_circle_circle_touching_and_containment() {
        let c1 = Circle::new(V2::new(0.0, 0.0), 1.0);
        let c2 = Circle::new(V2::new(2.0, 0.0), 1.0); // external tangent
        let c3 = Circle::new(V2::new(0.0, 0.0), 3.0);
        let c4 = Circle::new(V2::new(0.5, 0.0), 1.0); // fully contained in c3, no touch

        assert!(c1.intersects_circle(&c2));
        assert!(c2.intersects_circle(&c1));

        assert!(!c3.intersects_circle(&c4));
        assert!(!c4.intersects_circle(&c3));
    }

    #[test]
    fn intersects_rect_rect_touching_and_containment() {
        let outer = Rect::new(V2::new(0.0, 0.0), V2::new(2.0, 2.0));
        let touching = Rect::new(V2::new(2.0, 0.5), V2::new(3.0, 1.5)); // touches along edge segment
        let inside = Rect::new(V2::new(0.5, 0.5), V2::new(1.5, 1.5)); // strictly inside, no edge touch

        assert!(outer.intersects_rect(&touching));
        assert!(touching.intersects_rect(&outer));

        assert!(!outer.intersects_rect(&inside));
        assert!(!inside.intersects_rect(&outer));
    }

    #[test]
    fn intersects_rect_circle_touching_and_containment() {
        let rect = Rect::new(V2::new(0.0, 0.0), V2::new(2.0, 2.0));
        let touching_circle = Circle::new(V2::new(2.5, 1.0), 0.5); // tangent to right edge
        let circle_inside = Circle::new(V2::new(1.0, 1.0), 0.2); // fully inside rect, no boundary touch
        let circle_contains_rect = Circle::new(V2::new(1.0, 1.0), 5.0); // fully contains rect, no boundary touch

        assert!(rect.intersects_circle(&touching_circle));
        assert!(touching_circle.intersects_rect(&rect));

        assert!(!rect.intersects_circle(&circle_inside));
        assert!(!circle_inside.intersects_rect(&rect));

        assert!(!rect.intersects_circle(&circle_contains_rect));
        assert!(!circle_contains_rect.intersects_rect(&rect));
    }

    #[test]
    fn intersects_path_with_circle_and_rect_touching_and_containment() {
        let square = Path::new_from(vec![
            V2::new(0.0, 0.0),
            V2::new(2.0, 0.0),
            V2::new(2.0, 2.0),
            V2::new(0.0, 2.0),
            V2::new(0.0, 0.0),
        ]);

        let circle_inside = Circle::new(V2::new(1.0, 1.0), 0.2);
        let circle_touching = Circle::new(V2::new(1.0, -0.5), 0.5); // tangent to bottom edge

        let rect_inside = Rect::new(V2::new(0.5, 0.5), V2::new(1.5, 1.5));
        let rect_touching = Rect::new(V2::new(2.0, 0.5), V2::new(3.0, 1.5)); // touches right edge

        assert!(!square.intersects_circle(&circle_inside));
        assert!(!circle_inside.intersects_path(&square));

        assert!(square.intersects_circle(&circle_touching));
        assert!(circle_touching.intersects_path(&square));

        assert!(!square.intersects_rect(&rect_inside));
        assert!(!rect_inside.intersects_path(&square));

        assert!(square.intersects_rect(&rect_touching));
        assert!(rect_touching.intersects_path(&square));
    }

    #[test]
    fn intersects_path_path_touching_and_containment() {
        let outer = Path::new_from(vec![
            V2::new(0.0, 0.0),
            V2::new(2.0, 0.0),
            V2::new(2.0, 2.0),
            V2::new(0.0, 2.0),
            V2::new(0.0, 0.0),
        ]);
        let inner = Path::new_from(vec![
            V2::new(0.5, 0.5),
            V2::new(1.5, 0.5),
            V2::new(1.5, 1.5),
            V2::new(0.5, 1.5),
            V2::new(0.5, 0.5),
        ]);
        let touching = Path::new_from(vec![V2::new(2.0, 0.5), V2::new(2.0, 1.5)]); // colinear with edge

        assert!(!outer.intersects_path(&inner));
        assert!(!inner.intersects_path(&outer));

        assert!(outer.intersects_path(&touching));
        assert!(touching.intersects_path(&outer));
    }

    #[test]
    fn intersects_shape_dispatch() {
        let rect = Shape::Rect(Rect::new(V2::new(0.0, 0.0), V2::new(2.0, 2.0)));
        let touching_circle = Shape::Circle(Circle::new(V2::new(2.5, 1.0), 0.5));
        let inner_path = Shape::Path(Path::new_from(vec![
            V2::new(0.5, 0.5),
            V2::new(1.5, 0.5),
            V2::new(1.5, 1.5),
            V2::new(0.5, 1.5),
            V2::new(0.5, 0.5),
        ]));
        let outer_path = Shape::Path(Path::new_from(vec![
            V2::new(0.0, 0.0),
            V2::new(2.0, 0.0),
            V2::new(2.0, 2.0),
            V2::new(0.0, 2.0),
            V2::new(0.0, 0.0),
        ]));

        assert!(rect.intersects(&touching_circle));
        assert!(touching_circle.intersects(&rect));

        assert!(!outer_path.intersects(&inner_path));
        assert!(!inner_path.intersects(&outer_path));
    }

    #[test]
    fn containment_circle_cases() {
        let outer = Circle::new(V2::new(0.0, 0.0), 3.0);
        let inner = Circle::new(V2::new(0.5, 0.0), 1.0);
        let touching = Circle::new(V2::new(5.0, 0.0), 2.0);
        let far = Circle::new(V2::new(10.0, 0.0), 1.0);

        assert_eq!(outer.contains_circle(&inner), Containment::Full);
        assert_eq!(outer.contains_circle(&touching), Containment::Partial);
        assert_eq!(outer.contains_circle(&far), Containment::None);
    }

    #[test]
    fn containment_rect_cases() {
        let outer = Rect::new(V2::new(0.0, 0.0), V2::new(4.0, 4.0));
        let inner = Rect::new(V2::new(1.0, 1.0), V2::new(2.0, 2.0));
        let touching = Rect::new(V2::new(4.0, 1.0), V2::new(5.0, 2.0));
        let far = Rect::new(V2::new(5.0, 5.0), V2::new(6.0, 6.0));

        assert_eq!(outer.contains_rect(&inner), Containment::Full);
        assert_eq!(outer.contains_rect(&touching), Containment::Partial);
        assert_eq!(outer.contains_rect(&far), Containment::None);
    }

    #[test]
    fn containment_path_open_is_treated_as_closed() {
        let open_square = Path::new_from(vec![
            V2::new(0.0, 0.0),
            V2::new(2.0, 0.0),
            V2::new(2.0, 2.0),
            V2::new(0.0, 2.0),
        ]);

        let inner_circle = Circle::new(V2::new(1.0, 1.0), 0.25);
        let touching_circle = Circle::new(V2::new(1.0, -0.25), 0.25);

        let inner_rect = Rect::new(V2::new(0.5, 0.5), V2::new(1.5, 1.5));

        let inner_open_triangle = Path::new_from(vec![
            V2::new(0.7, 0.7),
            V2::new(1.3, 0.7),
            V2::new(1.0, 1.3),
        ]);

        assert_eq!(
            open_square.contains_circle(&inner_circle),
            Containment::Full
        );
        assert_eq!(
            open_square.contains_circle(&touching_circle),
            Containment::Partial
        );

        assert_eq!(open_square.contains_rect(&inner_rect), Containment::Full);

        assert_eq!(
            open_square.contains_path(&inner_open_triangle),
            Containment::Full
        );
    }

    #[test]
    fn containment_cross_shape_partial_and_none() {
        let rect = Rect::new(V2::new(0.0, 0.0), V2::new(2.0, 2.0));
        let circle = Circle::new(V2::new(3.0, 1.0), 1.5);
        let far_path = Path::new_from(vec![
            V2::new(10.0, 10.0),
            V2::new(11.0, 10.0),
            V2::new(11.0, 11.0),
            V2::new(10.0, 11.0),
        ]);

        assert_eq!(rect.contains_circle(&circle), Containment::Partial);
        assert_eq!(circle.contains_rect(&rect), Containment::Partial);

        assert_eq!(circle.contains_path(&far_path), Containment::None);
        assert_eq!(rect.contains_path(&far_path), Containment::None);
    }

    #[test]
    fn containment_shape_dispatch() {
        let outer_rect = Shape::Rect(Rect::new(V2::new(0.0, 0.0), V2::new(4.0, 4.0)));
        let inner_circle = Shape::Circle(Circle::new(V2::new(2.0, 2.0), 1.0));
        let touching_rect = Shape::Rect(Rect::new(V2::new(4.0, 1.0), V2::new(5.0, 2.0)));
        let far_path = Shape::Path(Path::new_from(vec![
            V2::new(10.0, 10.0),
            V2::new(11.0, 10.0),
            V2::new(11.0, 11.0),
            V2::new(10.0, 11.0),
        ]));

        assert_eq!(outer_rect.contains(&inner_circle), Containment::Full);
        assert_eq!(outer_rect.contains(&touching_rect), Containment::Partial);
        assert_eq!(outer_rect.contains(&far_path), Containment::None);
    }

    #[test]
    fn containment_primitive_contains_shape_helpers() {
        let outer_rect = Rect::new(V2::new(0.0, 0.0), V2::new(4.0, 4.0));
        let outer_circle = Circle::new(V2::new(0.0, 0.0), 5.0);
        let outer_path = Path::new_from(vec![
            V2::new(-3.0, -3.0),
            V2::new(3.0, -3.0),
            V2::new(3.0, 3.0),
            V2::new(-3.0, 3.0),
            V2::new(-3.0, -3.0),
        ]);

        let inner_circle_shape = Shape::Circle(Circle::new(V2::new(1.0, 1.0), 0.5));
        let touching_rect_shape = Shape::Rect(Rect::new(V2::new(4.0, 1.0), V2::new(5.0, 2.0)));
        let far_path_shape = Shape::Path(Path::new_from(vec![
            V2::new(10.0, 10.0),
            V2::new(11.0, 10.0),
            V2::new(11.0, 11.0),
            V2::new(10.0, 11.0),
        ]));

        assert_eq!(
            outer_rect.contains_shape(&inner_circle_shape),
            Containment::Full
        );
        assert_eq!(
            outer_rect.contains_shape(&touching_rect_shape),
            Containment::Partial
        );
        assert_eq!(
            outer_rect.contains_shape(&far_path_shape),
            Containment::None
        );

        assert_eq!(
            outer_circle.contains_shape(&inner_circle_shape),
            Containment::Full
        );
        assert_eq!(
            outer_circle.contains_shape(&touching_rect_shape),
            Containment::Partial
        );
        assert_eq!(
            outer_circle.contains_shape(&far_path_shape),
            Containment::None
        );

        assert_eq!(
            outer_path.contains_shape(&inner_circle_shape),
            Containment::Full
        );
        assert_eq!(
            outer_path.contains_shape(&touching_rect_shape),
            Containment::None
        );
        assert_eq!(
            outer_path.contains_shape(&far_path_shape),
            Containment::None
        );
    }
}
