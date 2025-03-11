pub trait Mirror {
    fn mirror_x(&self) -> Self;
    fn mirror_x_mut(&mut self);

    fn mirror_y(&self) -> Self;
    fn mirror_y_mut(&mut self);
}
