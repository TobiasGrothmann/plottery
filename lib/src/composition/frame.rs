use crate::{Rect, V2};

/// A rectangular frame with a specified size, position, and margin.
///
/// The frame defines both an outer rectangle (the full frame) and an inner rectangle
/// (the area inside the margins).
///
/// ```
/// # use plottery_lib::*;
/// let size = V2::a4(); // A4 paper size
/// let frame = Frame::new_xy(size, size.min_axis() * 0.1); // 10% margin of the smallest axis
/// let frame_inner = frame.inner_rect();
/// let frame_outer = frame.outer_rect();
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub bottom_left: V2,
    pub size: V2,
    pub margin: V2,
}

impl Frame {
    pub fn new(size: V2, margin: V2) -> Self {
        Self {
            bottom_left: V2::xy(0.0),
            size,
            margin,
        }
    }
    pub fn new_at(bottom_left: V2, size: V2, margin: V2) -> Self {
        Self {
            bottom_left,
            size,
            margin,
        }
    }
    pub fn new_from_rect(rect: Rect, margin: V2) -> Self {
        Self {
            bottom_left: rect.bl(),
            size: rect.size(),
            margin,
        }
    }

    /// Creates a new `Frame` with a symmetric margin.
    pub fn new_xy(size: V2, margin: f32) -> Self {
        Self {
            bottom_left: V2::xy(0.0),
            size,
            margin: V2::xy(margin),
        }
    }
    /// Creates a new `Frame` with a symmetric margin.
    pub fn new_at_xy(bottom_left: V2, size: V2, margin: f32) -> Self {
        Self {
            bottom_left,
            size,
            margin: V2::xy(margin),
        }
    }
    /// Creates a new `Frame` with a symmetric margin.
    pub fn new_from_rect_xy(rect: Rect, margin: f32) -> Self {
        Self {
            bottom_left: rect.bl(),
            size: rect.size(),
            margin: V2::xy(margin),
        }
    }

    /// Returns the inner rectangle of the frame (the area inside the margins).
    pub fn inner_rect(&self) -> Rect {
        Rect::new(
            self.bottom_left + self.margin,
            self.bottom_left + self.size - self.margin,
        )
    }
    /// Returns the outer rectangle of the frame (the full frame area).
    pub fn outer_rect(&self) -> Rect {
        Rect::new(self.bottom_left, self.bottom_left + self.size)
    }

    pub fn center(&self) -> V2 {
        self.outer_rect().center()
    }
}
