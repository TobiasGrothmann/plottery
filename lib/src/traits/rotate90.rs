use crate::V2;

use super::Translate;

pub trait Rotate90: Translate {
    #[must_use]
    fn rotate_90(&self) -> Self;
    fn rotate_90_mut(&mut self) {
        *self = self.rotate_90();
    }
    #[must_use]
    fn rotate_180(&self) -> Self {
        self.rotate_90().rotate_90()
    }
    fn rotate_180_mut(&mut self) {
        *self = self.rotate_180();
    }
    #[must_use]
    fn rotate_270(&self) -> Self {
        self.rotate_180().rotate_90()
    }
    fn rotate_270_mut(&mut self) {
        *self = self.rotate_270();
    }

    #[must_use]
    fn rotate_90_around(&self, pivot: V2) -> Self {
        self.translate(-pivot).rotate_90().translate(pivot)
    }
    fn rotate_90_around_mut(&mut self, pivot: V2) {
        *self = self.rotate_90_around(pivot);
    }
    #[must_use]
    fn rotate_180_around(&self, pivot: V2) -> Self {
        self.translate(-pivot).rotate_180().translate(pivot)
    }
    fn rotate_180_around_mut(&mut self, pivot: V2) {
        *self = self.rotate_180_around(pivot);
    }
    #[must_use]
    fn rotate_270_around(&self, pivot: V2) -> Self {
        self.translate(-pivot).rotate_270().translate(pivot)
    }
    fn rotate_270_around_mut(&mut self, pivot: V2) {
        *self = self.rotate_270_around(pivot);
    }
}
