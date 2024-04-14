use crate::{Angle, V2};

pub trait Rotate {
    fn rotate(&self, angle: &Angle) -> Self;
    fn rotate_mut(&mut self, angle: &Angle);

    fn rotate_around(&self, pivot: &V2, angle: &Angle) -> Self;
    fn rotate_around_mut(&mut self, pivot: &V2, angle: &Angle);
}
