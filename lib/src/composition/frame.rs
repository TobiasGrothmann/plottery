use crate::{Layer, Path, Rect, V2};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CutGuideEdge {
    Top,
    Right,
    Bot,
    Left,
    TopBot,
    LeftRight,
    All,
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

    pub fn cut_guide(&self, edge: CutGuideEdge, start_from_edge: f32) -> Layer {
        let o = self.outer_rect();
        let mut guides = vec![];

        if o.width() >= start_from_edge * 2.0 {
            if matches!(
                edge,
                CutGuideEdge::Bot | CutGuideEdge::TopBot | CutGuideEdge::All
            ) {
                guides.push(Self::cut_guide_helper(o.bl(), o.br(), start_from_edge));
            }
            if matches!(
                edge,
                CutGuideEdge::Top | CutGuideEdge::TopBot | CutGuideEdge::All
            ) {
                guides.push(Self::cut_guide_helper(o.tl(), o.tr(), start_from_edge));
            }
        }

        if o.height() >= start_from_edge * 2.0 {
            if matches!(
                edge,
                CutGuideEdge::Left | CutGuideEdge::LeftRight | CutGuideEdge::All
            ) {
                guides.push(Self::cut_guide_helper(o.bl(), o.tl(), start_from_edge));
            }
            if matches!(
                edge,
                CutGuideEdge::Right | CutGuideEdge::LeftRight | CutGuideEdge::All
            ) {
                guides.push(Self::cut_guide_helper(o.br(), o.tr(), start_from_edge));
            }
        }

        Layer::new_from(guides)
    }

    pub fn cut_guide_left_right(&self, start_from_edge: f32) -> Layer {
        self.cut_guide(CutGuideEdge::LeftRight, start_from_edge)
    }

    pub fn cut_guide_top_bot(&self, start_from_edge: f32) -> Layer {
        self.cut_guide(CutGuideEdge::TopBot, start_from_edge)
    }

    fn cut_guide_helper(v1: V2, v2: V2, start_from_edge: f32) -> crate::Shape {
        Path::new_shape_from(vec![
            v1.towards(v2, start_from_edge),
            v2.towards(v1, start_from_edge),
        ])
    }
}
