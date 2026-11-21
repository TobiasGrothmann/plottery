use serde::{Deserialize, Serialize};

use super::easing::Easing;

/// A value going from `from` to `to` along an [`Easing`], e.g. a radius that shrinks ring by ring.
///
/// Unlike [`Range`](crate::Range), `from` may be larger than `to`.
/// ```
/// # use plottery_lib::*;
/// let radius = Ramp::new(4.0, 1.0);
/// assert_eq!(radius.at(0.5), 2.5);
/// assert_eq!(radius.steps(4), vec![4.0, 3.0, 2.0, 1.0]);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ramp {
    pub from: f32,
    pub to: f32,
    pub easing: Easing,
}

impl Ramp {
    /// Linear from `from` to `to`.
    pub fn new(from: f32, to: f32) -> Self {
        Self {
            from,
            to,
            easing: Easing::linear(),
        }
    }

    #[must_use]
    pub fn with_easing(self, easing: Easing) -> Self {
        Self { easing, ..self }
    }

    /// `t` in `0..=1`, clamped.
    pub fn at(&self, t: f32) -> f32 {
        self.from + (self.to - self.from) * self.easing.apply(t)
    }

    /// `n` values from `from` to `to`, both included.
    pub fn steps(&self, n: usize) -> Vec<f32> {
        match n {
            0 => Vec::new(),
            1 => vec![self.from],
            _ => (0..n).map(|i| self.at(i as f32 / (n - 1) as f32)).collect(),
        }
    }
}

impl Default for Ramp {
    fn default() -> Self {
        Self::new(0.0, 1.0)
    }
}
