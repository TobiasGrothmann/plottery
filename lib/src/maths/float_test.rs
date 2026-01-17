#[cfg(test)]
mod test_float {
    use crate::FloatInterpolation;

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
