use crate::Rect;

pub trait BoundingBox {
    fn bounding_box(&self) -> Option<Rect>;
}

/// Above the touch tolerances of `Line::intersection` and `Line::contains_point_on_segment`.
const TOUCH_MARGIN: f32 = 0.01;

/// False when the boxes are apart, so no exact intersection or containment test can succeed.
pub(crate) fn bounds_may_touch(a: &impl BoundingBox, b: &impl BoundingBox) -> bool {
    match (a.bounding_box(), b.bounding_box()) {
        (Some(a), Some(b)) => a.expand(TOUCH_MARGIN).overlaps(&b),
        _ => false,
    }
}
