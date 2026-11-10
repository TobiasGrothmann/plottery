#[cfg(test)]
mod test_random {
    use crate::{maths::thread_local::seed_for_test, rand_index, rand_sign};

    #[test]
    fn sign_and_index() {
        let _rng = seed_for_test(1);
        for _ in 0..100 {
            assert!(rand_sign().abs() == 1.0);
            assert!(rand_index(3) < 3);
        }
        assert_eq!(rand_index(1), 0);
    }
}
