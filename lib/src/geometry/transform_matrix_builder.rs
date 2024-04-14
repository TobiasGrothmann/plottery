use crate::{Angle, V2};

use super::TransformMatrix;

#[derive(Debug, Clone, Default)]
pub struct TransformMatrixBuilder {
    transforms: Vec<TransformMatrix>,
}

impl TransformMatrixBuilder {
    pub fn build(&self) -> TransformMatrix {
        TransformMatrix::combine_transforms(&self.transforms)
    }

    pub fn scale_2d(&mut self, scale: &V2) -> &mut Self {
        self.transforms.push(TransformMatrix::scale_2d(scale));
        self
    }

    pub fn scale(&mut self, scalar: f32) -> &mut Self {
        self.transforms
            .push(TransformMatrix::scale_2d(&V2::xy(scalar)));
        self
    }

    pub fn rotate(&mut self, angle: &Angle) -> &mut Self {
        self.transforms.push(TransformMatrix::rotate(angle));
        self
    }

    pub fn shear(&mut self, dist: &V2) -> &mut Self {
        self.transforms.push(TransformMatrix::shear(dist));
        self
    }

    pub fn mirror_x(&mut self) -> &mut Self {
        self.transforms.push(TransformMatrix::mirror_x());
        self
    }

    pub fn mirror_y(&mut self) -> &mut Self {
        self.transforms.push(TransformMatrix::mirror_y());
        self
    }

    pub fn translate(&mut self, offset: &V2) -> &mut Self {
        self.transforms.push(TransformMatrix::translate(offset));
        self
    }
}
