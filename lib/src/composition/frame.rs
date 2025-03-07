use crate::{Rect, V2};

/// A rectangular frame with a specified size, position, and margin.
///
/// The frame defines both an outer rectangle (the full frame) and an inner rectangle
/// (the area inside the margins).
///
/// ```
/// # use plottery_lib::*;
/// let size = V2::a4(); // A4 paper size
/// let frame = Frame::new(size, size.min_axis() * 0.1); // 10% margin of the smallest axis
/// let frame_inner = frame.inner_rect();
/// let frame_outer = frame.outer_rect();
/// ```
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
    /// Creates a new frame at the specified position with the given size and margin.
    pub fn new_at(bottom_left: V2, size: V2, margin: f32) -> Self {
        Self {
            bottom_left,
            size,
            margin,
        }
    }
    /// Creates a new frame from an existing rectangle with the specified margin.
    pub fn new_from_rect(rect: Rect, margin: f32) -> Self {
        Self {
            bottom_left: rect.bl(),
            size: rect.size(),
            margin,
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
}
