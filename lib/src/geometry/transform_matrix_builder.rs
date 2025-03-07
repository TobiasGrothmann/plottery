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
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let transform = TransformMatrix::builder()
    ///     .scale_2d(&V2::new(2.0, 3.0))
    ///     .build();
    /// ```
    pub fn scale_2d(&mut self, scale: &V2) -> &mut Self {
        self.transforms.push(TransformMatrix::scale_2d(scale));
        self
    }

    /// Adds a uniform scale transformation using the same scale factor for both x and y.
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let transform = TransformMatrix::builder()
    ///     .scale(2.0)
    ///     .build();
    /// ```
    pub fn scale(&mut self, scalar: f32) -> &mut Self {
        self.transforms
            .push(TransformMatrix::scale_2d(&V2::xy(scalar)));
        self
    }

    /// Adds a rotation transformation by the specified angle.
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let transform = TransformMatrix::builder()
    ///     .rotate(&Angle::quarter_rotation())
    ///     .build();
    /// ```
    pub fn rotate(&mut self, angle: &Angle) -> &mut Self {
        self.transforms.push(TransformMatrix::rotate(angle));
        self
    }

    /// Adds a shear transformation, where `dist.x` controls horizontal shearing and `dist.y` controls vertical shearing.
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let transform = TransformMatrix::builder()
    ///     .shear(&V2::new(0.5, 0.0)) // Horizontal shear
    ///     .build();
    /// ```
    pub fn shear(&mut self, dist: &V2) -> &mut Self {
        self.transforms.push(TransformMatrix::shear(dist));
        self
    }

    /// Adds a transformation that mirrors across the x-axis.
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let transform = TransformMatrix::builder()
    ///     .mirror_x()
    ///     .build();
    /// ```
    pub fn mirror_x(&mut self) -> &mut Self {
        self.transforms.push(TransformMatrix::mirror_x());
        self
    }

    /// Adds a transformation that mirrors across the y-axis.
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let transform = TransformMatrix::builder()
    ///     .mirror_y()
    ///     .build();
    /// ```
    pub fn mirror_y(&mut self) -> &mut Self {
        self.transforms.push(TransformMatrix::mirror_y());
        self
    }

    /// Adds a translation transformation that moves points by the specified offset.
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let transform = TransformMatrix::builder()
    ///     .translate(&V2::new(10.0, 5.0))
    ///     .build();
    /// ```
    pub fn translate(&mut self, offset: &V2) -> &mut Self {
        self.transforms.push(TransformMatrix::translate(offset));
        self
    }
}
