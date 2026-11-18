#[cfg(test)]
mod test_noise {
    use rand::{rngs::StdRng, Rng, SeedableRng};

    use crate::{
        maths::global_rng::{seed, seed_for_test},
        MarchingSquares, Noise, NoiseScale, NoiseSettings, NoiseType, Rect, SampleSettings, V2,
    };

    #[test]
    fn test_noise_value_range() {
        let _rng = seed_for_test(1);
        let mut rng = StdRng::seed_from_u64(1);
        for noise_type in [NoiseType::Perlin, NoiseType::Simplex, NoiseType::Worley] {
            let noise = Noise::new(noise_type).frequency(0.01);
            for _ in 0..10000 {
                let location = V2::new(rng.gen_range(-500.0..500.0), rng.gen_range(-500.0..500.0));
                assert!((0.0..=1.0).contains(&noise.get(location)));
            }
        }
    }

    #[test]
    fn as_fn_matches_get() {
        let _rng = seed_for_test(1);
        let noise = Noise::perlin().seed(3);
        let field = noise.as_fn();
        let p = V2::new(0.3, 0.7);
        assert_eq!(field(p), noise.get(p));
        let squares = MarchingSquares::new(
            Rect::new(V2::zero(), V2::xy(1.0)),
            SampleSettings::new(4.0),
            noise.as_fn(),
        );
        let _ = squares.lines(0.5);
    }

    #[test]
    fn global_seed_decides_the_seed_of_new_noise() {
        let _rng = seed_for_test(1);
        let p = V2::new(0.5, 0.5);
        for noise_type in [NoiseType::Perlin, NoiseType::Simplex, NoiseType::Worley] {
            seed(1);
            let first = Noise::new(noise_type).get(p);
            seed(1);
            assert_eq!(first, Noise::new(noise_type).get(p));
            seed(2);
            assert_ne!(first, Noise::new(noise_type).get(p));
        }
    }

    #[test]
    fn test_noise_settings_per_paper_divides_frequency() {
        let size = V2::new(20.0, 40.0);
        let _rng = seed_for_test(1);
        let settings = NoiseSettings::default()
            .with_frequency(4.0)
            .with_scale(NoiseScale::PerPaper);

        // the same explicit seed, so the two noises can match
        let per_paper = settings.noise_for(size).seed(7);
        let absolute = settings.with_frequency(4.0 / 20.0).noise().seed(7);
        for i in 0..20 {
            let p = V2::new(i as f32 * 1.3, i as f32 * 0.7);
            assert_eq!(per_paper.get(p), absolute.get(p));
        }

        let per_cm = settings
            .with_scale(NoiseScale::PerCm)
            .noise_for(size)
            .seed(7);
        assert_eq!(
            per_cm.get(V2::xy(2.0)),
            settings.noise().seed(7).get(V2::xy(2.0))
        );
    }
}
