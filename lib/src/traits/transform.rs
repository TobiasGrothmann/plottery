use crate::geometry::TransformMatrix;

pub trait Transform {
    fn transform(&self, matrix: &TransformMatrix) -> Self;
    fn transform_mut(&mut self, matrix: &TransformMatrix);
}
