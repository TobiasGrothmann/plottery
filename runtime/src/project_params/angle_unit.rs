use plottery_lib::Angle;
use serde::{Deserialize, Serialize};

/// Unit an `Angle` param is shown in. The value itself is stored in radians.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AngleUnit {
    #[default]
    Degrees,
    Rotations,
    Radians,
}

impl AngleUnit {
    pub const ALL: [AngleUnit; 3] = [AngleUnit::Degrees, AngleUnit::Rotations, AngleUnit::Radians];

    pub fn short_name(&self) -> &'static str {
        match self {
            AngleUnit::Degrees => "deg",
            AngleUnit::Rotations => "rot",
            AngleUnit::Radians => "rad",
        }
    }

    pub fn from_angle(&self, angle: Angle) -> f32 {
        match self {
            AngleUnit::Degrees => angle.to_degree(),
            AngleUnit::Rotations => angle.to_rotations(),
            AngleUnit::Radians => angle.to_rad(),
        }
    }

    pub fn to_angle(&self, value: f32) -> Angle {
        match self {
            AngleUnit::Degrees => Angle::from_degrees(value),
            AngleUnit::Rotations => Angle::from_rotations(value),
            AngleUnit::Radians => Angle::from_rad(value),
        }
    }
}
