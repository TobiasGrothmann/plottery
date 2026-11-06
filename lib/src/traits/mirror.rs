pub trait Mirror: Sized {
    #[must_use]
    fn mirror_x(&self) -> Self;
    fn mirror_x_mut(&mut self) {
        *self = self.mirror_x();
    }

    #[must_use]
    fn mirror_y(&self) -> Self;
    fn mirror_y_mut(&mut self) {
        *self = self.mirror_y();
    }
}
