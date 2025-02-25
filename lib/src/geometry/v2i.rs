use crate::V2;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct V2i {
    pub x: i32,
    pub y: i32,
}

impl V2i {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn to_float(&self) -> V2 {
        V2::new(self.x as f32, self.y as f32)
    }

    pub fn abs(&self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
        }
    }

    pub fn is_zero(&self) -> bool {
        self.x == 0 && self.y == 0
    }
}
