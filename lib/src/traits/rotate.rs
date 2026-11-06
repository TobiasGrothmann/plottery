use crate::{Angle, V2};

use super::Translate;

pub trait Rotate: Translate {
    #[must_use]
    fn rotate(&self, angle: Angle) -> Self;
    fn rotate_mut(&mut self, angle: Angle) {
        *self = self.rotate(angle);
    }

    #[must_use]
    fn rotate_around(&self, pivot: V2, angle: Angle) -> Self {
        self.translate(-pivot).rotate(angle).translate(pivot)
    }
    fn rotate_around_mut(&mut self, pivot: V2, angle: Angle) {
        *self = self.rotate_around(pivot, angle);
    }
}
