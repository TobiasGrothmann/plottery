use crate::V2;

/// 2D vector: `(x, y)` with `x` and `y` being integers. see also [`V2`]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct V2i {
    pub x: i32,
    pub y: i32,
}

impl V2i {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// new [`V2`] with `x` and `y` casted to float
    pub fn to_float(&self) -> V2 {
        V2::new(self.x as f32, self.y as f32)
    }

    /// new V2i with absolute values of `x` and `y` individually
    pub fn abs(&self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
        }
    }

    /// check if `(0, 0)`
    pub fn is_zero(&self) -> bool {
        self.x == 0 && self.y == 0
    }
}
