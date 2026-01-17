use std::{f32::consts::PI, iter::Sum};

use geo_types::Coord;
use mint::Point2;
use serde::{Deserialize, Serialize};

use crate::{
    rand_range, Angle, Mirror, Rect, Rotate, Rotate90, SampleSettings, Scale, Transform, Translate,
    V2i,
};

/// A 2D vector with floating-point coordinates.
///
/// # Examples
///
/// ```
/// # use plottery_lib::*;
/// let v1 = V2::new(1.0, 2.0);
/// let v2 = V2::new(3.0, 4.0);
/// let sum = v1 + v2;
/// let scaled = v1 * 2.0;
/// ```
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct V2 {
    /// In **Plottery** this represents the horizontal axis (left/right)
    pub x: f32,
    /// In **Plottery** this represents the vertical axis (up/down)
    pub y: f32,
}

impl V2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    /// Creates a new V2 with the same value for both x and y.
    pub fn xy(x_and_y: f32) -> Self {
        Self {
            x: x_and_y,
            y: x_and_y,
        }
    }

    /// Creates a new V2 from a geo coordinate.
    pub fn new_from_geo(geo_coord: &Coord<f32>) -> Self {
        Self {
            x: geo_coord.x,
            y: geo_coord.y,
        }
    }

    /// Creates a new V2 from polar coordinates (angle and distance).
    pub fn polar(angle: Angle, distance: f32) -> Self {
        Self {
            x: angle.to_rad().cos() * distance,
            y: angle.to_rad().sin() * distance,
        }
    }

    /// Returns the zero vector (0, 0).
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
    /// Returns the unit vector (0, 1) pointing up. see also [`Angle::up_cc()`]
    pub fn up() -> Self {
        Self { x: 0.0, y: 1.0 }
    }
    /// Returns the unit vector (1, 0) pointing right. see also [`Angle::right_cc()`]
    pub fn right() -> Self {
        Self { x: 1.0, y: 0.0 }
    }
    /// Returns the unit vector (0, -1) pointing down. see also [`Angle::down_cc()`]
    pub fn down() -> Self {
        Self { x: 0.0, y: -1.0 }
    }
    /// Returns the unit vector (-1, 0) pointing left. see also [`Angle::left_cc()`]
    pub fn left() -> Self {
        Self { x: -1.0, y: 0.0 }
    }

    /// Returns a random vector exactly on the unit circle.
    pub fn random_unit_circle() -> Self {
        Self::polar(Angle::rand(), 1.0)
    }
    /// Returns a random vector inside or on the unit circle.
    pub fn random_unit_disk() -> Self {
        let angle = Angle::rand();
        let radius = rand_range(0.0, 1.0);
        Self::polar(angle, radius)
    }
    /// Returns a random vector inside or on the given rectangle.
    pub fn random_in_rect(rect: &Rect) -> Self {
        Self::new(
            rand_range(rect.bl().x, rect.tr().x),
            rand_range(rect.bl().y, rect.tr().y),
        )
    }

    /// Returns a vector with dimensions for a DIN A paper size specified by number.
    /// Uses [ISO 216](https://en.wikipedia.org/wiki/ISO_216) standard.
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

    /// Returns dimensions for A0 paper size (84.1 × 118.9 cm).
    pub fn a0() -> Self {
        Self { x: 84.1, y: 118.9 }
    }
    /// Returns dimensions for A1 paper size (59.4 × 84.1 cm).
    pub fn a1() -> Self {
        Self { x: 59.4, y: 84.1 }
    }
    /// Returns dimensions for A2 paper size (42.0 × 59.4 cm).
    pub fn a2() -> Self {
        Self { x: 42.0, y: 59.4 }
    }
    /// Returns dimensions for A3 paper size (29.7 × 42.0 cm).
    pub fn a3() -> Self {
        Self { x: 29.7, y: 42.0 }
    }
    /// Returns dimensions for A4 paper size (21.0 × 29.7 cm).
    pub fn a4() -> Self {
        Self { x: 21.0, y: 29.7 }
    }
    /// Returns dimensions for A5 paper size (14.8 × 21.0 cm).
    pub fn a5() -> Self {
        Self { x: 14.8, y: 21.0 }
    }
    /// Returns dimensions for A6 paper size (10.5 × 14.8 cm).
    pub fn a6() -> Self {
        Self { x: 10.5, y: 14.8 }
    }
    /// Returns dimensions for A7 paper size (7.4 × 10.5 cm).
    pub fn a7() -> Self {
        Self { x: 7.4, y: 10.5 }
    }
    /// Returns dimensions for A8 paper size (5.2 × 7.4 cm).
    pub fn a8() -> Self {
        Self { x: 5.2, y: 7.4 }
    }
    /// Returns dimensions for A9 paper size (3.7 × 5.2 cm).
    pub fn a9() -> Self {
        Self { x: 3.7, y: 5.2 }
    }
    /// Returns dimensions for A10 paper size (2.6 × 3.7 cm).
    pub fn a10() -> Self {
        Self { x: 2.6, y: 3.7 }
    }

    /// Returns a vector with dimensions for a B paper size specified by number.
    /// Uses [ISO 216](https://en.wikipedia.org/wiki/ISO_216) standard.
    pub fn din_b(number: u8) -> Self {
        match number {
            0 => Self::b0(),
            1 => Self::b1(),
            2 => Self::b2(),
            3 => Self::b3(),
            4 => Self::b4(),
            5 => Self::b5(),
            6 => Self::b6(),
            7 => Self::b7(),
            8 => Self::b8(),
            9 => Self::b9(),
            10 => Self::b10(),
            _ => panic!("DIN B number out of range."),
        }
    }

    /// Returns dimensions for B0 paper size (100.0 × 141.4 cm).
    pub fn b0() -> Self {
        Self { x: 84.1, y: 118.9 }
    }
    /// Returns dimensions for B1 paper size (70.7 × 100.0 cm).
    pub fn b1() -> Self {
        Self { x: 70.7, y: 100.0 }
    }
    /// Returns dimensions for B2 paper size (50.0 × 70.7 cm).
    pub fn b2() -> Self {
        Self { x: 50.0, y: 70.7 }
    }
    /// Returns dimensions for B3 paper size (35.3 × 50.0 cm).
    pub fn b3() -> Self {
        Self { x: 35.3, y: 50.0 }
    }
    /// Returns dimensions for B4 paper size (25.0 × 35.3 cm).
    pub fn b4() -> Self {
        Self { x: 25.0, y: 35.3 }
    }
    /// Returns dimensions for B5 paper size (17.6 × 25.0 cm).
    pub fn b5() -> Self {
        Self { x: 17.6, y: 25.0 }
    }
    /// Returns dimensions for B6 paper size (12.5 × 17.6 cm).
    pub fn b6() -> Self {
        Self { x: 12.5, y: 17.6 }
    }
    /// Returns dimensions for B7 paper size (8.8 × 12.5 cm).
    pub fn b7() -> Self {
        Self { x: 8.8, y: 12.5 }
    }
    /// Returns dimensions for B8 paper size (6.2 × 8.8 cm).
    pub fn b8() -> Self {
        Self { x: 6.2, y: 8.8 }
    }
    /// Returns dimensions for B9 paper size (4.4 × 6.2 cm).
    pub fn b9() -> Self {
        Self { x: 4.4, y: 6.2 }
    }
    /// Returns dimensions for B10 paper size (3.1 × 4.4 cm).
    pub fn b10() -> Self {
        Self { x: 3.1, y: 4.4 }
    }

    /// Returns a new V2 with x and y values swapped.
    pub fn swap_axes(&self) -> Self {
        Self {
            x: self.y,
            y: self.x,
        }
    }

    /// Returns a new V2 preserving x but setting y to 0.0.
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

    /// Returns a new V2 preserving y but setting x to 0.0.
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

    /// Converts to a geo coordinate.
    pub fn as_geo_coord(&self) -> Coord<f32> {
        Coord {
            x: self.x,
            y: self.y,
        }
    }

    /// Returns a tuple of (x, y).
    pub fn as_tuple(&self) -> (f32, f32) {
        (self.x, self.y)
    }
    /// Returns an array [x, y].
    pub fn as_array(&self) -> [f32; 2] {
        [self.x, self.y]
    }
    /// Returns a vector [x, y].
    pub fn as_vec(&self) -> Vec<f32> {
        vec![self.x, self.y]
    }

    /// Returns a new V2 with rounded x and y values.
    pub fn round(&self) -> Self {
        Self {
            x: self.x.round(),
            y: self.y.round(),
        }
    }
    /// Returns a new V2i with x and y values rounded to integers.
    pub fn round_to_int(&self) -> V2i {
        V2i::new(self.x.round() as i32, self.y.round() as i32)
    }

    /// Returns a new V2 with ceiling of x and y values.
    pub fn ceil(&self) -> Self {
        Self {
            x: self.x.ceil(),
            y: self.y.ceil(),
        }
    }
    /// Returns a new V2i with ceiling of x and y values converted to integers.
    pub fn ceil_to_int(&self) -> V2i {
        V2i::new(self.x.ceil() as i32, self.y.ceil() as i32)
    }

    /// Returns a new V2 with floor of x and y values.
    pub fn floor(&self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
        }
    }
    /// Returns a new V2i with floor of x and y values converted to integers.
    pub fn floor_to_int(&self) -> V2i {
        V2i::new(self.x.floor() as i32, self.y.floor() as i32)
    }

    /// Calculates the dot product of this vector with another vector.
    ///
    /// The dot product of vectors **a** and **b** is: a · b = a.x * b.x + a.y * b.y
    ///
    /// # Properties
    /// - If a · b > 0, the vectors point in a similar direction
    /// - If a · b < 0, the vectors point in opposite directions
    /// - If a · b = 0, the vectors are perpendicular (orthogonal)
    pub fn dot(&self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Returns a new V2 where each component is the minimum of the corresponding components.
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let v1 = V2::new(1.0, 2.0);
    /// let v2 = V2::new(2.0, 1.0);
    /// assert_eq!(v1.min(v2), V2::new(1.0, 1.0));
    /// ```
    pub fn min(&self, other: Self) -> Self {
        V2::new(self.x.min(other.x), self.y.min(other.y))
    }
    /// Returns a new V2 where each component is the maximum of the corresponding components.
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let v1 = V2::new(1.0, 2.0);
    /// let v2 = V2::new(2.0, 1.0);
    /// assert_eq!(v1.max(v2), V2::new(2.0, 2.0));
    /// ```
    pub fn max(&self, other: Self) -> Self {
        V2::new(self.x.max(other.x), self.y.max(other.y))
    }

    /// Returns the smaller of x and y components.
    pub fn min_axis(&self) -> f32 {
        self.x.min(self.y)
    }
    /// Returns the larger of x and y components.
    pub fn max_axis(&self) -> f32 {
        self.x.max(self.y)
    }

    /// Calculates the Euclidean distance to another vector.
    pub fn dist(&self, other: Self) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
    /// Calculates the squared Euclidean distance to another vector.
    pub fn dist_squared(&self, other: Self) -> f32 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }
    /// Calculates the Manhattan (taxicab) distance to another vector.
    ///
    /// The Manhattan distance between points a and b is: |a.x - b.x| + |a.y - b.y|
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let v1 = V2::new(0.0, 0.0);
    /// let v2 = V2::new(2.0, 2.0);
    /// assert_eq!(v1.dist_manhattan(v2), 4.0);
    /// ```
    pub fn dist_manhattan(&self, other: Self) -> f32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    /// Returns the length (magnitude) of this vector.
    pub fn len(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    /// Returns the squared length of this vector.
    pub fn len_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Returns the angle of this vector.
    pub fn angle(&self) -> Angle {
        let mut rad = self.y.atan2(self.x);
        if rad < 0.0 {
            rad += 2.0 * PI;
        }
        Angle::from_rad(rad)
    }
    /// Returns the angle from this vector to another vector.
    pub fn angle_to(&self, other: Self) -> Angle {
        (other - self).angle()
    }

    /// Returns this vector normalized to a length of 1.0.
    pub fn normalize(&self) -> Self {
        let len = self.len();
        if len == 0.0 {
            *self
        } else {
            *self / len
        }
    }
    /// Returns this vector with the same direction but a specified length.
    pub fn normalize_to(&self, len: f32) -> Self {
        *self * len / self.len()
    }

    /// Projects this vector onto another vector.
    pub fn project_onto(&self, other: Self) -> Self {
        let length_squared = other.len_squared();
        let dot_product = self.dot(other);
        V2::new(
            (dot_product / length_squared) * other.x,
            (dot_product / length_squared) * other.y,
        )
    }

    /// Returns a vector with the same direction but length clamped between min_len and max_len.
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

    /// Clamps the vector's components inside the given range. This is equivalent to clamping
    /// it inside a rectangle defined by the corners `min` and `max`.
    pub fn clamp(&self, min: V2, max: V2) -> Self {
        V2::new(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y))
    }

    /// Returns a new V2 with a function applied to both x and y components.
    pub fn map(&self, f: fn(f32) -> f32) -> Self {
        V2::new(f(self.x), f(self.y))
    }

    /// Returns a new V2 with square root applied to both components.
    pub fn sqrt(&self) -> Self {
        V2::new(self.x.sqrt(), self.y.sqrt())
    }

    /// Linearly interpolates between this vector and another.
    ///
    /// When t=0.0, returns self. When t=1.0, returns other.
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let v1 = V2::new(0.0, 0.0);
    /// let v2 = V2::new(2.0, 2.0);
    /// assert_eq!(v1.lerp(v2, 0.5), V2::new(1.0, 1.0));
    /// ```
    pub fn lerp(&self, other: Self, t: f32) -> Self {
        V2::new(
            self.x + t * (other.x - self.x),
            self.y + t * (other.y - self.y),
        )
    }

    /// Returns an iterator to interpolate from this vector to another in a fixed number of steps.
    ///
    /// The iterator yields steps+1 points, including both start and end points.
    ///
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

    /// Returns an iterator to interpolate from this vector to another with density based on sample settings.
    ///
    /// The number of points is determined by the distance between vectors and the sample settings.
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let start = V2::new(0.0, 0.0);
    /// let end = V2::new(10.0, 10.0);
    /// for point in start.lerp_iter(end, SampleSettings::new(5.0)) {
    ///     println!("{:?}", point);
    /// }
    /// ```
    pub fn lerp_iter(&self, end: V2, sample_settings: SampleSettings) -> V2Interpolator {
        let distance = self.dist(end);
        V2Interpolator::new(
            *self,
            end,
            sample_settings.get_num_points_for_length(distance) as usize,
        )
    }

    /// Returns a new `V2` with a distance towads `around` raised to the power of `distance_power`.
    /// The angle to `around` remains the same.
    ///
    /// ### Example
    /// ```
    /// # use plottery_lib::*;
    /// let v = V2::new(2.0, 0.0); // point to distort
    /// let around = V2::zero(); // center point for distortion
    /// let power = 3.0;
    /// assert_eq!(v.distort_pow(around, power), V2::new(2.0_f32.powf(power), 0.0));
    /// ```
    pub fn distort_pow(&self, around: V2, distance_power: f32) -> Self {
        let distance = self.dist(around);
        let angle = around.angle_to(*self);
        let new_distance = distance.powf(distance_power);
        around + V2::polar(angle, new_distance)
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
        let interpolated = self.start.lerp(self.end, t);
        self.current_step += 1;
        Some(interpolated)
    }
}

impl Rotate for V2 {
    fn rotate(&self, angle: Angle) -> Self {
        let angle = angle.to_rad();
        let angle_sin = angle.sin();
        let angle_cos = angle.cos();
        Self::new(
            self.x * angle_cos - self.y * angle_sin,
            self.x * angle_sin + self.y * angle_cos,
        )
    }
    fn rotate_mut(&mut self, angle: Angle) {
        *self = self.rotate(angle);
    }

    fn rotate_around(&self, pivot: V2, angle: Angle) -> Self {
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
    fn rotate_around_mut(&mut self, pivot: V2, angle: Angle) {
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

    fn rotate_90_around(&self, pivot: V2) -> Self {
        Self::new(-self.y + pivot.y + pivot.x, self.x - pivot.x + pivot.y)
    }
    fn rotate_90_around_mut(&mut self, pivot: V2) {
        *self = self.rotate_90_around(pivot);
    }

    fn rotate_180_around(&self, pivot: V2) -> Self {
        Self::new(pivot.x * 2.0 - self.x, pivot.y * 2.0 - self.y)
    }
    fn rotate_180_around_mut(&mut self, pivot: V2) {
        *self = self.rotate_180_around(pivot);
    }

    fn rotate_270_around(&self, pivot: V2) -> Self {
        Self::new(self.y - pivot.y + pivot.x, -self.x + pivot.x + pivot.y)
    }
    fn rotate_270_around_mut(&mut self, pivot: V2) {
        *self = self.rotate_270_around(pivot);
    }
}

impl Mirror for V2 {
    fn mirror_x(&self) -> Self {
        Self::new(-self.x, self.y)
    }
    fn mirror_x_mut(&mut self) {
        *self = self.mirror_x();
    }

    fn mirror_y(&self) -> Self {
        Self::new(self.x, -self.y)
    }
    fn mirror_y_mut(&mut self) {
        *self = self.mirror_y();
    }
}

impl Translate for V2 {
    fn translate(&self, dist: V2) -> Self {
        self + dist
    }
    fn translate_mut(&mut self, dist: V2) {
        *self += dist;
    }
}

impl Transform for V2 {
    fn transform(&self, matrix: &super::TransformMatrix) -> Self {
        matrix.mul_vector(*self)
    }

    fn transform_mut(&mut self, matrix: &super::TransformMatrix) {
        *self = matrix.mul_vector(*self);
    }
}

impl Scale for V2 {
    fn scale(&self, factor: f32) -> Self {
        self * factor
    }

    fn scale_mut(&mut self, factor: f32) {
        *self *= factor;
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
