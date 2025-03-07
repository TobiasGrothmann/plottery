use crate::V2;

/// A 2D vector with integer coordinates (x, y). See also [`V2`] for the float equivalent.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct V2i {
    pub x: i32,
    pub y: i32,
}

impl V2i {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Converts to a [`V2`] with x and y cast to float.
    pub fn to_float(&self) -> V2 {
        V2::new(self.x as f32, self.y as f32)
    }

    /// Returns a new V2i with the absolute values of x and y.
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
