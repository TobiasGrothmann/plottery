use crate::vec2::V2;

pub trait Rotate90 {
    fn rotate_90(&self) -> Self;
    fn rotate_180(&self) -> Self;
    fn rotate_270(&self) -> Self;

    fn rotate_90_around(&self, pivot: &V2) -> Self;
    fn rotate_180_around(&self, pivot: &V2) -> Self;
    fn rotate_270_around(&self, pivot: &V2) -> Self;
}
