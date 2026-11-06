use crate::V2;

pub trait Translate: Sized {
    #[must_use]
    fn translate(&self, dist: V2) -> Self;
    fn translate_mut(&mut self, dist: V2) {
        *self = self.translate(dist);
    }
}
