use std::{f32::consts::PI, iter::Sum};

use geo_types::Coord;
use mint::Point2;
use serde::{Deserialize, Serialize};

use crate::{rand_range, Angle, Rect, Rotate, Rotate90, SampleSettings};

use super::v2i::V2i;

/// 2D vector: `(x, y)`. see also [`V2i`]
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

impl V2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    /// new V2 with the same x and y value
    pub fn xy(x_and_y: f32) -> Self {
        Self {
            x: x_and_y,
            y: x_and_y,
        }
    }
    pub fn new_from_geo(geo_coord: &Coord<f32>) -> Self {
        Self {
            x: geo_coord.x,
            y: geo_coord.y,
        }
    }
    /// new V2 from polar coordinates
    pub fn polar(angle: Angle, distance: f32) -> Self {
        Self {
            x: angle.to_rad().cos() * distance,
            y: angle.to_rad().sin() * distance,
        }
    }
    /// origin vector
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
    /// vector of length 1.0 pointing up
    pub fn up() -> Self {
        Self { x: 0.0, y: 1.0 }
    }
    /// vector of length 1.0 pointing right
    pub fn right() -> Self {
        Self { x: 1.0, y: 0.0 }
    }
    /// vector of length 1.0 pointing down
    pub fn down() -> Self {
        Self { x: 0.0, y: -1.0 }
    }
    /// vector of length 1.0 pointing left
    pub fn left() -> Self {
        Self { x: 0.0, y: -1.0 }
    }

    /// new V2 randomly exactly on the unit circle
    pub fn random_unit_circle() -> Self {
        Self::polar(Angle::rand(), 1.0)
    }
    /// new V2 randomly inside or on the unit circle
    pub fn random_unit_disk() -> Self {
        let angle = Angle::rand();
        let radius = rand_range(0.0, 1.0);
        Self::polar(angle, radius)
    }
    /// new V2 randomly inside or on a given rectangle
    pub fn random_in_rect(rect: &Rect) -> Self {
        Self::new(
            rand_range(rect.bl().x, rect.tr().x),
            rand_range(rect.bl().y, rect.tr().y),
        )
    }

    /// new V2 given by [DIN 476-2](https://en.wikipedia.org/wiki/Paper_size) A (internationally [ISO 216](https://en.wikipedia.org/wiki/ISO_216))
    pub fn din_a(number: u8) -> Self {
        match number {
            0 => Self::a0(),
            1 => Self::a1(),
            2 => Self::a2(),
            3 => Self::a3(),
            4 => Self::a4(),
            5 => Self::a5(),
            6 => Self::a6(),
            7 => Self::a7(),
            8 => Self::a8(),
            9 => Self::a9(),
            10 => Self::a10(),
            _ => panic!("DIN A number out of range."),
        }
    }
    /// new V2 given by A0 [DIN 476-2](https://en.wikipedia.org/wiki/Paper_size) / [ISO 216](https://en.wikipedia.org/wiki/ISO_216)
    pub fn a0() -> Self {
        Self { x: 84.1, y: 118.9 }
    }
    /// new V2 given by A1 [DIN 476-2](https://en.wikipedia.org/wiki/Paper_size) / [ISO 216](https://en.wikipedia.org/wiki/ISO_216)
    pub fn a1() -> Self {
        Self { x: 59.4, y: 84.1 }
    }
    /// new V2 given by A2 [DIN 476-2](https://en.wikipedia.org/wiki/Paper_size) / [ISO 216](https://en.wikipedia.org/wiki/ISO_216)
    pub fn a2() -> Self {
        Self { x: 42.0, y: 59.4 }
    }
    /// new V2 given by A3 [DIN 476-2](https://en.wikipedia.org/wiki/Paper_size) / [ISO 216](https://en.wikipedia.org/wiki/ISO_216)
    pub fn a3() -> Self {
        Self { x: 29.7, y: 42.0 }
    }
    /// new V2 given by A4 [DIN 476-2](https://en.wikipedia.org/wiki/Paper_size) / [ISO 216](https://en.wikipedia.org/wiki/ISO_216)
    pub fn a4() -> Self {
        Self { x: 21.0, y: 29.7 }
    }
    /// new V2 given by A5 [DIN 476-2](https://en.wikipedia.org/wiki/Paper_size) / [ISO 216](https://en.wikipedia.org/wiki/ISO_216)
    pub fn a5() -> Self {
        Self { x: 14.8, y: 21.0 }
    }
    /// new V2 given by A6 [DIN 476-2](https://en.wikipedia.org/wiki/Paper_size) / [ISO 216](https://en.wikipedia.org/wiki/ISO_216)
    pub fn a6() -> Self {
        Self { x: 10.5, y: 14.8 }
    }
    /// new V2 given by A7 [DIN 476-2](https://en.wikipedia.org/wiki/Paper_size) / [ISO 216](https://en.wikipedia.org/wiki/ISO_216)
    pub fn a7() -> Self {
        Self { x: 7.4, y: 10.5 }
    }
    /// new V2 given by A8 [DIN 476-2](https://en.wikipedia.org/wiki/Paper_size) / [ISO 216](https://en.wikipedia.org/wiki/ISO_216)
    pub fn a8() -> Self {
        Self { x: 5.2, y: 7.4 }
    }
    /// new V2 given by A9 [DIN 476-2](https://en.wikipedia.org/wiki/Paper_size) / [ISO 216](https://en.wikipedia.org/wiki/ISO_216)
    pub fn a9() -> Self {
        Self { x: 3.7, y: 5.2 }
    }
    /// new V2 given by A10 [DIN 476-2](https://en.wikipedia.org/wiki/Paper_size) / [ISO 216](https://en.wikipedia.org/wiki/ISO_216)
    pub fn a10() -> Self {
        Self { x: 2.6, y: 3.7 }
    }

    /// new V2 with x and y values swapped
    pub fn swap(&self) -> Self {
        Self {
            x: self.y,
            y: self.x,
        }
    }

    /// new V2 with y set to 0.0
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let v = V2::new(1.0, 2.0);
    /// assert_eq!(v.only_x(), V2::new(1.0, 0.0));
    /// ```
    pub fn only_x(&self) -> Self {
        Self { x: self.x, y: 0.0 }
    }
    /// new V2 with x set to 0.0
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let v = V2::new(1.0, 2.0);
    /// assert_eq!(v.only_y(), V2::new(0.0, 2.0));
    /// ```
    pub fn only_y(&self) -> Self {
        Self { x: 0.0, y: self.y }
    }

    pub fn as_geo_coord(&self) -> Coord<f32> {
        Coord {
            x: self.x,
            y: self.y,
        }
    }
    /// tuple of `(x, y)`
    pub fn as_tuple(&self) -> (f32, f32) {
        (self.x, self.y)
    }
    /// array of `[x, y]`
    pub fn as_array(&self) -> [f32; 2] {
        [self.x, self.y]
    }
    /// vector of `vec![x, y]`
    pub fn as_vec(&self) -> Vec<f32> {
        vec![self.x, self.y]
    }

    /// new V2 with x and y values rounded individually
    pub fn round(&self) -> Self {
        Self {
            x: self.x.round(),
            y: self.y.round(),
        }
    }
    /// new V2i with x and y values rounded to integer individually
    pub fn round_to_int(&self) -> V2i {
        V2i::new(self.x.round() as i32, self.y.round() as i32)
    }
    /// new V2 with x and y values ceiled individually
    pub fn ceil(&self) -> Self {
        Self {
            x: self.x.ceil(),
            y: self.y.ceil(),
        }
    }
    /// new V2i with x and y values ceiled to integer individually
    pub fn ceil_to_int(&self) -> V2i {
        V2i::new(self.x.ceil() as i32, self.y.ceil() as i32)
    }
    /// new V2 with x and y values floored individually
    pub fn floor(&self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
        }
    }
    /// new V2i with x and y values floored to integer individually
    pub fn floor_to_int(&self) -> V2i {
        V2i::new(self.x.floor() as i32, self.y.floor() as i32)
    }

    /// Calculates the dot product of two 2D vectors.
    ///
    /// The dot product of vectors **a** and **b** is defined as:
    ///
    /// ```text
    /// a · b = a.x * b.x + a.y * b.y
    /// ```
    ///
    /// ### Key Properties
    /// - If `a · b > 0`, the vectors point in a similar direction.
    /// - If `a · b < 0`, the vectors point in opposite directions.
    /// - If `a · b == 0`, the vectors are **perpendicular** (orthogonal).
    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// new V2 with the new x and y set to the minimum of the old x and y individually (`x.min(x), y.min(y)`)
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let v1 = V2::new(1.0, 2.0);
    /// let v2 = V2::new(2.0, 1.0);
    /// assert_eq!(v1.min(&v2), V2::new(1.0, 1.0));
    /// ```
    pub fn min(&self, other: &Self) -> Self {
        V2::new(self.x.min(other.x), self.y.min(other.y))
    }
    /// new V2 with the new x and y set to the maximum of the old x and y individually (`x.max(x), y.max(y)`)
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let v1 = V2::new(1.0, 2.0);
    /// let v2 = V2::new(2.0, 1.0);
    /// assert_eq!(v1.max(&v2), V2::new(2.0, 2.0));
    /// ```
    pub fn max(&self, other: &Self) -> Self {
        V2::new(self.x.max(other.x), self.y.max(other.y))
    }

    /// value of the smaller of either x and y (`x.min(y)`)
    pub fn min_axis(&self) -> f32 {
        self.x.min(self.y)
    }
    /// value of the larger of either x and y (`x.max(y)`)
    pub fn max_axis(&self) -> f32 {
        self.x.max(self.y)
    }

    /// euclidean distance to `other`
    pub fn dist(&self, other: &Self) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
    /// squared euclidean distance to `other`
    pub fn dist_squared(&self, other: &Self) -> f32 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }
    /// The Manhattan distance (or **taxicab distance**) between points `a` and `b` is given by the summed difference of their coordinates.
    /// ```text
    /// d = |a.x - b.x| + |a.y - b.y|
    /// ```
    ///
    /// ### Characteristics:
    /// - Measures distance when movement is restricted to horizontal and vertical directions.
    /// - Unlike Euclidean distance, it does not account for diagonal movement.
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let v1 = V2::new(0.0, 0.0);
    /// let v2 = V2::new(2.0, 2.0);
    /// assert_eq!(v1.dist_manhattan(&v2), 4.0);
    /// ```
    pub fn dist_manhattan(&self, other: &Self) -> f32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    /// length of the vector
    pub fn len(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    /// squared length of the vector
    pub fn len_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// angle of the vector
    pub fn angle(&self) -> Angle {
        let mut rad = self.y.atan2(self.x);
        if rad < 0.0 {
            rad += 2.0 * PI;
        }
        Angle::from_rad(rad)
    }
    /// angle of the vector going from self to `other`
    pub fn angle_to(&self, other: &Self) -> Angle {
        (other - self).angle()
    }

    /// new V2 with same angle but length of 1.0
    pub fn normalize(&self) -> Self {
        let len = self.len();
        if len == 0.0 {
            *self
        } else {
            *self / len
        }
    }
    /// new V2 with same angle but length of `len`
    pub fn normalize_to(&self, len: f32) -> Self {
        *self * len / self.len()
    }

    /// project a point onto the infinite line defined by the vector
    pub fn project_onto(&self, other: &Self) -> Self {
        let length_squared = other.len_squared();
        let dot_product = self.dot(other);
        V2::new(
            (dot_product / length_squared) * other.x,
            (dot_product / length_squared) * other.y,
        )
    }

    /// new V2 with same angle but a length between `min_len` and `max_len` inclusive
    pub fn clamp_len(&self, min_len: f32, max_len: f32) -> Self {
        let len = self.len();
        if len < min_len {
            *self * (min_len / len)
        } else if len > max_len {
            *self * (max_len / len)
        } else {
            *self
        }
    }

    /// new V2 with a function `f` applied to both x and y
    pub fn map(&self, f: fn(f32) -> f32) -> Self {
        V2::new(f(self.x), f(self.y))
    }

    /// new V2 with both x and y values as square root of the old x and y values individually
    pub fn sqrt(&self) -> Self {
        V2::new(self.x.sqrt(), self.y.sqrt())
    }

    /// new V2 on the line from `self` to `other` with `t` as the interpolation factor.
    /// a value of `t = 0.0` returns `self`, a value of `t = 1.0` returns `other`
    /// values between `0.0` and `1.0` return points between `self` and `other`
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let v1 = V2::new(0.0, 0.0);
    /// let v2 = V2::new(2.0, 2.0);
    /// assert_eq!(v1.lerp(&v2, 0.5), V2::new(1.0, 1.0));
    /// ```
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        V2::new(
            self.x + t * (other.x - self.x),
            self.y + t * (other.y - self.y),
        )
    }
    /// iterator to lerp from `self` to `end` in `steps` number of steps.
    /// The iterator will return `steps + 1` `V2`s, because both `self` and `end` are included.
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let start = V2::new(0.0, 0.0);
    /// let end = V2::new(10.0, 10.0);
    /// for point in start.lerp_iter_fixed(end, 10) {
    ///     println!("{:?}", point);
    /// }
    /// ```
    pub fn lerp_iter_fixed(&self, end: V2, steps: usize) -> V2Interpolator {
        V2Interpolator::new(*self, end, steps)
    }
    /// iterator to lerp from `self` to `end`. The number of steps are determined by the distance and `sample_settings.points_per_unit`.
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let start = V2::new(0.0, 0.0);
    /// let end = V2::new(10.0, 10.0);
    /// for point in start.lerp_iter(end, &SampleSettings::new(5.0)) {
    ///     println!("{:?}", point);
    /// }
    /// ```
    pub fn lerp_iter(&self, end: V2, sample_settings: &SampleSettings) -> V2Interpolator {
        let distance = self.dist(&end);
        V2Interpolator::new(
            *self,
            end,
            sample_settings.get_num_points_for_length(distance) as usize,
        )
    }
}

pub struct V2Interpolator {
    start: V2,
    end: V2,
    steps: usize,
    current_step: usize,
}

impl V2Interpolator {
    pub fn new(start: V2, end: V2, steps: usize) -> Self {
        Self {
            start,
            end,
            steps,
            current_step: 0,
        }
    }
}

impl Iterator for V2Interpolator {
    type Item = V2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_step > self.steps {
            return None;
        }
        let t = self.current_step as f32 / self.steps as f32;
        let interpolated = self.start.lerp(&self.end, t);
        self.current_step += 1;
        Some(interpolated)
    }
}

impl Rotate for V2 {
    fn rotate(&self, angle: &Angle) -> Self {
        let angle = angle.to_rad();
        let angle_sin = angle.sin();
        let angle_cos = angle.cos();
        Self::new(
            self.x * angle_cos - self.y * angle_sin,
            self.x * angle_sin + self.y * angle_cos,
        )
    }
    fn rotate_mut(&mut self, angle: &Angle) {
        *self = self.rotate(angle);
    }

    fn rotate_around(&self, pivot: &V2, angle: &Angle) -> Self {
        let angle = angle.to_rad();
        let angle_sin = angle.sin();
        let angle_cos = angle.cos();

        let x_offset = self.x - pivot.x;
        let y_offset = self.y - pivot.y;

        Self::new(
            (x_offset * angle_cos - y_offset * angle_sin) + pivot.x,
            (x_offset * angle_sin + y_offset * angle_cos) + pivot.y,
        )
    }
    fn rotate_around_mut(&mut self, pivot: &V2, angle: &Angle) {
        *self = self.rotate_around(pivot, angle);
    }
}

impl Rotate90 for V2 {
    fn rotate_90(&self) -> Self {
        Self::new(-self.y, self.x)
    }
    fn rotate_90_mut(&mut self) {
        *self = self.rotate_90();
    }

    fn rotate_180(&self) -> Self {
        Self::new(-self.x, -self.y)
    }
    fn rotate_180_mut(&mut self) {
        *self = self.rotate_180();
    }

    fn rotate_270(&self) -> Self {
        Self::new(self.y, -self.x)
    }
    fn rotate_270_mut(&mut self) {
        *self = self.rotate_270();
    }

    fn rotate_90_around(&self, pivot: &V2) -> Self {
        Self::new(-self.y + pivot.y + pivot.x, self.x - pivot.x + pivot.y)
    }
    fn rotate_90_around_mut(&mut self, pivot: &V2) {
        *self = self.rotate_90_around(pivot);
    }

    fn rotate_180_around(&self, pivot: &V2) -> Self {
        Self::new(pivot.x * 2.0 - self.x, pivot.y * 2.0 - self.y)
    }
    fn rotate_180_around_mut(&mut self, pivot: &V2) {
        *self = self.rotate_180_around(pivot);
    }

    fn rotate_270_around(&self, pivot: &V2) -> Self {
        Self::new(self.y - pivot.y + pivot.x, -self.x + pivot.x + pivot.y)
    }
    fn rotate_270_around_mut(&mut self, pivot: &V2) {
        *self = self.rotate_270_around(pivot);
    }
}

impl Sum for V2 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(V2::zero(), |a, b| a + b)
    }
}

impl From<Point2<f32>> for V2 {
    fn from(point: Point2<f32>) -> Self {
        Self::new(point.x, point.y)
    }
}

impl From<V2> for Point2<f32> {
    fn from(v2: V2) -> Self {
        Point2 { x: v2.x, y: v2.y }
    }
}

impl From<&V2> for Point2<f32> {
    fn from(v2: &V2) -> Self {
        Point2 { x: v2.x, y: v2.y }
    }
}

impl From<&Point2<f32>> for V2 {
    fn from(point: &Point2<f32>) -> Self {
        Self::new(point.x, point.y)
    }
}

impl From<(f32, f32)> for V2 {
    fn from(tuple: (f32, f32)) -> Self {
        V2::new(tuple.0, tuple.1)
    }
}
