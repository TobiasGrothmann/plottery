use crate::{Rect, V2};

pub struct Frame {
    pub bottom_left: V2,
    pub size: V2,
    pub margin: f32,
}

impl Frame {
    pub fn new(size: V2, margin: f32) -> Self {
        Self {
            bottom_left: V2::xy(0.0),
            size,
            margin,
        }
    }
    pub fn new_at(bottom_left: V2, size: V2, margin: f32) -> Self {
        Self {
            bottom_left,
            size,
            margin,
        }
    }
    pub fn new_from_rect(rect: Rect, margin: f32) -> Self {
        Self {
            bottom_left: rect.bl(),
            size: rect.size(),
            margin,
        }
    }

    pub fn inner_rect(&self) -> Rect {
        Rect::new(
            self.bottom_left + self.margin,
            self.bottom_left + self.size - self.margin,
        )
    }

    pub fn outer_rect(&self) -> Rect {
        Rect::new(self.bottom_left, self.bottom_left + self.size)
    }
}
