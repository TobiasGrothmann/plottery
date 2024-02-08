use crate::{Angle, V2};

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

    pub fn scale_2d(x: f32, y: f32) -> Self {
        Self {
            tl: x,
            bl: 0.0,
            tr: 0.0,
            br: y,
            u: 0.0,
            v: 0.0,
        }
    }

    pub fn scale(scalar: f32) -> Self {
        Self::scale_2d(scalar, scalar)
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

    pub fn shear(x: f32, y: f32) -> Self {
        Self {
            tl: 1.0,
            bl: y,
            tr: x,
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

    pub fn translate(x: f32, y: f32) -> Self {
        Self {
            tl: 1.0,
            bl: 0.0,
            tr: 0.0,
            br: 1.0,
            u: x,
            v: y,
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
            .fold(TransformMatrix::identity(), |acc, t| acc.mul_matrix(t))
    }
}
