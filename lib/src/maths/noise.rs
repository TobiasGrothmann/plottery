use std::cell::RefCell;

use fastnoise_lite::{FastNoiseLite, NoiseType};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::V2;

thread_local! {
    static PERLIN: RefCell<FastNoiseLite> = {
        let mut noise = FastNoiseLite::new();
        noise.set_noise_type(Some(NoiseType::Perlin));
        RefCell::new(noise)
    };
    static SIMPLEX: RefCell<FastNoiseLite> = {
        let mut noise = FastNoiseLite::new();
        noise.set_noise_type(Some(NoiseType::OpenSimplex2S));
        RefCell::new(noise)
    };
    static WORLEY: RefCell<FastNoiseLite> = {
        let mut noise = FastNoiseLite::new();
        noise.set_noise_type(Some(NoiseType::Cellular));
        RefCell::new(noise)
    };
}

pub fn seed(seed: i32) {
    PERLIN.with_borrow_mut(|noise| noise.set_seed(Some(seed)));
    SIMPLEX.with_borrow_mut(|noise| noise.set_seed(Some(seed)));
    WORLEY.with_borrow_mut(|noise| noise.set_seed(Some(seed)));
}

pub fn seed_random() {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as i32;
    PERLIN.with_borrow_mut(|noise| noise.set_seed(Some(seed)));
    SIMPLEX.with_borrow_mut(|noise| noise.set_seed(Some(seed)));
    WORLEY.with_borrow_mut(|noise| noise.set_seed(Some(seed)));
}

pub fn perlin_2d(location: &V2) -> f32 {
    let val = PERLIN.with_borrow(|noise| noise.get_noise_2d(location.x, location.y));
    (val + 1.0) / 2.0
}

pub fn perlin_3d(x: f32, y: f32, z: f32) -> f32 {
    let val = PERLIN.with_borrow(|noise| noise.get_noise_3d(x, y, z));
    (val + 1.0) / 2.0
}

pub fn simplex_2d(location: &V2) -> f32 {
    let val = SIMPLEX.with_borrow(|noise| noise.get_noise_2d(location.x, location.y));
    (val + 1.0) / 2.0
}

pub fn simplex_3d(x: f32, y: f32, z: f32) -> f32 {
    let val = SIMPLEX.with_borrow(|noise| noise.get_noise_3d(x, y, z));
    (val + 1.0) / 2.0
}

pub fn worley_2d(location: &V2) -> f32 {
    let val = WORLEY.with_borrow(|noise| noise.get_noise_2d(location.x, location.y));
    (val + 1.0) / 2.0
}

pub fn worley_3d(x: f32, y: f32, z: f32) -> f32 {
    let val = WORLEY.with_borrow(|noise| noise.get_noise_3d(x, y, z));
    (val + 1.0) / 2.0
}
