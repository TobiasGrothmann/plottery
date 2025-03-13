#[cfg(test)]
mod test_angle {
    use std::f32::consts::PI;

    use crate::{Angle, LARGE_EPSILON};

    #[test]
    fn angle_creation() {
        let a = Angle::from_degrees(180.0);
        let b = Angle::from_rad(PI);
        let c = Angle::from_rotations(0.5);

        assert_eq!(Into::<f32>::into(a), Into::<f32>::into(b));
        assert_eq!(Into::<f32>::into(a), Into::<f32>::into(c));

        let a = Angle::from_degrees(-90.0);
        let b = Angle::from_rad(-PI * 0.5);
        let c = Angle::from_rotations(-0.25);

        assert_eq!(Into::<f32>::into(a), Into::<f32>::into(b));
        assert_eq!(Into::<f32>::into(a), Into::<f32>::into(c));
    }

    #[test]
    fn angle_conversion() {
        let a = Angle::from_degrees(-90.0);
        let b = Angle::from_rad(-PI * 0.5);
        let c = Angle::from_rotations(-0.25);

        assert_eq!(a.to_rad(), -PI * 0.5);
        assert_eq!(a.to_rad(), b.to_rad());
        assert_eq!(a.to_rad(), c.to_rad());

        assert_eq!(a.to_degree(), -90.0);
        assert_eq!(a.to_degree(), b.to_degree());
        assert_eq!(a.to_degree(), c.to_degree());

        assert_eq!(a.to_rotations(), -0.25);
        assert_eq!(a.to_rotations(), b.to_rotations());
        assert_eq!(a.to_rotations(), c.to_rotations());
    }

    #[test]
    fn wrap_around() {
        let a = Angle::from_degrees(360.0 + 90.0);
        let b = Angle::from_rad(2.0 * PI + PI * 0.5);
        let c = Angle::from_rotations(1.25);

        assert_eq!(a.to_rad(), b.to_rad());
        assert_eq!(a.to_rad(), c.to_rad());

        assert!(
            (a.mod_one_rotation().to_rad() - Angle::from_degrees(90.0).to_rad()).abs()
                < LARGE_EPSILON
        );
    }

    #[test]
    fn operators() {
        let a = Angle::from_degrees(180.0) + Angle::from_degrees(180.0);
        assert_eq!(a.to_degree(), 360.0);

        let a = Angle::from_degrees(180.0) - Angle::from_degrees(180.0);
        assert_eq!(a.to_degree(), 0.0);

        let a = Angle::from_degrees(180.0) * 2.0;
        assert_eq!(a.to_degree(), 360.0);

        let a = Angle::from_degrees(360.0) / 2.0;
        assert_eq!(a.to_degree(), 180.0);
    }

    #[test]
    fn equality() {
        let a = Angle::from_degrees(180.0);

        let a1 = Angle::from_degrees(180.0);
        let a2 = Angle::from_rad(PI);
        let a3 = Angle::from_rotations(0.5);
        let a4 = Angle::from_degrees(180.0 + 360.0).mod_one_rotation();
        assert_eq!(a, a1);
        assert_eq!(a, a2);
        assert_eq!(a, a3);
        assert_eq!(a, a4);

        let b1 = Angle::from_degrees(180.0 + 360.0);
        let b2 = Angle::from_degrees(180.0 - 360.0);
        assert_ne!(a, b1);
        assert_ne!(a, b2);
    }

    #[test]
    fn comparison() {
        let a = Angle::from_degrees(180.0);

        let b = Angle::from_degrees(181.0);
        assert!(a < b);

        let c = Angle::from_degrees(180.0 + 360.0);
        assert!(a < c);

        let d = Angle::from_degrees(179.0);
        assert!(a > d);

        let e = Angle::from_degrees(-180.0);
        assert!(a > e);
    }

    #[test]
    fn lerp_to_fixed() {
        let a = Angle::from_degrees(0.0);
        let b = Angle::from_degrees(180.0);

        let mut i = 0;
        let mut last_angle = a;
        for interpolated_angle in a.lerp_iter_fixed(b, 100) {
            assert!(interpolated_angle >= last_angle);
            last_angle = interpolated_angle;
            i += 1;
        }

        assert_eq!(i, 101);
    }

    #[test]
    fn with_smallest_rotation() {
        let a = Angle::from_degrees(10.0);

        let b = Angle::from_degrees(90.0).with_smallest_rotation_to(a);
        assert_eq!(b, Angle::from_degrees(90.0));

        let c = Angle::from_degrees(270.0).with_smallest_rotation_to(a);
        assert_eq!(c, Angle::from_degrees(-90.0));

        let c = Angle::from_degrees(-180.0).with_smallest_rotation_to(a);
        assert_eq!(c, Angle::from_degrees(180.0));
    }

    #[test]
    fn distance() {
        let a = Angle::from_degrees(10.0);

        // simple
        assert_eq!(
            a.dist_mod_one_rotation(Angle::from_degrees(20.0)),
            Angle::from_degrees(10.0)
        );
        assert_eq!(
            Angle::from_degrees(20.0).dist_mod_one_rotation(a),
            Angle::from_degrees(10.0)
        );
        assert_eq!(
            a.dist_mod_one_rotation(Angle::from_degrees(0.0)),
            Angle::from_degrees(10.0)
        );

        // wrap around
        assert_eq!(
            a.dist_mod_one_rotation(Angle::from_degrees(350.0)),
            Angle::from_degrees(20.0)
        );
        assert_eq!(
            Angle::from_degrees(350.0).dist_mod_one_rotation(a),
            Angle::from_degrees(20.0)
        );

        for deg_a in -360..720 {
            let a = Angle::from_degrees(deg_a as f32);
            for deg_b in -360..720 {
                let b = Angle::from_degrees(deg_b as f32);
                assert_eq!(a.dist_mod_one_rotation(b), b.dist_mod_one_rotation(a));
                println!(
                    "from {} to {} -> distance: {}",
                    deg_a,
                    deg_b,
                    a.dist_mod_one_rotation(b).to_degree()
                );
                assert!(a.dist_mod_one_rotation(b) >= Angle::zero());
                assert!(a.dist_mod_one_rotation(b) <= Angle::half_rotation());
            }
        }

        // negative
        assert_eq!(
            Angle::from_degrees(-5.0).dist_mod_one_rotation(Angle::from_degrees(5.0)),
            Angle::from_degrees(10.0)
        );
        assert_eq!(
            Angle::from_degrees(360.0).dist_mod_one_rotation(Angle::from_degrees(360.0)),
            Angle::from_degrees(0.0)
        );
        assert_eq!(
            Angle::from_degrees(-90.0).dist_mod_one_rotation(Angle::from_degrees(270.0)),
            Angle::from_degrees(0.0)
        );
        assert_eq!(
            Angle::from_degrees(-90.0)
                .dist_mod_one_rotation(Angle::from_degrees(270.0) + Angle::full_rotation()),
            Angle::from_degrees(0.0)
        );
        assert_eq!(
            Angle::from_degrees(-90.0).dist_mod_one_rotation(
                Angle::from_degrees(270.0)
                    + Angle::full_rotation()
                    + Angle::full_rotation()
                    + Angle::full_rotation()
            ),
            Angle::from_degrees(0.0)
        );
    }

    #[test]
    fn angle_direction_cw_cc() {
        assert_eq!(Angle::right_cw().positive(), Angle::right_cc());
        assert_eq!(Angle::up_cw().positive(), Angle::up_cc());
        assert_eq!(Angle::left_cw().positive(), Angle::left_cc());
        assert_eq!(Angle::down_cw().positive(), Angle::down_cc());
    }
}
