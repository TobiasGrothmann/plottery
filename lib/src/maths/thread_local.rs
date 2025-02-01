use std::cell::RefCell;

use fastnoise_lite::{FastNoiseLite, NoiseType};
use rand::{rngs::StdRng, Rng, SeedableRng};

thread_local! {
    pub static PERLIN: RefCell<FastNoiseLite> = {
        let mut noise = FastNoiseLite::new();
        noise.set_noise_type(Some(NoiseType::Perlin));
        RefCell::new(noise)
    };
    pub static SIMPLEX: RefCell<FastNoiseLite> = {
        let mut noise = FastNoiseLite::new();
        noise.set_noise_type(Some(NoiseType::OpenSimplex2S));
        RefCell::new(noise)
    };
    pub static WORLEY: RefCell<FastNoiseLite> = {
        let mut noise = FastNoiseLite::new();
        noise.set_noise_type(Some(NoiseType::Cellular));
        RefCell::new(noise)
    };
    pub static RNG: RefCell<StdRng> = RefCell::new(StdRng::from_entropy());
}

pub fn seed(seed: i32) {
    PERLIN.with_borrow_mut(|noise| noise.set_seed(Some(seed)));
    SIMPLEX.with_borrow_mut(|noise| noise.set_seed(Some(seed)));
    WORLEY.with_borrow_mut(|noise| noise.set_seed(Some(seed)));
    RNG.with_borrow_mut(|rng| *rng = StdRng::seed_from_u64(seed as u64));
}

pub fn seed_random() {
    seed(rand::thread_rng().gen::<i32>());
}
