use crate::{Angle, V2};

use super::TransformMatrixBuilder;

/// A 2D transformation matrix represented in the following format:
/// ```text
/// | tl tr u |
/// | bl br v |
/// | 0  0  1 |
/// ```
///
/// This 3x3 matrix can represent various 2D transformations including:
/// - Translation
/// - Rotation
/// - Scaling
/// - Shearing
/// - Combinations of these transformations
///
/// The bottom row is implicit as [0, 0, 1] and not stored.
///
/// ### Example
/// ```
/// # use plottery_lib::*;
/// // Create a transform that rotates by 90 degrees and translates by (5, 10)
/// let transform = TransformMatrix::builder()
///     .rotate(&Angle::quarter_rotation())
///     .translate(&V2::new(5.0, 10.0))
///     .build();
///     
/// // Apply the transform to a point
/// let point = V2::new(1.0, 2.0);
/// let transformed = transform.mul_vector(&point);
/// ```
///
/// see [`TransformMatrixBuilder`]
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
    /// new [`TransformMatrixBuilder`] to construct complex transformations.
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
    pub fn builder() -> TransformMatrixBuilder {
        TransformMatrixBuilder::default()
    }

    /// identity transformation matrix that leaves points unchanged when applied:
    /// ```text
    /// | 1 0 0 |
    /// | 0 1 0 |
    /// | 0 0 1 |
    /// ```
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let identity = TransformMatrix::identity();
    /// let point = V2::new(3.0, 4.0);
    /// let result = identity.mul_vector(&point);
    /// assert_eq!(point, result); // Point remains unchanged
    /// ```
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

    /// new TransformMatrix that transforms a [`V2`] by multiplying its `x` and `y` coordinates by the corresponding
    /// scale factors in the given vector.
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let scale = TransformMatrix::scale_2d(&V2::new(2.0, 3.0));
    /// let point = V2::new(1.0, 1.0);
    /// let scaled = scale.mul_vector(&point);
    /// assert_eq!(scaled, V2::new(2.0, 3.0));
    /// ```
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

    /// new TransformMatrix that rotates points by the specified angle.
    ///
    /// The rotation is counter-clockwise for positive angles.
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let rotate = TransformMatrix::rotate(&Angle::quarter_rotation());
    /// let point = V2::new(1.0, 0.0);
    /// let rotated = rotate.mul_vector(&point);
    /// assert!((rotated.x - 0.0).abs() < 0.001);
    /// assert!((rotated.y - 1.0).abs() < 0.001);
    /// ```
    pub fn rotate(angle: &Angle) -> Self {
        let (sin, cos) = angle.rad_sin_cos();
        Self {
            tl: cos,
            bl: sin,
            tr: -sin,
            br: cos,
            u: 0.0,
            v: 0.0,
        }
    }

    /// new TransformMatrix that transforms a [`V2`] by shearing, where `dist.x` controls horizontal shearing, and `dist.y` controls vertical shearing.
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// // Create a horizontal shear of 1.0
    /// let shear = TransformMatrix::shear(&V2::new(1.0, 0.0));
    /// let point = V2::new(1.0, 1.0);
    /// let sheared = shear.mul_vector(&point);
    /// assert_eq!(sheared, V2::new(2.0, 1.0));
    /// ```
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

    /// new TransformMatrix that mirrors a [`V2`] across the x-axis.
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let mirror = TransformMatrix::mirror_x();
    /// let point = V2::new(2.0, 3.0);
    /// let mirrored = mirror.mul_vector(&point);
    /// assert_eq!(mirrored, V2::new(-2.0, 3.0));
    /// ```
    pub fn mirror_x() -> Self {
        Self {
            tl: -1.0,
            bl: 0.0,
            tr: 0.0,
            br: 1.0,
            u: 0.0,
            v: 0.0,
        }
    }

    /// new TransformMatrix that mirrors a [`V2`] across the y-axis.
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let mirror = TransformMatrix::mirror_y();
    /// let point = V2::new(2.0, 3.0);
    /// let mirrored = mirror.mul_vector(&point);
    /// assert_eq!(mirrored, V2::new(2.0, -3.0));
    /// ```
    pub fn mirror_y() -> Self {
        Self {
            tl: 1.0,
            bl: 0.0,
            tr: 0.0,
            br: -1.0,
            u: 0.0,
            v: 0.0,
        }
    }

    /// new TransformMatrix that moves points by the specified offset.
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let translate = TransformMatrix::translate(&V2::new(5.0, 10.0));
    /// let point = V2::new(2.0, 3.0);
    /// let translated = translate.mul_vector(&point);
    /// assert_eq!(translated, V2::new(7.0, 13.0));
    /// ```
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

    /// Multiplies two transformation matrices to create a new transformation that applies
    /// the effects of both matrices in sequence (first `other`, then `self`).
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// // First rotate, then translate
    /// let rotate = TransformMatrix::rotate(&Angle::quarter_rotation());
    /// let translate = TransformMatrix::translate(&V2::new(5.0, 0.0));
    ///
    /// // Combined transformation
    /// let combined = translate.mul_matrix(&rotate);
    ///
    /// // Apply to a point
    /// let point = V2::new(1.0, 0.0);
    /// let result = combined.mul_vector(&point);
    /// assert!((result.x - 5.0).abs() < 0.001);
    /// assert!((result.y - 1.0).abs() < 0.001);
    /// ```
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

    /// Applies the transformation to a vector.
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let transform = TransformMatrix::rotate(&Angle::quarter_rotation());
    /// let point = V2::new(1.0, 0.0);
    /// let transformed = transform.mul_vector(&point);
    /// assert!((transformed.x - 0.0).abs() < 0.001);
    /// assert!((transformed.y - 1.0).abs() < 0.001);
    /// ```
    pub fn mul_vector(&self, v: &V2) -> V2 {
        V2 {
            x: self.tl * v.x + self.tr * v.y + self.u,
            y: self.bl * v.x + self.br * v.y + self.v,
        }
    }

    /// Combines multiple transformations into a single transformation matrix.
    ///
    /// Transformations are applied in reverse order (from last to first in the array).
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let scale = TransformMatrix::scale_2d(&V2::new(2.0, 2.0));
    /// let rotate = TransformMatrix::rotate(&Angle::quarter_rotation());
    /// let translate = TransformMatrix::translate(&V2::new(5.0, 0.0));
    ///
    /// // First scale, then rotate, then translate
    /// let combined = TransformMatrix::combine_transforms(&[scale, rotate, translate]);
    ///
    /// // Apply to a point
    /// let point = V2::new(1.0, 0.0);
    /// let result = combined.mul_vector(&point);
    ///
    /// // Expected: scale by 2, rotate 90°, translate by (5,0)
    /// // (2,0) -> (0,2) -> (5,2)
    /// assert!((result.x - 5.0).abs() < 0.001);
    /// assert!((result.y - 2.0).abs() < 0.001);
    /// ```
    pub fn combine_transforms(transforms: &[TransformMatrix]) -> Self {
        transforms
            .iter()
            .rev()
            .fold(TransformMatrix::identity(), |acc, t| acc.mul_matrix(t))
    }
}
