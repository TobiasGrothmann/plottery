use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::{rand_range, Interpolate, Interpolator, SampleSettings, GOLDEN_RATIO};

/// A struct for working with angles in different units (radians, degrees, or rotations).
///
/// Angles are stored internally as radians.
/// ```
/// # use plottery_lib::*;
/// let angle_deg = Angle::from_degrees(90.0);
/// let angle_rad = Angle::from_rad(std::f32::consts::PI / 2.0);
/// let angle_rot = Angle::from_rotations(0.25);
/// assert_eq!(angle_deg, angle_rad);
/// assert_eq!(angle_deg, angle_rot);
/// ```
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialOrd)]
pub struct Angle {
    rad: f32,
}

impl Angle {
    pub fn zero() -> Self {
        Self { rad: 0.0 }
    }
    pub fn from_rad(rad: f32) -> Self {
        Self { rad }
    }
    pub fn from_degrees(degree: f32) -> Self {
        Self {
            rad: (degree / 360.0) * 2.0 * PI,
        }
    }
    /// Creates a new angle from number of rotations.
    /// ```
    /// # use plottery_lib::*;
    /// let angle = Angle::from_rotations(0.25);
    /// assert_eq!(angle.to_degrees(), 90.0);
    /// ```
    pub fn from_rotations(rotations: f32) -> Self {
        Self {
            rad: rotations * 2.0 * PI,
        }
    }

    /// Creates a random angle between 0 and 2π (0° and 360°), see [`crate::seed`].
    pub fn rand() -> Self {
        Self {
            rad: rand_range(0.0, 2.0 * PI),
        }
    }

    pub fn quarter_rotation() -> Self {
        Self::from_rotations(0.25)
    }
    pub fn half_rotation() -> Self {
        Self::from_rotations(0.5)
    }
    pub fn full_rotation() -> Self {
        Self::from_rotations(1.0)
    }

    /// Returns an angle representing a direction to the right. Equivalent to [`Angle::zero()`]. see also [`crate::V2::right()`]
    pub fn right_cc() -> Self {
        Self::from_degrees(0.0)
    }
    /// Returns an angle representing a direction to the right. Equivalent to [`Angle::zero()`]. see also [`crate::V2::right()`]
    pub fn right_cw() -> Self {
        Self::from_degrees(0.0)
    }

    /// Returns an angle representing a direction upwards, counter clockwise (left / positive) from [`Angle::zero()`]. see also [`crate::V2::up()`]
    pub fn up_cc() -> Self {
        Self::from_degrees(90.0)
    }
    /// Returns an angle representing a direction upwards, clockwise (right / negative) from [`Angle::zero()`]. see also [`crate::V2::up()`]
    pub fn up_cw() -> Self {
        Self::from_degrees(-270.0)
    }

    /// Returns an angle representing a direction to the left, counter clockwise (left / positive) from  [`Angle::zero()`]. see also [`crate::V2::left()`]
    pub fn left_cc() -> Self {
        Self::from_degrees(180.0)
    }
    /// Returns an angle representing a direction to the left, clockwise (right / negative) from  [`Angle::zero()`]. see also [`crate::V2::left()`]
    pub fn left_cw() -> Self {
        Self::from_degrees(-180.0)
    }

    /// Returns an angle representing a direction downwards, counter clockwise (left / positive) from  [`Angle::zero()`]. see also [`crate::V2::down()`]
    pub fn down_cc() -> Self {
        Self::from_degrees(270.0)
    }
    /// Returns an angle representing a direction downwards, clockwise (right / negative) from  [`Angle::zero()`]. see also [`crate::V2::down()`]
    pub fn down_cw() -> Self {
        Self::from_degrees(-90.0)
    }

    /// The golden angle, ≈ 137.5°: the circle split in the golden ratio.
    pub fn golden_angle() -> Self {
        Self::from_rotations(1.0 - 1.0 / GOLDEN_RATIO)
    }

    pub fn to_rad(&self) -> f32 {
        self.rad
    }
    pub fn to_degrees(&self) -> f32 {
        360.0 * (self.rad / (2.0 * PI))
    }
    pub fn to_rotations(&self) -> f32 {
        self.rad / (2.0 * PI)
    }

    /// Gets both sine and cosine of this angle in radians: `(self.to_rad().sin(), self.to_rad().cos())`.
    pub fn rad_sin_cos(&self) -> (f32, f32) {
        (self.rad.sin(), self.rad.cos())
    }

    /// Returns a new angle modulo 2π (360°). The sign is kept, see [`Angle::positive`] for a result in 0..2π.
    #[must_use]
    pub fn mod_one_rotation(&self) -> Self {
        Angle::from_rad(self.rad % (2.0 * PI))
    }
    /// Returns a new angle modulo `other`. The sign is kept.
    #[must_use]
    pub fn modulo(&self, other: Angle) -> Self {
        Angle::from_rad(self.rad % other.rad)
    }

    /// Returns a new angle with the same direction but positive value.
    /// ```
    /// # use plottery_lib::*;
    /// let angle = Angle::from_degrees(-90.0);
    /// assert_eq!(angle.positive().to_degrees(), 270.0);
    /// ```
    #[must_use]
    pub fn positive(&self) -> Self {
        let rad = self.rad % (2.0 * PI);
        if rad < 0.0 {
            Angle::from_rad(rad + 2.0 * PI)
        } else {
            Angle::from_rad(rad)
        }
    }

    #[must_use]
    pub fn abs(&self) -> Self {
        Angle::from_rad(self.rad.abs())
    }

    /// The angle a quarter turn clockwise from this one, to its right like [`crate::Line::normal_right`].
    #[must_use]
    pub fn normal_right(&self) -> Self {
        self - Angle::quarter_rotation()
    }

    /// Interpolates between `self` and `other` with `t` as the interpolation factor.
    ///
    /// A value of `t = 0.0` returns `self`, a value of `t = 1.0` returns `other`.
    /// Values between `0.0` and `1.0` return points between `self` and `other`.
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let a1 = Angle::zero();
    /// let a2 = Angle::from_degrees(180.0);
    /// assert_eq!(a1.lerp(a2, 0.5), Angle::from_degrees(90.0));
    /// ```
    pub fn lerp(&self, end: Angle, t: f32) -> Angle {
        Angle::from_rad(self.rad * (1.0 - t) + end.rad * t)
    }

    /// Returns an iterator to interpolate from `self` to `end` in `steps` number of steps.
    ///
    /// The iterator will return `steps + 1` angles, including both `self` and `end`.
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let a1 = Angle::zero();
    /// let a2 = Angle::from_degrees(90.0);
    /// for angle in a1.lerp_iter_fixed(a2, 9) {
    ///     println!("{:?}", angle);
    /// }
    /// ```
    pub fn lerp_iter_fixed(&self, end: Angle, steps: usize) -> Interpolator<Angle> {
        Interpolator::new(*self, end, steps)
    }

    /// Returns an iterator to interpolate from `self` to `end`.
    ///
    /// The number of steps is determined by `sample_settings.points_per_unit` and the given `radius`.
    /// The arc length of the arc with radius `radius` is used as the distance between `self` and `end`.
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let a1 = Angle::zero();
    /// let a2 = Angle::from_degrees(90.0);
    /// for angle in a1.lerp_iter(a2, SampleSettings::new(10.0), 1.0) {
    ///     println!("{:?}", angle);
    /// }
    /// ```
    pub fn lerp_iter(
        &self,
        end: Angle,
        sample_settings: SampleSettings,
        radius: f32,
    ) -> Interpolator<Angle> {
        let distance = (end.rad - self.rad).abs() * radius;
        Interpolator::new(
            *self,
            end,
            sample_settings.num_points_for_length(distance) as usize,
        )
    }

    /// Returns a new angle that represents the smallest rotation to `other`.
    pub fn with_smallest_rotation_to(&self, other: Angle) -> Angle {
        let angle_diff_rotations = (self - other).to_rotations();
        if angle_diff_rotations > 0.5 {
            self - Angle::full_rotation()
        } else if angle_diff_rotations < -0.5 {
            self + Angle::full_rotation()
        } else {
            *self
        }
    }

    /// Returns the smallest angular difference between two angles, accounting for wrapping around a full rotation.
    pub fn dist_mod_one_rotation(&self, other: Angle) -> Angle {
        let angle_diff_abs =
            (other.mod_one_rotation().positive() - self.mod_one_rotation().positive()).abs();
        angle_diff_abs.min(Angle::full_rotation() - angle_diff_abs)
    }

    pub fn min(&self, other: Angle) -> Angle {
        if self.rad < other.rad {
            *self
        } else {
            other
        }
    }
    pub fn max(&self, other: Angle) -> Angle {
        if self.rad > other.rad {
            *self
        } else {
            other
        }
    }
}

impl Interpolate for Angle {
    fn interpolate(self, end: Self, step: usize, steps: usize) -> Self {
        let t = step as f32 / steps as f32;
        Angle::from_rad(self.rad * (1.0 - t) + end.rad * t)
    }
}

impl From<Angle> for f32 {
    fn from(angle: Angle) -> Self {
        angle.rad
    }
}
impl From<f32> for Angle {
    fn from(rad: f32) -> Self {
        Self { rad }
    }
}
