#[cfg(test)]
mod test_noise {
    use rand::random;

    use crate::{
        perlin_2d, perlin_3d, seed, seed_random, simplex_2d, simplex_3d, worley_2d, worley_3d, V2,
    };

    #[test]
    fn test_noise_value_range() {
        println!("perlin_2d");
        test_noise_range_helper(perlin_2d);
        println!("simplex_2d");
        test_noise_range_helper(simplex_2d);
        println!("worley_2d");
        test_noise_range_helper(worley_2d);
    }
    fn test_noise_range_helper(noise: fn(&V2) -> f32) {
        for _ in 0..10000 {
            let location = V2::new(
                random::<f32>() * 1000.0 - 500.0,
                random::<f32>() * 1000.0 - 500.0,
            );
            let noise = noise(&location);
            assert!(noise >= 0.0 && noise <= 1.0);
        }
    }

    #[test]
    fn test_noise_seed() {
        println!("perlin_2d");
        test_noise_seed_helper(perlin_2d);
        println!("simplex_2d");
        test_noise_seed_helper(simplex_2d);
        println!("worley_2d");
        test_noise_seed_helper(worley_2d);
    }
    fn test_noise_seed_helper(noise: fn(&V2) -> f32) {
        seed(1);
        let noise1 = noise(&V2::new(0.5, 0.5));
        let noise2 = noise(&V2::new(0.5, 0.5));
        assert_eq!(noise1, noise2); // same location should result in same value

        seed(2);
        let noise3 = noise(&V2::new(0.5, 0.5));
        let noise4 = noise(&V2::new(0.5, 0.5));
        assert_eq!(noise3, noise4); // same location should result in same value
        assert_ne!(noise1, noise3); // different seed should result in different value
        assert_ne!(noise1, noise4);

        // using seed_random should result in different values each time
        seed_random();
        let noise5 = noise(&V2::new(0.5, 0.5));
        std::thread::sleep(std::time::Duration::from_millis(1)); // necessary because SystemTime::now is used as seed
        seed_random();
        let noise6 = noise(&V2::new(0.5, 0.5));
        assert_ne!(noise5, noise6)
    }

    #[test]
    fn test_noise_location() {
        println!("perlin");
        test_noise_location_helper(perlin_2d, perlin_3d);
        println!("simplex");
        test_noise_location_helper(simplex_2d, simplex_3d);
        println!("worley");
        test_noise_location_helper(worley_2d, worley_3d);
    }
    fn test_noise_location_helper(noise_2d: fn(&V2) -> f32, noise_3d: fn(f32, f32, f32) -> f32) {
        seed_random();
        let noise1 = noise_2d(&V2::new(0.5, 0.5));
        let noise2 = noise_2d(&V2::new(0.47, 0.53));
        assert_ne!(noise1, noise2);

        seed_random();
        let noise1 = noise_3d(0.5, 0.5, 0.5);
        let noise2 = noise_3d(0.47, 0.53, 0.5);
        assert_ne!(noise1, noise2);
    }
}
