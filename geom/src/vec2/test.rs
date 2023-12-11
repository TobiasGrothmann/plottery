#[cfg(test)]
mod test_vec2 {
    use crate::{
        angle::angle::Angle,
        traits::{rotate::Rotate, rotate90::Rotate90},
        vec2::V2,
    };

    #[test]
    fn add() {
        let v = V2::new(1.0, 2.0) + V2::new(1.0, 1.0);
        assert_eq!(v.x, 2.0);
        assert_eq!(v.y, 3.0);

        let v = V2::new(5.0, 5.0) + V2::xy(-5.0);
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 0.0);
    }

    #[test]
    fn sub() {
        let v = V2::new(1.0, 2.0) - V2::new(1.0, 1.0);
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 1.0);

        let v = V2::new(5.0, 5.0) - V2::xy(-5.0);
        assert_eq!(v.x, 10.0);
        assert_eq!(v.y, 10.0);
    }

    #[test]
    fn mult() {
        let v = V2::new(1.0, 2.0) * V2::new(1.0, 1.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);

        let v = V2::new(5.0, 5.0) * V2::xy(-5.0);
        assert_eq!(v.x, -25.0);
        assert_eq!(v.y, -25.0);
    }

    #[test]
    fn div() {
        let v = V2::new(1.0, 2.0) / V2::new(1.0, 1.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);

        let v = V2::new(5.0, 5.0) / V2::xy(-5.0);
        assert_eq!(v.x, -1.0);
        assert_eq!(v.y, -1.0);
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
        let v_new = v.rotate(&Angle::from_degree(90.0));
        assert_eq!(v_new, V2::new(0.0, 1.0));

        let v = V2::new(1.0, 0.0);
        let v_new = v.rotate(&Angle::from_degree(-90.0));
        assert_eq!(v_new, V2::new(0.0, -1.0));

        let v = V2::new(1.0, 0.0);
        let v_new = v.rotate(&&Angle::from_rotations(1.0));
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
        let v2 = v.rotate_around(&pivot, &Angle::from_degree(90.0));
        assert_eq!(v1, v2);

        let v1 = v.rotate_180_around(&pivot);
        let v2 = v.rotate_around(&pivot, &Angle::from_degree(180.0));
        assert_eq!(v1, v2);

        let v1 = v.rotate_270_around(&pivot);
        let v2 = v.rotate_around(&pivot, &Angle::from_degree(270.0));
        assert_eq!(v1, v2);
    }

    #[test]
    fn rotate_around() {
        let v = V2::new(1.0, 0.0);
        let v_new = v.rotate_around(&V2::new(1.0, 1.0), &Angle::from_degree(90.0));
        assert_eq!(v_new, V2::new(2.0, 1.0));
    }

    #[test]
    fn min_max() {
        let v = V2::new(1.0, 0.0).min(&V2::new(1.0, 1.0));
        assert_eq!(v, V2::new(1.0, 0.0));

        let v = V2::new(1.0, 0.0).max(&V2::new(1.0, 1.0));
        assert_eq!(v, V2::new(1.0, 1.0));
    }
}
