use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct Angle {
    rad: f32,
}

impl Angle {
    pub fn from_rad(rad: f32) -> Self {
        Self { rad }
    }
    pub fn from_degree(degree: f32) -> Self {
        Self {
            rad: (degree / 360.0) * 2.0 * PI,
        }
    }
    pub fn from_rotations(rotations: f32) -> Self {
        Self {
            rad: rotations * 2.0 * PI,
        }
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

    pub fn wrap(&self) -> Self {
        Angle::from_rad(self.rad % (2.0 * PI))
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
