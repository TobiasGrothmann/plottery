use std::f32::consts::PI;

use crate::{
    angle::angle::Angle,
    shape::{SampleSettings, Shape},
    traits::{rotate::Rotate, rotate90::Rotate90},
    vec2::V2,
};

#[derive(Debug, Clone)]
pub struct Circle {
    pub center: V2,
    pub radius: f32,
}

impl Circle {
    pub fn new(center: V2, radius: f32) -> Self {
        Self {
            center: center,
            radius: radius,
        }
    }

    pub fn area(&self) -> f32 {
        self.radius.powi(2) * PI
    }
    pub fn circumference(&self) -> f32 {
        self.radius * 2.0 * PI
    }
}

impl Shape for Circle {
    fn get_points(&self, sample_settings: &SampleSettings) -> Vec<V2> {
        let num_samples = sample_settings
            .get_num_points_for_length(self.circumference())
            .max(8);
        let angle_per_step = 2.0 * PI / num_samples as f32;
        (0..num_samples + 1)
            .map(|i| &self.center + V2::polar(i as f32 * angle_per_step, self.radius))
            .collect()
    }
}

impl Rotate for Circle {
    fn rotate(&self, angle: &Angle) -> Self {
        Circle::new(self.center.rotate(angle), self.radius)
    }
    fn rotate_around(&self, pivot: &V2, angle: &Angle) -> Self {
        Circle::new(self.center.rotate_around(pivot, angle), self.radius)
    }
}

impl Rotate90 for Circle {
    fn rotate_90(&self) -> Self {
        Circle::new(self.center.rotate_90(), self.radius)
    }
    fn rotate_180(&self) -> Self {
        Circle::new(self.center.rotate_180(), self.radius)
    }
    fn rotate_270(&self) -> Self {
        Circle::new(self.center.rotate_270(), self.radius)
    }

    fn rotate_90_around(&self, pivot: &V2) -> Self {
        Circle::new(self.center.rotate_90_around(pivot), self.radius)
    }
    fn rotate_180_around(&self, pivot: &V2) -> Self {
        Circle::new(self.center.rotate_180_around(pivot), self.radius)
    }
    fn rotate_270_around(&self, pivot: &V2) -> Self {
        Circle::new(self.center.rotate_270_around(pivot), self.radius)
    }
}
