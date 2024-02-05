pub trait Scale {
    fn scale(&self, factor: f32) -> Self;
    fn scale_inplace(&mut self, factor: f32);
}
