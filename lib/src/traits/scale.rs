use crate::V2;

use super::Translate;

pub trait Scale: Translate {
    #[must_use]
    fn scale(&self, factor: f32) -> Self;
    fn scale_mut(&mut self, factor: f32) {
        *self = self.scale(factor);
    }

    /// Scales with `pivot` staying in place.
    #[must_use]
    fn scale_around(&self, pivot: V2, factor: f32) -> Self {
        self.translate(-pivot).scale(factor).translate(pivot)
    }
    fn scale_around_mut(&mut self, pivot: V2, factor: f32) {
        *self = self.scale_around(pivot, factor);
    }
}
