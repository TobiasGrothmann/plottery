#[cfg(test)]
mod test_v2 {
    use itertools::Itertools;

    use crate::{geometry::Angle, Rotate, Rotate90, LARGE_EPSILON, V2};

    #[test]
    fn polar() {
        assert_eq!(V2::polar(Angle::from_rad(0.0), 1.0), V2::new(1.0, 0.0));
        assert_eq!(
            V2::polar(Angle::from_degrees(180.0), 1.0),
            V2::new(-1.0, 0.0)
        );
        assert_eq!(V2::polar(Angle::from_degrees(90.0), 5.0), V2::new(0.0, 5.0));
        assert_eq!(
            V2::polar(Angle::from_degrees(45.0), 1.0),
            V2::xy(1.0 / 2.0f32.sqrt())
        );
    }

    #[test]
    fn add() {
        let v = V2::new(1.0, 2.0) + V2::new(1.0, 1.0);
        assert_eq!(v.x, 2.0);
        assert_eq!(v.y, 3.0);

        let v = V2::new(5.0, 5.0) + V2::xy(-5.0);
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 0.0);

        let mut v = V2::new(5.0, 5.0);
        v += V2::new(-5.0, 5.0);
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 10.0);
    }

    #[test]
    fn sub() {
        let v = V2::new(1.0, 2.0) - V2::new(1.0, 1.0);
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 1.0);

        let v = V2::new(5.0, 5.0) - V2::xy(-5.0);
        assert_eq!(v.x, 10.0);
        assert_eq!(v.y, 10.0);

        let mut v = V2::new(5.0, 5.0);
        v -= V2::new(5.0, 0.0);
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 5.0);
    }

    #[test]
    fn mult() {
        let v = V2::new(1.0, 2.0) * V2::new(1.0, 1.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);

        let v = V2::new(5.0, 5.0) * V2::xy(-5.0);
        assert_eq!(v.x, -25.0);
        assert_eq!(v.y, -25.0);

        let mut v = V2::new(5.0, 5.0);
        v *= V2::xy(2.0);
        assert_eq!(v.x, 10.0);
        assert_eq!(v.y, 10.0);
    }

    #[test]
    fn div() {
        let v = V2::new(1.0, 2.0) / V2::new(1.0, 1.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);

        let v = V2::new(5.0, 5.0) / V2::xy(-5.0);
        assert_eq!(v.x, -1.0);
        assert_eq!(v.y, -1.0);

        let mut v = V2::new(5.0, 5.0);
        v /= 5.0;
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 1.0);
    }

    #[test]
    fn dist() {
        let dist = V2::new(1.0, 2.0).dist(&V2::new(1.0, 1.0));
        assert_eq!(dist, 1.0);

        let v1 = V2::new(5.0, 2.0);
        let v2 = V2::new(6.0, 3.0);
        assert_eq!(v1.dist(&v2), 2.0_f32.sqrt());
    }

    #[test]
    fn dist_manhattan() {
        let dist = V2::new(1.0, 2.0).dist_manhattan(&V2::new(1.0, 1.0));
        assert_eq!(dist, 1.0);

        let v1 = V2::new(5.0, 2.0);
        let v2 = V2::new(6.0, 3.0);
        assert_eq!(v1.dist_manhattan(&v2), 2.0);
    }

    #[test]
    fn rotate() {
        let v = V2::new(1.0, 0.0);
        let v_new = v.rotate(&Angle::from_degrees(90.0));
        assert_eq!(v_new, V2::new(0.0, 1.0));

        let v = V2::new(1.0, 0.0);
        let v_new = v.rotate(&Angle::from_degrees(-90.0));
        assert_eq!(v_new, V2::new(0.0, -1.0));

        let v = V2::new(1.0, 0.0);
        let v_new = v.rotate(&Angle::from_rotations(1.0));
        assert_eq!(v, v_new);
    }

    #[test]
    fn rotate_90_180_270() {
        let v = V2::new(1.0, 0.0);
        let v_new = v.rotate_90();
        assert_eq!(v_new, V2::new(0.0, 1.0));

        let v = V2::new(1.0, 0.0);
        let v_new = v.rotate_180();
        assert_eq!(v_new, V2::new(-1.0, 0.0));

        let v = V2::new(1.0, 0.0);
        let v_new = v.rotate_270();
        assert_eq!(v_new, V2::new(0.0, -1.0));
    }

    #[test]
    fn rotate_around_90_180_270() {
        let v = V2::new(1.0, 0.0);
        let v_new = v.rotate_90_around(&V2::new(1.0, 1.0));
        assert_eq!(v_new, V2::new(2.0, 1.0));

        let v = V2::new(1.0, 0.0);
        let v_new = v.rotate_180_around(&V2::new(1.0, 1.0));
        assert_eq!(v_new, V2::new(1.0, 2.0));

        let v = V2::new(1.0, 0.0);
        let v_new = v.rotate_270_around(&V2::new(1.0, 1.0));
        assert_eq!(v_new, V2::new(0.0, 1.0));
    }

    #[test]
    fn rotate_around_90_180_270_compare() {
        let v = V2::new(1.61, -9.2);
        let pivot = V2::new(-5.0, 4.221);

        let v1 = v.rotate_90_around(&pivot);
        let v2 = v.rotate_around(&pivot, &Angle::from_degrees(90.0));
        assert_eq!(v1, v2);

        let v1 = v.rotate_180_around(&pivot);
        let v2 = v.rotate_around(&pivot, &Angle::from_degrees(180.0));
        assert_eq!(v1, v2);

        let v1 = v.rotate_270_around(&pivot);
        let v2 = v.rotate_around(&pivot, &Angle::from_degrees(270.0));
        assert_eq!(v1, v2);
    }

    #[test]
    fn rotate_around() {
        let v = V2::new(1.0, 0.0);
        let v_new = v.rotate_around(&V2::new(1.0, 1.0), &Angle::from_degrees(90.0));
        assert_eq!(v_new, V2::new(2.0, 1.0));
    }

    #[test]
    fn rotate_around_mut() {
        let mut v = V2::new(1.0, 0.0);
        v.rotate_around_mut(&V2::new(1.0, 1.0), &Angle::from_degrees(90.0));
        assert_eq!(v, V2::new(2.0, 1.0));
    }

    #[test]
    fn min_max() {
        let v = V2::new(1.0, 0.0).min(&V2::new(1.0, 1.0));
        assert_eq!(v, V2::new(1.0, 0.0));

        let v = V2::new(1.0, 0.0).max(&V2::new(1.0, 1.0));
        assert_eq!(v, V2::new(1.0, 1.0));
    }

    #[test]
    fn len() {
        let v = V2::new(1.0, 0.0);
        assert_eq!(v.len(), 1.0);

        let v = V2::new(1.0, 1.0);
        assert_eq!(v.len(), 2.0_f32.sqrt());
    }

    #[test]
    fn len_squared() {
        let v = V2::new(1.0, 0.0);
        assert_eq!(v.len_squared(), 1.0);

        let v = V2::new(1.0, 1.0);
        assert_eq!(v.len_squared(), 2.0_f32);
    }

    #[test]
    fn din_a_sizes() {
        let sizes = [V2::a0(),
            V2::a1(),
            V2::a2(),
            V2::a3(),
            V2::a4(),
            V2::a5(),
            V2::a6(),
            V2::a7(),
            V2::a8(),
            V2::a9(),
            V2::a10()];

        for (i, (size, size_next)) in sizes.iter().tuple_windows().enumerate() {
            assert!(size.len() > size_next.len());
            assert_eq!(size.x, size_next.y);

            let area_target = (100.0 * 100.0) / 2.0_f32.powi(i as i32); // Din a0 is 1m^2 - area gets halved every step
            let area = size.x * size.y;
            assert!((area - area_target).abs() < 6.0); // the actually used sizes have quite a big error
        }
    }

    #[test]
    fn normalize() {
        let v = V2::new(1.0, 0.0).normalize();
        assert_eq!(v, V2::new(1.0, 0.0));

        let v = V2::new(1.0, 1.0).normalize();
        assert!((v.len() - 1.0).abs() < LARGE_EPSILON);

        let v = V2::new(-1.0, 5.0).normalize_to(5.0);
        assert_eq!(v.len(), 5.0);
    }

    #[test]
    fn angle() {
        let v = V2::new(1.0, 0.0);
        assert_eq!(v.angle(), Angle::from_degrees(0.0));

        let v = V2::new(0.0, 1.0);
        assert_eq!(v.angle().to_degree(), 90.0);

        let v = V2::new(-1.0, 0.0);
        assert_eq!(v.angle(), Angle::from_degrees(180.0));

        let v = V2::new(0.0, -1.0);
        assert_eq!(v.angle(), Angle::from_degrees(270.0));

        let v = V2::new(1.0, 1.0);
        assert_eq!(v.angle(), Angle::from_degrees(45.0));
    }

    #[test]
    fn project() {
        let v = V2::new(0.0, 1.0);
        let v_proj = v.project_onto(&V2::new(1.0, 1.0));
        assert_eq!(v_proj, V2::new(0.5, 0.5));

        let v = V2::new(1.0, 1.0);
        let v_proj = v.project_onto(&V2::new(1.0, 0.0));
        assert_eq!(v_proj, V2::new(1.0, 0.0));

        let v = V2::new(-3.0, -2.0);
        let v_proj = v.project_onto(&V2::new(-1.0, 0.0));
        assert_eq!(v_proj, V2::new(-3.0, 0.0));
    }

    #[test]
    fn clamp_len() {
        let v = V2::new(1.0, 0.0).clamp_len(0.0, 1.0);
        assert_eq!(v, V2::new(1.0, 0.0));

        let v = V2::new(1.0, 0.0).clamp_len(0.0, 0.5);
        assert_eq!(v, V2::new(0.5, 0.0));

        let v = V2::new(1.0, 1.0).clamp_len(0.0, 1.0);
        assert_eq!(v, V2::xy(1.0 / 2.0_f32.sqrt()));
    }

    #[test]
    fn map() {
        let v = V2::new(1.0, 2.0).map(|val| val * 2.0);
        assert_eq!(v, V2::new(2.0, 4.0));
    }

    #[test]
    fn lerp() {
        let v1 = V2::new(1.0, 1.0);
        let v2 = V2::new(3.0, 2.0);

        let v_lerp = v1.lerp(&v2, 0.0);
        assert_eq!(v_lerp, v1);

        let v_lerp = v1.lerp(&v2, 1.0);
        assert_eq!(v_lerp, v2);

        let v_lerp = v1.lerp(&v2, 0.5);
        assert_eq!(v_lerp, V2::new(2.0, 1.5));
    }

    #[test]
    fn lerp_iter_fixed() {
        let v1 = V2::new(0.0, 0.0);
        let v2 = V2::new(10.0, 5.0);

        let mut i = 0;
        let mut last_v = v1;
        for interpolated_v in v1.lerp_iter_fixed(v2, 100) {
            assert!(interpolated_v.x >= last_v.x);
            assert!(interpolated_v.y >= last_v.y);
            assert_eq!(interpolated_v.y * 2.0, interpolated_v.x);
            last_v = interpolated_v;
            i += 1;
        }

        assert_eq!(i, 101);
    }
}
