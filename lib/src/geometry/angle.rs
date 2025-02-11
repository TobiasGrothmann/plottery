use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::{SampleSettings, GR, LARGE_EPSILON};

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialOrd)]
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
    pub fn from_rotations(rotations: f32) -> Self {
        Self {
            rad: rotations * 2.0 * PI,
        }
    }
    pub fn rand() -> Self {
        Self {
            rad: rand::random::<f32>() * 2.0 * PI,
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
    pub fn golden_ratio() -> Self {
        Self::from_rotations(GR)
    }
    pub fn golden_ratio_inverse() -> Self {
        Self::from_rotations(1.0 / GR)
    }
    pub fn root_two() -> Self {
        Self::from_rotations(2.0_f32.sqrt())
    }
    pub fn root_two_inverse() -> Self {
        Self::from_rotations(1.0 / 2.0_f32.sqrt())
    }

    pub fn to_rad(&self) -> f32 {
        self.rad
    }
    pub fn to_degree(&self) -> f32 {
        360.0 * (self.rad / (2.0 * PI))
    }
    pub fn to_rotations(&self) -> f32 {
        self.rad / (2.0 * PI)
    }

    pub fn sin_cos(&self) -> (f32, f32) {
        (self.rad.sin(), self.rad.cos())
    }
    pub fn mod_one_rotation(&self) -> Self {
        Angle::from_rad(self.rad % (2.0 * PI))
    }
    pub fn abs(&self) -> Self {
        Angle::from_rad(self.rad.abs())
    }
    pub fn flip_sign(&self) -> Self {
        Angle::from_rad(self.rad * -1.0)
    }
    pub fn normal_right(&self) -> Self {
        self + Angle::quarter_rotation()
    }

    pub fn lerp(&self, end: Angle, t: f32) -> Angle {
        Angle::from_rad(self.rad * (1.0 - t) + end.rad * t)
    }
    pub fn lerp_iter_fixed(&self, end: Angle, steps: usize) -> AngleInterpolator {
        AngleInterpolator::new(*self, end, steps)
    }
    pub fn lerp_iter(
        &self,
        end: Angle,
        sample_settings: &SampleSettings,
        radius: f32,
    ) -> AngleInterpolator {
        let distance = (end.rad - self.rad).abs() * radius;
        AngleInterpolator::new(
            *self,
            end,
            sample_settings.get_num_points_for_length(distance) as usize,
        )
    }

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
}

pub struct AngleInterpolator {
    start_rad: f32,
    end_rad: f32,
    steps: usize,
    current_step: usize,
}

impl AngleInterpolator {
    pub fn new(start: Angle, end: Angle, steps: usize) -> Self {
        Self {
            start_rad: start.rad,
            end_rad: end.rad,
            steps,
            current_step: 0,
        }
    }
}

impl Iterator for AngleInterpolator {
    type Item = Angle;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_step > self.steps {
            return None;
        }
        let t = self.current_step as f32 / self.steps as f32;
        let interpolated_rad = self.start_rad * (1.0 - t) + self.end_rad * t;
        self.current_step += 1;
        Some(Angle::from_rad(interpolated_rad))
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

impl PartialEq for Angle {
    fn eq(&self, other: &Angle) -> bool {
        (self.rad - other.rad).abs() < LARGE_EPSILON
    }
}

impl Eq for Angle {}
