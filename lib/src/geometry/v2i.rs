use std::ops;

use serde::{Deserialize, Serialize};

use crate::V2;

/// A 2D vector with integer coordinates (x, y). See also [`V2`] for the float equivalent.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Serialize, Deserialize)]
pub struct V2i {
    pub x: i32,
    pub y: i32,
}

impl V2i {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Converts to a [`V2`] with x and y cast to float.
    pub fn to_float(&self) -> V2 {
        V2::new(self.x as f32, self.y as f32)
    }

    /// Returns a new V2i with the absolute values of x and y.
    #[must_use]
    pub fn abs(&self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
        }
    }

    /// Checks if this vector is (0, 0).
    pub fn is_zero(&self) -> bool {
        self.x == 0 && self.y == 0
    }
}

impl From<(i32, i32)> for V2i {
    fn from((x, y): (i32, i32)) -> Self {
        Self::new(x, y)
    }
}

impl From<V2i> for V2 {
    fn from(v: V2i) -> Self {
        v.to_float()
    }
}

impl ops::Add for V2i {
    type Output = V2i;
    fn add(self, rhs: V2i) -> V2i {
        V2i::new(self.x + rhs.x, self.y + rhs.y)
    }
}
impl ops::Sub for V2i {
    type Output = V2i;
    fn sub(self, rhs: V2i) -> V2i {
        V2i::new(self.x - rhs.x, self.y - rhs.y)
    }
}
impl ops::Neg for V2i {
    type Output = V2i;
    fn neg(self) -> V2i {
        V2i::new(-self.x, -self.y)
    }
}
impl ops::Mul<i32> for V2i {
    type Output = V2i;
    fn mul(self, rhs: i32) -> V2i {
        V2i::new(self.x * rhs, self.y * rhs)
    }
}
impl ops::AddAssign for V2i {
    fn add_assign(&mut self, rhs: V2i) {
        *self = *self + rhs;
    }
}
impl ops::SubAssign for V2i {
    fn sub_assign(&mut self, rhs: V2i) {
        *self = *self - rhs;
    }
}
