use crate::V2;

pub trait Rotate90 {
    fn rotate_90(&self) -> Self;
    fn rotate_90_inplace(&mut self);
    fn rotate_180(&self) -> Self;
    fn rotate_180_inplace(&mut self);
    fn rotate_270(&self) -> Self;
    fn rotate_270_inplace(&mut self);

    fn rotate_90_around(&self, pivot: &V2) -> Self;
    fn rotate_90_around_inplace(&mut self, pivot: &V2);
    fn rotate_180_around(&self, pivot: &V2) -> Self;
    fn rotate_180_around_inplace(&mut self, pivot: &V2);
    fn rotate_270_around(&self, pivot: &V2) -> Self;
    fn rotate_270_around_inplace(&mut self, pivot: &V2);
}
