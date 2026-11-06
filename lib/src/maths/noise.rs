use std::fmt;

use fastnoise_lite::{FastNoiseLite, FractalType};
use rand::Rng;

use crate::V2;

use super::thread_local::{PERLIN, RNG, SIMPLEX, WORLEY};

pub fn perlin_2d(location: V2) -> f32 {
    let val = PERLIN.with_borrow(|noise| noise.get_noise_2d(location.x, location.y));
    (val + 1.0) / 2.0
}

pub fn perlin_3d(x: f32, y: f32, z: f32) -> f32 {
    let val = PERLIN.with_borrow(|noise| noise.get_noise_3d(x, y, z));
    (val + 1.0) / 2.0
}

pub fn simplex_2d(location: V2) -> f32 {
    let val = SIMPLEX.with_borrow(|noise| noise.get_noise_2d(location.x, location.y));
    (val + 1.0) / 2.0
}

pub fn simplex_3d(x: f32, y: f32, z: f32) -> f32 {
    let val = SIMPLEX.with_borrow(|noise| noise.get_noise_3d(x, y, z));
    (val + 1.0) / 2.0
}

pub fn worley_2d(location: V2) -> f32 {
    let val = WORLEY.with_borrow(|noise| noise.get_noise_2d(location.x, location.y));
    (val + 1.0) / 2.0
}

pub fn worley_3d(x: f32, y: f32, z: f32) -> f32 {
    let val = WORLEY.with_borrow(|noise| noise.get_noise_3d(x, y, z));
    (val + 1.0) / 2.0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoiseType {
    Perlin,
    Simplex,
    Worley,
}

/// Noise with its own seed and frequency. The default frequency of 1.0 gives roughly one feature per unit.
pub struct Noise {
    noise_type: NoiseType,
    seed: i32,
    frequency: f32,
    octaves: u32,
    inner: FastNoiseLite,
}

impl Noise {
    /// Seeded from the global random number generator, see [`seed`](crate::seed).
    pub fn new(noise_type: NoiseType) -> Self {
        let seed = RNG.lock().expect("Failed to acquire RNG lock").gen();
        Self::build(noise_type, seed, 1.0, 1)
    }
    pub fn perlin() -> Self {
        Self::new(NoiseType::Perlin)
    }
    pub fn simplex() -> Self {
        Self::new(NoiseType::Simplex)
    }
    pub fn worley() -> Self {
        Self::new(NoiseType::Worley)
    }

    #[must_use]
    pub fn seed(self, seed: i32) -> Self {
        Self::build(self.noise_type, seed, self.frequency, self.octaves)
    }
    #[must_use]
    pub fn frequency(self, frequency: f32) -> Self {
        Self::build(self.noise_type, self.seed, frequency, self.octaves)
    }
    /// Number of fractal (FBm) octaves, 1 disables the fractal.
    #[must_use]
    pub fn octaves(self, octaves: u32) -> Self {
        Self::build(self.noise_type, self.seed, self.frequency, octaves)
    }

    /// Noise value in `0..=1`
    pub fn get(&self, location: V2) -> f32 {
        (self.inner.get_noise_2d(location.x, location.y) + 1.0) / 2.0
    }
    /// The noise as a scalar field, e.g. for [`crate::MarchingSquares`] or weighted relaxation.
    pub fn as_fn(&self) -> impl Fn(V2) -> f32 + '_ {
        move |location| self.get(location)
    }

    /// Noise value in `0..=1`
    pub fn get_3d(&self, location: V2, z: f32) -> f32 {
        (self.inner.get_noise_3d(location.x, location.y, z) + 1.0) / 2.0
    }

    fn build(noise_type: NoiseType, seed: i32, frequency: f32, octaves: u32) -> Self {
        let mut inner = FastNoiseLite::new();
        inner.set_noise_type(Some(match noise_type {
            NoiseType::Perlin => fastnoise_lite::NoiseType::Perlin,
            NoiseType::Simplex => fastnoise_lite::NoiseType::OpenSimplex2S,
            NoiseType::Worley => fastnoise_lite::NoiseType::Cellular,
        }));
        inner.set_seed(Some(seed));
        inner.set_frequency(Some(frequency));
        if octaves > 1 {
            inner.set_fractal_type(Some(FractalType::FBm));
            inner.set_fractal_octaves(Some(octaves as i32));
        }
        Self {
            noise_type,
            seed,
            frequency,
            octaves,
            inner,
        }
    }
}

impl Clone for Noise {
    fn clone(&self) -> Self {
        Self::build(self.noise_type, self.seed, self.frequency, self.octaves)
    }
}

impl fmt::Debug for Noise {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Noise")
            .field("noise_type", &self.noise_type)
            .field("seed", &self.seed)
            .field("frequency", &self.frequency)
            .field("octaves", &self.octaves)
            .finish()
    }
}
