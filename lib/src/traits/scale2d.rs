use crate::V2;

pub trait Scale2D {
    fn scale_2d(&self, factor: V2) -> Self;
    fn scale_2d_mut(&mut self, factor: V2);
}
