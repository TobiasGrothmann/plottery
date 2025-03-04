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
}
