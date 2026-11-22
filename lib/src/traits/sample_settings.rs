use serde::{Deserialize, Serialize};

/// How densely curves are sampled into points.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SampleSettings {
    pub points_per_unit: f32,
}

impl SampleSettings {
    pub fn new(points_per_unit: f32) -> Self {
        Self { points_per_unit }
    }
    pub fn num_points_for_length(&self, length: f32) -> i32 {
        (length.abs() * self.points_per_unit).ceil() as i32
    }
    pub fn low_res() -> Self {
        Self {
            points_per_unit: 1.0,
        }
    }
}

impl From<f32> for SampleSettings {
    fn from(points_per_unit: f32) -> Self {
        Self::new(points_per_unit)
    }
}

impl Default for SampleSettings {
    fn default() -> Self {
        Self {
            points_per_unit: 50.0,
        }
    }
}
