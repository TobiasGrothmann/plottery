use crate::geometry::TransformMatrix;

pub trait Transform: Sized {
    #[must_use]
    fn transform(&self, matrix: &TransformMatrix) -> Self;
    fn transform_mut(&mut self, matrix: &TransformMatrix) {
        *self = self.transform(matrix);
    }
}
