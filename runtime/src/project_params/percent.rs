use std::fmt;

use serde::{Deserialize, Serialize};

/// A share of a whole: a `0 % .. 100 %` slider in the editor, a fraction in code.
///
/// ```
/// # use plottery_runtime::Percent;
/// let density = Percent::new(25.0);
/// assert_eq!(density.fraction(), 0.25);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Percent {
    percent: f32,
}

impl Percent {
    /// From a number of percent, `50.0` is a half.
    pub fn new(percent: f32) -> Self {
        Self { percent }
    }

    /// From a fraction, `0.5` is 50 %.
    pub fn from_fraction(fraction: f32) -> Self {
        Self::new(fraction * 100.0)
    }

    pub fn percent(&self) -> f32 {
        self.percent
    }

    /// `0.5` for 50 %.
    pub fn fraction(&self) -> f32 {
        self.percent / 100.0
    }
}

impl fmt::Display for Percent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.percent)
    }
}
