#[cfg(test)]
mod test_path {
    use crate::{
        geometry::TransformMatrix,
        traits::{transform::Transform, ClosestPoint, Scale},
        Angle, BoundingBox, Circle, Path, Plottable, Rect, SampleSettings, Translate,
        LARGE_EPSILON, V2,
    };
    use std::f32::consts::PI;

    #[test]
    fn path() {
        let p = Path::new_shape_from(vec![V2::new(0.0, 0.0), V2::new(1.0, 0.0)]);
        let points = p.get_points(SampleSettings::default());
        assert_eq!(points.len(), 2);
    }

    #[test]
    fn iterators() {
        let p: Path = (0..100)
            .map(|i| V2::new(i as f32 * 0.1, (i as f32).sin()))
            .collect();
        assert_eq!(p.get_points(SampleSettings::default()).len(), 100);

        for (i, point) in p.get_points(SampleSettings::default()).iter().enumerate() {
            assert_eq!(point, V2::new(i as f32 * 0.1, (i as f32).sin()));
        }
    }

    #[test]
    fn from_vec_v2_into_path() {
        let points = vec![V2::new(0.0, 0.0), V2::new(1.0, 2.0), V2::new(3.0, 5.0)];
        let path: Path = points.clone().into();

        assert_eq!(path.get_points_ref(), points.as_slice());
    }

    #[test]
    fn from_slice_v2_into_path() {
        let points = [V2::new(0.0, 0.0), V2::new(1.0, 2.0), V2::new(3.0, 5.0)];
        let path: Path = points.as_slice().into();

        assert_eq!(path.get_points_ref(), points.as_slice());
    }

    #[test]
    fn from_iterator_ref_v2_into_path() {
        let points = [V2::new(0.0, 0.0), V2::new(1.0, 2.0), V2::new(3.0, 5.0)];
        let path: Path = points.iter().collect();

        assert_eq!(path.get_points_ref(), points.as_slice());
    }

    #[test]
    fn shape() {
        let p = Path::new_shape_from(vec![
            V2::new(0.0, 0.0),
            V2::new(1.0, 0.0),
            V2::new(2.0, 1.0),
        ]);
        assert_eq!(p.length(), 1.0 + 2.0_f32.sqrt());
        assert!(!p.is_closed());

        let r = Rect::new_shape(V2::new(-1.2, -5.0), V2::new(2.0, 3.1));
        let p = Path::new_shape_from(r.get_points(SampleSettings::default()));
        assert!((r.length() - p.length()).abs() < LARGE_EPSILON);
        assert!(p.is_closed());
    }

    #[test]
    fn bounding_box() {
        let p = Path::new_from(vec![
            V2::new(3.0, -1.0),
            V2::new(-2.0, 5.0),
            V2::new(1.0, 2.0),
            V2::new(-1.0, -4.0),
        ]);
        assert_eq!(
            p.bounding_box(),
            Some(Rect::new(V2::new(-2.0, -4.0), V2::new(3.0, 5.0)))
        );

        let empty = Path::new();
        assert_eq!(empty.bounding_box(), None);
    }

    #[test]
    fn scale() {
        let p = Path::new_shape_from(vec![V2::new(0.0, 0.1), V2::new(1.0, 0.5)]);
        let p_scaled = p.scale(2.0);
        assert_eq!(p_scaled.get_points(SampleSettings::default()).len(), 2);
        assert_eq!(
            p_scaled.get_points(SampleSettings::default())[0],
            V2::new(0.0, 0.2)
        );
        assert_eq!(
            p_scaled.get_points(SampleSettings::default())[1],
            V2::new(2.0, 1.0)
        );
    }

    #[test]
    fn transform() {
        let p = Path::new_shape_from(vec![V2::new(0.0, 0.1), V2::new(1.0, 0.5)]);

        let scale = TransformMatrix::scale_2d(V2::xy(2.0));
        let translate = TransformMatrix::translate(V2::new(1.0, 0.0));
        let combined = TransformMatrix::combine_transforms(&[scale, translate]);

        let transformed = p.transform(&combined);
        let transformed_2 = p.scale(2.0).translate(V2::new(1.0, 0.0));

        assert_eq!(
            transformed.get_points(SampleSettings::default()),
            transformed_2.get_points(SampleSettings::default())
        );
    }

    #[test]
    fn closest_point() {
        let p = Path::new_from(vec![
            V2::new(0.0, 0.0),
            V2::new(3.0, 3.0),
            V2::new(6.0, 0.0),
        ]);

        let point = V2::new(3.0, 3.0);
        assert_eq!(
            p.closest_point(SampleSettings::default(), point),
            Some(point)
        );

        let point = V2::new(5.0, 5.0);
        assert_eq!(
            p.closest_point(SampleSettings::default(), point),
            Some(V2::new(3.0, 3.0))
        );

        let point = V2::new(1.5, 1.5);
        assert_eq!(
            p.closest_point(SampleSettings::default(), point),
            Some(point)
        );

        let point = V2::new(0.0, 3.0); // (1.5, 4.5) has the same distance, but first point is chosen
        assert_eq!(
            p.closest_point(SampleSettings::default(), point),
            Some(V2::new(1.5, 1.5))
        );

        let point = V2::new(3.0, 0.0);
        assert_eq!(
            p.closest_point(SampleSettings::default(), point),
            Some(V2::new(1.5, 1.5))
        );

        let point = V2::new(100.0, 0.0);
        assert_eq!(
            p.closest_point(SampleSettings::default(), point),
            Some(V2::new(6.0, 0.0))
        );
    }

    #[test]
    fn chaikins() {
        let p = Path::new_from(vec![
            V2::new(0.0, 0.0),
            V2::new(1.0, 0.0),
            V2::new(1.0, 1.0),
        ]);

        let p_rounded = p.rounded_chaikins(1);
        let points = p_rounded.get_points(SampleSettings::default());
        assert_eq!(points.len(), 4);
        assert_eq!(points[0], V2::new(0.0, 0.0));
        assert_eq!(points[1], V2::new(0.75, 0.0));
        assert_eq!(points[2], V2::new(1.0, 0.25));
        assert_eq!(points[3], V2::new(1.0, 1.0));
        assert!(!p_rounded.is_closed());
    }

    #[test]
    fn chaikins_closed() {
        let p = Path::new_from(vec![
            V2::new(0.0, 0.0),
            V2::new(1.0, 0.0),
            V2::new(1.0, 1.0),
            V2::new(0.0, 1.0),
            V2::new(0.0, 0.0),
        ]);

        let p_rounded = p.rounded_chaikins(1);
        let points = p_rounded.get_points(SampleSettings::default());
        assert_eq!(points.len(), 9);
        assert_eq!(points[0], V2::new(0.25, 0.0));
        assert_eq!(points[1], V2::new(0.75, 0.0));
        assert_eq!(points[2], V2::new(1.0, 0.25));
        assert_eq!(points[3], V2::new(1.0, 0.75));
        assert_eq!(points[4], V2::new(0.75, 1.0));
        assert_eq!(points[5], V2::new(0.25, 1.0));
        assert_eq!(points[6], V2::new(0.0, 0.75));
        assert_eq!(points[7], V2::new(0.0, 0.25));
        assert!(p_rounded.is_closed());
    }

    #[test]
    fn simplify() {
        let center = V2::new(0.5, 2.0);
        let radius = 5.0;

        let c = Circle::new(center, radius);
        let points = c.get_points(SampleSettings::default());
        let p = Path::new_from(points.clone());

        let p_simplified1 = p.reduce_points(0.01);
        let points_simplified = p_simplified1.get_points(SampleSettings::default());

        let p_simplified2 = p.reduce_points(0.5);
        let points_simplified2 = p_simplified2.get_points(SampleSettings::default());

        assert!(points.len() > points_simplified.len());
        assert!(points_simplified.len() > points_simplified2.len());
        assert!(points.len() > points_simplified2.len());

        for point in points_simplified.iter() {
            assert!(points.contains(point));
            assert!((point.dist(center) - radius).abs() < LARGE_EPSILON);
        }
        for point in points_simplified2.iter() {
            assert!(points.contains(point));
            assert!((point.dist(center) - radius).abs() < LARGE_EPSILON);
        }
    }

    #[test]
    fn reduce_points_circle_path_stays_closed_and_on_radius() {
        let center = V2::new(0.5, 2.0);
        let radius = 5.0;

        let circle = Circle::new(center, radius);
        let original_points = circle.get_points(SampleSettings::default());
        let path = Path::new_from(original_points.clone());

        let reduced = path.reduce_points(0.25);
        let reduced_points = reduced.get_points(SampleSettings::default());

        assert!(reduced_points.len() < original_points.len());
        assert_eq!(reduced_points.first(), reduced_points.last());

        for point in reduced_points {
            assert!((point.dist(center) - radius).abs() < LARGE_EPSILON);
        }
    }

    #[test]
    fn arc_point_density_follows_sample_settings() {
        let center = V2::new(1.0, -2.0);
        let radius = 2.0;
        let sample = SampleSettings::new(10.0);
        let arc = Path::arc(
            Angle::zero(),
            Angle::quarter_rotation(),
            center,
            radius,
            sample,
        );

        // quarter circle: length = PI / 2 * radius = PI, so about 32 segments
        let expected = sample.get_num_points_for_length(PI * radius / 2.0) as usize + 1;
        assert_eq!(arc.get_points_ref().len(), expected);
        assert!(expected < 40);

        assert_eq!(*arc.get_start().unwrap(), center + V2::new(radius, 0.0));
        assert_eq!(*arc.get_end().unwrap(), center + V2::new(0.0, radius));
        for p in arc.iter() {
            assert!((p.dist(center) - radius).abs() < LARGE_EPSILON);
        }
    }

    #[test]
    fn slice_like_access() {
        let mut path = Path::line(V2::new(0.0, 0.0), V2::new(1.0, 0.0));
        path.extend([V2::new(1.0, 1.0)]);
        path.extend(&[V2::new(0.0, 1.0)]);

        assert_eq!(path.len(), 4);
        assert_eq!(path[1], V2::new(1.0, 0.0));
        assert_eq!(path.last(), Some(&V2::new(0.0, 1.0)));
        assert_eq!(path.windows(2).count(), 3);
        assert_eq!((&path).into_iter().count(), 4);

        let closed = path.closed();
        assert_eq!(closed.len(), 5);
        assert_eq!(closed.first(), closed.last());
        assert!(closed.is_closed());
    }

    #[test]
    fn arc_between_takes_the_shorter_way() {
        let center = V2::zero();
        let a = V2::new(1.0, 0.0);
        let b = V2::new(0.0, -1.0);
        let arc = Path::arc_between(center, 1.0, a, b, SampleSettings::new(10.0));
        assert_eq!(*arc.first().unwrap(), a);
        assert!(arc.last().unwrap().dist(b) < LARGE_EPSILON);
        assert!((arc.length() - PI / 2.0).abs() < 0.01);
        // never passes through the far side
        assert!(arc
            .iter()
            .all(|p| p.x >= -LARGE_EPSILON && p.y <= LARGE_EPSILON));

        let rect_path = Rect::new(V2::zero(), V2::xy(1.0)).to_path(SampleSettings::default());
        assert_eq!(rect_path.len(), 5);
        assert!(rect_path.is_closed());
    }

    #[test]
    fn contains_point() {
        let p = Path::new_from(vec![
            V2::new(0.0, 0.0),
            V2::new(1.0, 0.0),
            V2::new(1.0, 1.0),
            V2::new(0.0, 1.0),
            V2::new(0.0, 0.0),
        ]);

        assert!(p.contains_point(V2::new(0.5, 0.5)));
        assert!(p.contains_point(V2::new(0.9, 0.9)));
        assert!(p.contains_point(V2::new(0.1, 0.1)));

        assert!(!p.contains_point(V2::new(0.5, 1.5)));
        assert!(!p.contains_point(V2::new(1.5, 0.5)));
        assert!(!p.contains_point(V2::new(-0.5, 0.5)));
        assert!(!p.contains_point(V2::new(0.5, -0.5)));
    }

    #[test]
    fn contains_point_open_path_uses_closing_edge() {
        let open = Path::new_from(vec![
            V2::new(0.0, 0.0),
            V2::new(10.0, 0.0),
            V2::new(10.0, 10.0),
        ]);
        let closed = Path::new_from(vec![
            V2::new(0.0, 0.0),
            V2::new(10.0, 0.0),
            V2::new(10.0, 10.0),
            V2::new(0.0, 0.0),
        ]);

        for p in [&open, &closed] {
            assert!(!p.contains_point(V2::new(1.0, 2.0)));
            assert!(p.contains_point(V2::new(5.0, 2.0)));
        }
    }

    #[test]
    fn contains_point_2() {
        let p = Path::new_from(vec![
            V2::new(0.0, 0.0),
            V2::new(1.0, 0.0),
            V2::new(0.0, 1.0),
            V2::new(0.0, 0.0),
        ]);

        assert!(p.contains_point(V2::new(0.25, 0.25)));
        assert!(!p.contains_point(V2::new(1.0, 1.0)));

        assert!(!p.contains_point(V2::new(0.5, 1.5)));
        assert!(!p.contains_point(V2::new(1.5, 0.5)));
        assert!(!p.contains_point(V2::new(-0.5, 0.5)));
        assert!(!p.contains_point(V2::new(0.5, -0.5)));
    }
}
