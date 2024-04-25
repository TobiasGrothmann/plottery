use crate::V2;

use super::thread_local::{PERLIN, SIMPLEX, WORLEY};

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
