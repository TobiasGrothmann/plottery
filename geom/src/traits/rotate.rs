use crate::{angle::angle::Angle, vec2::V2};

pub trait Rotate {
    fn rotate(&self, angle: &Angle) -> Self;
    fn rotate_around(&self, pivot: &V2, angle: &Angle) -> Self;
}
