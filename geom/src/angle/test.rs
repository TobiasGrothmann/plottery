#[cfg(test)]
mod test_angle {
    use std::f32::consts::PI;

    use crate::angle::Angle;

    #[test]
    fn angle_creation() {
        let a = Angle::from_degree(180.0);
        let b = Angle::from_rad(PI);
        let c = Angle::from_rotations(0.5);

        assert_eq!(Into::<f32>::into(a.clone()), Into::<f32>::into(b));
        assert_eq!(Into::<f32>::into(a), Into::<f32>::into(c));

        let a = Angle::from_degree(-90.0);
        let b = Angle::from_rad(-PI * 0.5);
        let c = Angle::from_rotations(-0.25);

        assert_eq!(Into::<f32>::into(a.clone()), Into::<f32>::into(b));
        assert_eq!(Into::<f32>::into(a), Into::<f32>::into(c));
    }

    #[test]
    fn angle_conversion() {
        let a = Angle::from_degree(-90.0);
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
        let a = Angle::from_degree(360.0 + 90.0);
        let b = Angle::from_rad(2.0 * PI + PI * 0.5);
        let c = Angle::from_rotations(1.25);

        assert_eq!(a.to_rad(), b.to_rad());
        assert_eq!(a.to_rad(), c.to_rad());

        assert!((a.wrap().to_rad() - Angle::from_degree(90.0).to_rad()).abs() < 0.000001);
    }

    #[test]
    fn operators() {
        let a = Angle::from_degree(180.0) + Angle::from_degree(180.0);
        assert_eq!(a.to_degree(), 360.0);

        let a = Angle::from_degree(180.0) - Angle::from_degree(180.0);
        assert_eq!(a.to_degree(), 0.0);

        let a = Angle::from_degree(180.0) * 2.0;
        assert_eq!(a.to_degree(), 360.0);

        let a = Angle::from_degree(360.0) / 2.0;
        assert_eq!(a.to_degree(), 180.0);
    }
}
