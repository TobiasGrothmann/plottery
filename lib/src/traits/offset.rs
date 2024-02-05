use crate::V2;

pub trait Offset {
    fn offset(&self, offset: &V2) -> Self;
    fn offset_inplace(&mut self, offset: &V2);
}
