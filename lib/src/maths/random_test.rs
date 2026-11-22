#[cfg(test)]
mod test_random {
    use crate::{
        maths::global_rng::seed_for_test, rand_index, rand_range, rand_range_i,
        rand_range_i_inclusive, rand_sign, Range,
    };

    #[test]
    fn sign_and_index() {
        let _rng = seed_for_test(1);
        for _ in 0..100 {
            assert!(rand_sign().abs() == 1.0);
            assert!(rand_index(3) < 3);
        }
        assert_eq!(rand_index(1), 0);
    }

    #[test]
    fn empty_ranges_return_from() {
        let _rng = seed_for_test(2);
        assert_eq!(rand_range(2.0, 2.0), 2.0);
        assert_eq!(rand_range(3.0, 1.0), 3.0);
        assert_eq!(rand_range_i(4, 4), 4);
        assert_eq!(rand_range_i_inclusive(5, 1), 5);
        assert!(Range::new(i32::MAX - 1, i32::MAX).sample() >= i32::MAX - 1);
    }
}
