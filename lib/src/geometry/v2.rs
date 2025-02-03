use std::{f32::consts::PI, iter::Sum};

use geo_types::Coord;
use mint::Point2;
use serde::{Deserialize, Serialize};

use crate::{rand_range, Angle, Rect, Rotate, Rotate90};

use super::v2i::V2i;

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

impl V2 {
    pub fn xy(x_and_y: f32) -> Self {
        Self {
            x: x_and_y,
            y: x_and_y,
        }
    }

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn new_from_geo(geo_coord: &Coord<f32>) -> Self {
        Self {
            x: geo_coord.x,
            y: geo_coord.y,
        }
    }
    pub fn polar(angle: Angle, distance: f32) -> Self {
        Self {
            x: angle.to_rad().cos() * distance,
            y: angle.to_rad().sin() * distance,
        }
    }
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
    pub fn up() -> Self {
        Self { x: 0.0, y: 1.0 }
    }
    pub fn right() -> Self {
        Self { x: 1.0, y: 0.0 }
    }
    pub fn down() -> Self {
        Self { x: 0.0, y: -1.0 }
    }
    pub fn left() -> Self {
        Self { x: 0.0, y: -1.0 }
    }

    pub fn random_unit_circle() -> Self {
        Self::polar(Angle::rand(), 1.0)
    }
    pub fn random_unit_disk() -> Self {
        let angle = Angle::rand();
        let radius = rand_range(0.0, 1.0).sqrt();
        Self::polar(angle, radius)
    }
    pub fn random_in_rect(rect: &Rect) -> Self {
        Self::new(
            rand_range(rect.bl().x, rect.tr().x),
            rand_range(rect.bl().y, rect.tr().y),
        )
    }

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
    pub fn a0() -> Self {
        Self { x: 84.1, y: 118.9 }
    }
    pub fn a1() -> Self {
        Self { x: 59.4, y: 84.1 }
    }
    pub fn a2() -> Self {
        Self { x: 42.0, y: 59.4 }
    }
    pub fn a3() -> Self {
        Self { x: 29.7, y: 42.0 }
    }
    pub fn a4() -> Self {
        Self { x: 21.0, y: 29.7 }
    }
    pub fn a5() -> Self {
        Self { x: 14.8, y: 21.0 }
    }
    pub fn a6() -> Self {
        Self { x: 10.5, y: 14.8 }
    }
    pub fn a7() -> Self {
        Self { x: 7.4, y: 10.5 }
    }
    pub fn a8() -> Self {
        Self { x: 5.2, y: 7.4 }
    }
    pub fn a9() -> Self {
        Self { x: 3.7, y: 5.2 }
    }
    pub fn a10() -> Self {
        Self { x: 2.6, y: 3.7 }
    }

    pub fn swap(&self) -> Self {
        Self {
            x: self.y,
            y: self.x,
        }
    }

    pub fn only_x(&self) -> Self {
        Self { x: self.x, y: 0.0 }
    }
    pub fn only_y(&self) -> Self {
        Self { x: 0.0, y: self.y }
    }

    pub fn as_geo_coord(&self) -> Coord<f32> {
        Coord {
            x: self.x,
            y: self.y,
        }
    }
    pub fn as_tuple(&self) -> (f32, f32) {
        (self.x, self.y)
    }
    pub fn as_array(&self) -> [f32; 2] {
        [self.x, self.y]
    }
    pub fn as_vec(&self) -> Vec<f32> {
        vec![self.x, self.y]
    }

    pub fn round(&self) -> Self {
        Self {
            x: self.x.round(),
            y: self.y.round(),
        }
    }
    pub fn round_to_int(&self) -> V2i {
        V2i::new(self.x.round() as i32, self.y.round() as i32)
    }

    pub fn ceil(&self) -> Self {
        Self {
            x: self.x.ceil(),
            y: self.y.ceil(),
        }
    }
    pub fn ceil_to_int(&self) -> V2i {
        V2i::new(self.x.ceil() as i32, self.y.ceil() as i32)
    }

    pub fn floor(&self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
        }
    }
    pub fn floor_to_int(&self) -> V2i {
        V2i::new(self.x.floor() as i32, self.y.floor() as i32)
    }

    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn min(&self, other: &Self) -> Self {
        V2::new(self.x.min(other.x), self.y.min(other.y))
    }
    pub fn min_axis(&self) -> f32 {
        self.x.min(self.y)
    }
    pub fn max(&self, other: &Self) -> Self {
        V2::new(self.x.max(other.x), self.y.max(other.y))
    }
    pub fn max_axis(&self) -> f32 {
        self.x.max(self.y)
    }

    pub fn dist(&self, other: &Self) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
    pub fn dist_squared(&self, other: &Self) -> f32 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }
    pub fn dist_manhattan(&self, other: &Self) -> f32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    pub fn len(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub fn len_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn angle(&self) -> Angle {
        let mut rad = self.y.atan2(self.x);
        if rad < 0.0 {
            rad += 2.0 * PI;
        }
        Angle::from_rad(rad)
    }
    pub fn angle_to(&self, other: &Self) -> Angle {
        (other - self).angle()
    }

    pub fn normalize(&self) -> Self {
        let len = self.len();
        if len == 0.0 {
            *self
        } else {
            *self / len
        }
    }
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

    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        V2::new(
            self.x + t * (other.x - self.x),
            self.y + t * (other.y - self.y),
        )
    }

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

    pub fn map(&self, f: fn(f32) -> f32) -> Self {
        V2::new(f(self.x), f(self.y))
    }

    pub fn sqrt(&self) -> Self {
        V2::new(self.x.sqrt(), self.y.sqrt())
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
