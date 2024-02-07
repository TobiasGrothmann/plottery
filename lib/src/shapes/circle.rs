use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::{
    traits::{Normalize, Scale, Translate},
    Angle, BoundingBox, Plottable, Rect, Rotate, Rotate90, SampleSettings, Shape, V2,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Circle {
    pub center: V2,
    pub radius: f32,
}

impl Circle {
    pub fn new(center: V2, radius: f32) -> Self {
        Self { center, radius }
    }
    pub fn new_shape(center: V2, radius: f32) -> Shape {
        Shape::Circle(Self { center, radius })
    }

    pub fn area(&self) -> f32 {
        self.radius.powi(2) * PI
    }
    pub fn circumference(&self) -> f32 {
        self.radius * 2.0 * PI
    }
}

impl Plottable for Circle {
    fn get_points(&self, sample_settings: &SampleSettings) -> Vec<V2> {
        let num_samples = sample_settings
            .get_num_points_for_length(self.circumference())
            .max(8);
        let angle_per_step = 2.0 * PI / num_samples as f32;
        (0..num_samples + 1)
            .map(|i| {
                self.center + V2::polar(Angle::from_rad(i as f32 * angle_per_step), self.radius)
            })
            .collect()
    }

    fn length(&self) -> f32 {
        self.circumference()
    }

    fn is_closed(&self) -> bool {
        true
    }
}

impl Rotate for Circle {
    fn rotate(&self, angle: &Angle) -> Self {
        Circle {
            center: self.center.rotate(angle),
            radius: self.radius,
        }
    }
    fn rotate_inplace(&mut self, angle: &Angle) {
        self.center.rotate_inplace(angle);
    }

    fn rotate_around(&self, pivot: &V2, angle: &Angle) -> Self {
        Circle {
            center: self.center.rotate_around(pivot, angle),
            radius: self.radius,
        }
    }
    fn rotate_around_inplace(&mut self, pivot: &V2, angle: &Angle) {
        self.center.rotate_around_inplace(pivot, angle);
    }
}

impl Rotate90 for Circle {
    fn rotate_90(&self) -> Self {
        Circle {
            center: self.center.rotate_90(),
            radius: self.radius,
        }
    }
    fn rotate_90_inplace(&mut self) {
        self.center.rotate_90_inplace();
    }

    fn rotate_180(&self) -> Self {
        Circle {
            center: self.center.rotate_180(),
            radius: self.radius,
        }
    }
    fn rotate_180_inplace(&mut self) {
        self.center.rotate_180_inplace();
    }

    fn rotate_270(&self) -> Self {
        Circle {
            center: self.center.rotate_270(),
            radius: self.radius,
        }
    }
    fn rotate_270_inplace(&mut self) {
        self.center.rotate_270_inplace();
    }

    fn rotate_90_around(&self, pivot: &V2) -> Self {
        Circle {
            center: self.center.rotate_90_around(pivot),
            radius: self.radius,
        }
    }
    fn rotate_90_around_inplace(&mut self, pivot: &V2) {
        self.center.rotate_90_around_inplace(pivot);
    }

    fn rotate_180_around(&self, pivot: &V2) -> Self {
        Circle {
            center: self.center.rotate_180_around(pivot),
            radius: self.radius,
        }
    }
    fn rotate_180_around_inplace(&mut self, pivot: &V2) {
        self.center.rotate_180_around_inplace(pivot);
    }

    fn rotate_270_around(&self, pivot: &V2) -> Self {
        Circle {
            center: self.center.rotate_270_around(pivot),
            radius: self.radius,
        }
    }
    fn rotate_270_around_inplace(&mut self, pivot: &V2) {
        self.center.rotate_270_around_inplace(pivot);
    }
}

impl Translate for Circle {
    fn translate(&self, dist: &V2) -> Self {
        Circle {
            center: self.center + *dist,
            radius: self.radius,
        }
    }
    fn translate_inplace(&mut self, dist: &V2) {
        self.center += *dist;
    }
}

impl Scale for Circle {
    fn scale(&self, scale: f32) -> Self {
        Circle {
            center: self.center * scale,
            radius: self.radius * scale,
        }
    }
    fn scale_inplace(&mut self, scale: f32) {
        self.center *= scale;
        self.radius *= scale;
    }
}

impl Normalize for Circle {}

impl BoundingBox for Circle {
    fn bounding_box(&self) -> Option<Rect> {
        let min = self.center - V2::new(self.radius, self.radius);
        let max = self.center + V2::new(self.radius, self.radius);
        Some(Rect::new(min, max))
    }
}
