#[cfg(test)]
mod test_float {
    use crate::{fractions, fractions_exclusive, linspace, FloatInterpolation};
    use std::f32::consts::PI;

    #[test]
    fn helpers() {
        assert!((0.0_f32.sin01() - 0.5).abs() < 1e-6);
        assert!(((PI / 2.0).sin01() - 1.0).abs() < 1e-6);
        assert!((0.0_f32.cos01() - 1.0).abs() < 1e-6);
        assert_eq!(1.5_f32.clamp01(), 1.0);
        assert_eq!((-0.5_f32).clamp01(), 0.0);

        assert_eq!(
            fractions(4).collect::<Vec<_>>(),
            [0.0, 0.25, 0.5, 0.75, 1.0]
        );
        assert_eq!(
            fractions_exclusive(4).collect::<Vec<_>>(),
            [0.0, 0.25, 0.5, 0.75]
        );
        assert_eq!(linspace(1.0, 3.0, 3).collect::<Vec<_>>(), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn lerp() {
        assert_eq!(0.0.lerp(1.0, 0.0), 0.0);
        assert_eq!(0.0.lerp(1.0, 0.5), 0.5);
        assert_eq!(0.0.lerp(1.0, 1.0), 1.0);

        assert_eq!(0.0.lerp(10.0, 0.5), 5.0);
    }

    #[test]
    fn lerp_iter() {
        let mut iter = 0.0.lerp_iter(1.0, 0.5);
        assert_eq!(iter.next(), Some(0.0));
        assert_eq!(iter.next(), Some(0.5));
        assert_eq!(iter.next(), Some(1.0));
        assert_eq!(iter.next(), None);

        let mut iter = 0.0.lerp_iter(10.0, 2.1); // will be rounded to fit
        assert_eq!(iter.next(), Some(0.0));
        assert_eq!(iter.next(), Some(2.0));
        assert_eq!(iter.next(), Some(4.0));
        assert_eq!(iter.next(), Some(6.0));
        assert_eq!(iter.next(), Some(8.0));
        assert_eq!(iter.next(), Some(10.0));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn lerp_iter_fixed() {
        let mut iter = 0.0.lerp_iter_fixed(1.0, 4);
        assert_eq!(iter.next(), Some(0.0));
        assert_eq!(iter.next(), Some(0.25));
        assert_eq!(iter.next(), Some(0.5));
        assert_eq!(iter.next(), Some(0.75));
        assert_eq!(iter.next(), Some(1.0));
        assert_eq!(iter.next(), None);

        let mut iter = 0.0.lerp_iter_fixed(10.0, 2);
        assert_eq!(iter.next(), Some(0.0));
        assert_eq!(iter.next(), Some(5.0));
        assert_eq!(iter.next(), Some(10.0));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn linlin() {
        // Map 5.0 from [0,10] to [0,100]
        assert_eq!(5.0.linlin(0.0, 10.0, 0.0, 100.0), 50.0);

        // Map 0.5 from [0,1] to [0,10]
        assert_eq!(0.5.linlin(0.0, 1.0, 0.0, 10.0), 5.0);

        // Map 75.0 from [0,100] to [0,1]
        assert_eq!(75.0.linlin(0.0, 100.0, 0.0, 1.0), 0.75);

        // Map with negative ranges
        assert_eq!(0.0.linlin(-10.0, 10.0, 0.0, 100.0), 50.0);
        assert_eq!(5.0.linlin(0.0, 10.0, -100.0, 100.0), 0.0);

        // Map with reversed ranges
        assert_eq!(5.0.linlin(0.0, 10.0, 10.0, 0.0), 5.0);
    }

    #[test]
    fn linlin_f64() {
        // Map 5.0 from [0,10] to [0,100]
        assert_eq!(5.0_f64.linlin(0.0, 10.0, 0.0, 100.0), 50.0);

        // Map 0.5 from [0,1] to [0,10]
        assert_eq!(0.5_f64.linlin(0.0, 1.0, 0.0, 10.0), 5.0);

        // Map 75.0 from [0,100] to [0,1]
        assert_eq!(75.0_f64.linlin(0.0, 100.0, 0.0, 1.0), 0.75);

        // Map with negative ranges
        assert_eq!(0.0_f64.linlin(-10.0, 10.0, 0.0, 100.0), 50.0);
        assert_eq!(5.0_f64.linlin(0.0, 10.0, -100.0, 100.0), 0.0);

        // Map with reversed ranges
        assert_eq!(5.0_f64.linlin(0.0, 10.0, 10.0, 0.0), 5.0);
    }

    #[test]
    fn lerp_iter_f64() {
        let mut iter = 0.0_f64.lerp_iter(10.0, 2.1);
        assert_eq!(iter.next(), Some(0.0_f64));
        assert_eq!(iter.next(), Some(2.0_f64));
        assert_eq!(iter.next(), Some(4.0_f64));
        assert_eq!(iter.next(), Some(6.0_f64));
        assert_eq!(iter.next(), Some(8.0_f64));
        assert_eq!(iter.next(), Some(10.0_f64));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn lerp_iter_fixed_f64() {
        let mut iter = 0.0_f64.lerp_iter_fixed(1.0, 4);
        assert_eq!(iter.next(), Some(0.0_f64));
        assert_eq!(iter.next(), Some(0.25_f64));
        assert_eq!(iter.next(), Some(0.5_f64));
        assert_eq!(iter.next(), Some(0.75_f64));
        assert_eq!(iter.next(), Some(1.0_f64));
        assert_eq!(iter.next(), None);
    }
}
