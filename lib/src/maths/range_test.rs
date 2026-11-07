#[cfg(test)]
mod test_range {
    use crate::Range;

    #[test]
    fn new_orders_bounds() {
        assert_eq!(Range::new(5.0, 1.0), Range { min: 1.0, max: 5.0 });
    }

    #[test]
    fn lerp_clamp_contains() {
        let r = Range::new(2.0, 4.0);
        assert_eq!(r.lerp(0.25), 2.5);
        assert_eq!(r.clamp(9.0), 4.0);
        assert!(r.contains(2.0) && r.contains(4.0) && !r.contains(4.1));

        let i = Range::new(0, 10);
        assert_eq!(i.lerp(0.5), 5);
    }

    #[test]
    fn sample_stays_inside() {
        let r = Range::new(-1.0, 1.0);
        let i = Range::new(3, 5);
        for _ in 0..200 {
            assert!(r.contains(r.sample()));
            assert!(i.contains(i.sample()));
        }
        assert_eq!(Range::new(2.0, 2.0).sample(), 2.0);
        assert_eq!(Range::new(7, 7).sample(), 7);
    }
}
