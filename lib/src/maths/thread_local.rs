use std::cell::RefCell;
use std::sync::Mutex;

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
}

lazy_static::lazy_static! {
    pub static ref RNG: Mutex<StdRng> = Mutex::new(StdRng::from_entropy());
}

/// Seeds all thread-local random number generators with the given seed. This affects all functions in [`crate::random`]
pub fn seed(seed: i32) {
    PERLIN.with_borrow_mut(|noise| noise.set_seed(Some(seed)));
    SIMPLEX.with_borrow_mut(|noise| noise.set_seed(Some(seed)));
    WORLEY.with_borrow_mut(|noise| noise.set_seed(Some(seed)));
    if let Ok(mut rng) = RNG.lock() {
        *rng = StdRng::seed_from_u64(seed as u64);
    }
}

/// Seeds all thread-local random number generators with a random seed. This affects all functions in [`crate::random`]
pub fn seed_random() {
    seed(rand::thread_rng().gen::<i32>());
}
