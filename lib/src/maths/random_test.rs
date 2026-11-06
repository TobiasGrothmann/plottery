#[cfg(test)]
mod test_random {
    use crate::{rand_index, rand_sign};

    #[test]
    fn sign_and_index() {
        for _ in 0..100 {
            assert!(rand_sign().abs() == 1.0);
            assert!(rand_index(3) < 3);
        }
        assert_eq!(rand_index(1), 0);
    }
}
