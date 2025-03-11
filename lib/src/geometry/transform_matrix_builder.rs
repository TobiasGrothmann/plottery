use crate::{Angle, V2};

use super::TransformMatrix;

/// A builder for creating complex transformation matrices by combining multiple transformations.
///
/// Transformations are applied in the order they are added (first to last).
///
/// ### Example
/// ```
/// # use plottery_lib::*;
/// let transform = TransformMatrix::builder()
///     .scale_2d(&V2::new(2.0, 3.0))
///     .rotate(&Angle::from_degrees(45.0))
///     .translate(&V2::new(10.0, 5.0))
///     .build();
/// ```
///
/// see also [`TransformMatrix`]
#[derive(Debug, Clone, Default)]
pub struct TransformMatrixBuilder {
    transforms: Vec<TransformMatrix>,
}

impl TransformMatrixBuilder {
    /// Builds and returns the final transformation matrix by combining all added transformations.
    ///
    /// Transformations are combined in the order they were added.
    pub fn build(&self) -> TransformMatrix {
        TransformMatrix::combine_transforms(&self.transforms)
    }

    /// Adds a non-uniform scale transformation using different scale factors for x and y.
    pub fn scale_2d(mut self, scale: &V2) -> Self {
        self.transforms.push(TransformMatrix::scale_2d(scale));
        self
    }

    /// Adds a uniform scale transformation using the same scale factor for both x and y.
    pub fn scale(mut self, scalar: f32) -> Self {
        self.transforms
            .push(TransformMatrix::scale_2d(&V2::xy(scalar)));
        self
    }

    /// Adds a rotation transformation by the specified angle.
    pub fn rotate(mut self, angle: &Angle) -> Self {
        self.transforms.push(TransformMatrix::rotate(angle));
        self
    }

    /// Adds a shear transformation, where `dist.x` controls horizontal shearing and `dist.y` controls vertical shearing.
    pub fn shear(mut self, dist: &V2) -> Self {
        self.transforms.push(TransformMatrix::shear(dist));
        self
    }

    /// Adds a transformation that mirrors across the x-axis.
    pub fn mirror_x(mut self) -> Self {
        self.transforms.push(TransformMatrix::mirror_x());
        self
    }

    /// Adds a transformation that mirrors across the y-axis.
    pub fn mirror_y(mut self) -> Self {
        self.transforms.push(TransformMatrix::mirror_y());
        self
    }

    /// Adds a translation transformation that moves points by the specified offset.
    pub fn translate(mut self, offset: &V2) -> Self {
        self.transforms.push(TransformMatrix::translate(offset));
        self
    }
}
