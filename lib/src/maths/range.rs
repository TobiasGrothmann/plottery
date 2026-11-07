use serde::{Deserialize, Serialize};

use super::random::{rand_range, rand_range_i};

/// An inclusive `min..=max` interval, e.g. for randomised parameters.
///
/// Not to be confused with `std::ops::Range`, which is not in the prelude.
/// ```
/// # use plottery_lib::*;
/// let radius = Range::new(0.5, 2.0);
/// let r = radius.sample();
/// assert!(radius.contains(r));
/// assert_eq!(radius.lerp(0.5), 1.25);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Range<T> {
    pub min: T,
    pub max: T,
}

impl<T: PartialOrd + Copy> Range<T> {
    /// Swaps `min` and `max` if given in the wrong order.
    pub fn new(min: T, max: T) -> Self {
        if min <= max {
            Self { min, max }
        } else {
            Self { min: max, max: min }
        }
    }

    pub fn contains(&self, value: T) -> bool {
        self.min <= value && value <= self.max
    }

    pub fn clamp(&self, value: T) -> T {
        if value < self.min {
            self.min
        } else if value > self.max {
            self.max
        } else {
            value
        }
    }
}

impl Range<f32> {
    pub fn span(&self) -> f32 {
        self.max - self.min
    }
    /// Linear interpolation, `t` in `0..=1` maps to `min..=max`.
    pub fn lerp(&self, t: f32) -> f32 {
        self.min + self.span() * t
    }
    /// Uniform random value in the range, see [`rand_range`].
    pub fn sample(&self) -> f32 {
        if self.span() <= 0.0 {
            return self.min;
        }
        rand_range(self.min, self.max)
    }
}

impl Range<i32> {
    pub fn span(&self) -> i32 {
        self.max - self.min
    }
    /// Linear interpolation, `t` in `0..=1` maps to `min..=max`, rounded.
    pub fn lerp(&self, t: f32) -> i32 {
        (self.min as f32 + self.span() as f32 * t).round() as i32
    }
    /// Uniform random value in the range, both ends inclusive.
    pub fn sample(&self) -> i32 {
        rand_range_i(self.min, self.max + 1)
    }
}

impl Default for Range<f32> {
    fn default() -> Self {
        Self { min: 0.0, max: 1.0 }
    }
}

impl Default for Range<i32> {
    fn default() -> Self {
        Self { min: 0, max: 1 }
    }
}
