use crate::Rect;

pub trait BoundingBox {
    fn bounding_box(&self) -> Option<Rect>;
}
