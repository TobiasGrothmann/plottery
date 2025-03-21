//! Contains functions for generating random numbers.
//! see also [`crate::maths::seed`]

use rand::Rng;
use rand_distr::{Distribution, Normal, SkewNormal};

use super::thread_local::RNG;

/// uniform random float between `from` and `to`
pub fn rand_range(from: f32, to: f32) -> f32 {
    RNG.with_borrow_mut(|rng: &mut rand::prelude::StdRng| rng.gen_range(from..to))
}

/// uniform random integer between `from` and `to`
pub fn rand_range_i(from: i32, to: i32) -> i32 {
    RNG.with_borrow_mut(|rng: &mut rand::prelude::StdRng| rng.gen_range(from..to))
}

/// random boolean with given `chance` of being true
pub fn coin(chance: f32) -> bool {
    RNG.with_borrow_mut(|rng| rng.gen::<f32>() < chance)
}

/// see [`rand_distr::Normal`]
pub fn rand_normal(mean: f32, std_dev: f32) -> f32 {
    let normal = Normal::new(mean, std_dev).unwrap_or_else(|_| {
        panic!(
            "Invalid parameters for normal distribution: mean: {}, std_dev: {}",
            mean, std_dev
        )
    });
    RNG.with_borrow_mut(|rng| normal.sample(rng))
}

/// see [`rand_distr::SkewNormal`]
pub fn rand_normal_skewed(location: f32, scale: f32, shape: f32) -> f32 {
    let normal_skewed = SkewNormal::new(location, scale, shape).unwrap_or_else(|_| {
        panic!(
            "Invalid parameters for skewed normal distribution: location: {}, scale: {}, shape: {}",
            location, scale, shape
        )
    });
    RNG.with_borrow_mut(|rng| normal_skewed.sample(rng))
}

/// see [`rand_distr::Exp`]
pub fn rand_exponential(lambda: f32) -> f32 {
    let exponential = rand_distr::Exp::new(lambda).unwrap_or_else(|_| {
        panic!(
            "Invalid parameter for exponential distribution: lambda: {}",
            lambda
        )
    });
    RNG.with_borrow_mut(|rng| exponential.sample(rng))
}
