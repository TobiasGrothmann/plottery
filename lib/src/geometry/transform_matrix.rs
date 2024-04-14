use crate::{Angle, V2};

use super::TransformMatrixBuilder;

// matrix in the format:
// tl tr u
// bl br v
// 0  0  1
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformMatrix {
    pub tl: f32,
    pub bl: f32,
    pub tr: f32,
    pub br: f32,
    pub u: f32,
    pub v: f32,
}

impl TransformMatrix {
    pub fn builder() -> TransformMatrixBuilder {
        TransformMatrixBuilder::default()
    }

    pub fn identity() -> Self {
        Self {
            tl: 1.0,
            bl: 0.0,
            tr: 0.0,
            br: 1.0,
            u: 0.0,
            v: 0.0,
        }
    }

    pub fn scale_2d(scale: &V2) -> Self {
        Self {
            tl: scale.x,
            bl: 0.0,
            tr: 0.0,
            br: scale.y,
            u: 0.0,
            v: 0.0,
        }
    }

    pub fn rotate(angle: &Angle) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self {
            tl: cos,
            bl: sin,
            tr: -sin,
            br: cos,
            u: 0.0,
            v: 0.0,
        }
    }

    pub fn shear(dist: &V2) -> Self {
        Self {
            tl: 1.0,
            bl: dist.y,
            tr: dist.x,
            br: 1.0,
            u: 0.0,
            v: 0.0,
        }
    }

    pub fn mirror_x() -> Self {
        Self {
            tl: 1.0,
            bl: 0.0,
            tr: 0.0,
            br: -1.0,
            u: 0.0,
            v: 0.0,
        }
    }

    pub fn mirror_y() -> Self {
        Self {
            tl: -1.0,
            bl: 0.0,
            tr: 0.0,
            br: 1.0,
            u: 0.0,
            v: 0.0,
        }
    }

    pub fn translate(offset: &V2) -> Self {
        Self {
            tl: 1.0,
            bl: 0.0,
            tr: 0.0,
            br: 1.0,
            u: offset.x,
            v: offset.y,
        }
    }

    pub fn mul_matrix(&self, other: &TransformMatrix) -> Self {
        Self {
            tl: self.tl * other.tl + self.tr * other.bl,
            bl: self.bl * other.tl + self.br * other.bl,
            tr: self.tl * other.tr + self.tr * other.br,
            br: self.bl * other.tr + self.br * other.br,
            u: self.tl * other.u + self.tr * other.v + self.u,
            v: self.bl * other.u + self.br * other.v + self.v,
        }
    }

    pub fn mul_vector(&self, v: &V2) -> V2 {
        V2 {
            x: self.tl * v.x + self.tr * v.y + self.u,
            y: self.bl * v.x + self.br * v.y + self.v,
        }
    }

    pub fn combine_transforms(transforms: &[TransformMatrix]) -> Self {
        transforms
            .iter()
            .rev()
            .fold(TransformMatrix::identity(), |acc, t| acc.mul_matrix(t))
    }
}
