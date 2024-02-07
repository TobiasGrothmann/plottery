pub trait Scale {
    fn scale(&self, factor: f32) -> Self;
    fn scale_mut(&mut self, factor: f32);
}
