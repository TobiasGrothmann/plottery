//! Contains functions for generating random numbers.
//! see also [`crate::maths::seed`]

use rand::seq::SliceRandom;
use rand::Rng;
use rand_distr::{Distribution, Normal, SkewNormal};

use super::thread_local::RNG;

/// uniform random float between `from` and `to`
pub fn rand_range(from: f32, to: f32) -> f32 {
    RNG.lock()
        .expect("Failed to acquire RNG lock")
        .gen_range(from..to)
}

/// uniform random integer between `from` and `to`
pub fn rand_range_i(from: i32, to: i32) -> i32 {
    RNG.lock()
        .expect("Failed to acquire RNG lock")
        .gen_range(from..to)
}

/// uniform random integer between `from` and `to`, both inclusive
///
/// unlike [`rand_range_i`], which excludes `to`
pub fn rand_range_i_inclusive(from: i32, to: i32) -> i32 {
    RNG.lock()
        .expect("Failed to acquire RNG lock")
        .gen_range(from..=to)
}

/// randomly `1.0` or `-1.0`
pub fn rand_sign() -> f32 {
    if coin(0.5) {
        1.0
    } else {
        -1.0
    }
}

/// uniform random index for a collection of `len` elements
///
/// panics if `len` is 0
pub fn rand_index(len: usize) -> usize {
    assert!(len > 0, "rand_index called with len 0");
    RNG.lock()
        .expect("Failed to acquire RNG lock")
        .gen_range(0..len)
}

/// uniform random element of `items`
///
/// panics if `items` is empty
pub fn rand_choice<T>(items: &[T]) -> &T {
    assert!(!items.is_empty(), "rand_choice called with an empty slice");
    items
        .choose(&mut *RNG.lock().expect("Failed to acquire RNG lock"))
        .expect("slice is not empty")
}

/// random boolean with given `chance` of being true
pub fn coin(chance: f32) -> bool {
    RNG.lock().expect("Failed to acquire RNG lock").gen::<f32>() < chance
}

/// see [`rand_distr::Normal`]
pub fn rand_normal(mean: f32, std_dev: f32) -> f32 {
    let normal = Normal::new(mean, std_dev).unwrap_or_else(|_| {
        panic!(
            "Invalid parameters for normal distribution: mean: {}, std_dev: {}",
            mean, std_dev
        )
    });
    normal.sample(&mut *RNG.lock().expect("Failed to acquire RNG lock"))
}

/// see [`rand_distr::SkewNormal`]
pub fn rand_normal_skewed(location: f32, scale: f32, shape: f32) -> f32 {
    let normal_skewed = SkewNormal::new(location, scale, shape).unwrap_or_else(|_| {
        panic!(
            "Invalid parameters for skewed normal distribution: location: {}, scale: {}, shape: {}",
            location, scale, shape
        )
    });
    normal_skewed.sample(&mut *RNG.lock().expect("Failed to acquire RNG lock"))
}

/// see [`rand_distr::Exp`]
pub fn rand_exponential(lambda: f32) -> f32 {
    let exponential = rand_distr::Exp::new(lambda).unwrap_or_else(|_| {
        panic!(
            "Invalid parameter for exponential distribution: lambda: {}",
            lambda
        )
    });
    exponential.sample(&mut *RNG.lock().expect("Failed to acquire RNG lock"))
}
