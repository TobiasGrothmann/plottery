use geo_types::Coord;

use crate::{Angle, Rotate, Rotate90};

#[derive(Debug, Copy, Clone)]
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
    pub fn polar(angle: f32, distance: f32) -> Self {
        Self {
            x: angle.cos() * distance,
            y: angle.sin() * distance,
        }
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

    pub fn min(&self, other: &Self) -> Self {
        V2::new(self.x.min(other.x), self.y.min(other.y))
    }
    pub fn max(&self, other: &Self) -> Self {
        V2::new(self.x.max(other.x), self.y.max(other.y))
    }

    pub fn dist(&self, other: &Self) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
    pub fn dist_manhattan(&self, other: &Self) -> f32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
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
}

impl Rotate90 for V2 {
    fn rotate_90(&self) -> Self {
        Self::new(-self.y, self.x)
    }
    fn rotate_180(&self) -> Self {
        Self::new(-self.x, -self.y)
    }
    fn rotate_270(&self) -> Self {
        Self::new(self.y, -self.x)
    }

    fn rotate_90_around(&self, pivot: &V2) -> Self {
        Self::new(-self.y + pivot.y + pivot.x, self.x - pivot.x + pivot.y)
    }
    fn rotate_180_around(&self, pivot: &V2) -> Self {
        Self::new(pivot.x * 2.0 - self.x, pivot.y * 2.0 - self.y)
    }
    fn rotate_270_around(&self, pivot: &V2) -> Self {
        Self::new(self.y - pivot.y + pivot.x, -self.x + pivot.x + pivot.y)
    }
}
