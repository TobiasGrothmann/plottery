use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::GR;

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct Angle {
    rad: f32,
}

impl PartialEq for Angle {
    fn eq(&self, other: &Angle) -> bool {
        (self.rad - other.to_rad()).abs() < 0.00001
    }
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

    pub fn mod_2_pi(&self) -> Self {
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
