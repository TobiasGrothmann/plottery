use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedRange {
    pub min: f32,
    pub max: f32,
    pub accelleration_distance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotSettings {
    pub corner_slowdown_power: f32, // the lower the slower
    pub head_pressure: f32,

    pub speed_draw: SpeedRange,
    pub speed_travel: SpeedRange,
    pub speed_head_down: SpeedRange,
    pub speed_head_up: SpeedRange,
}

impl Default for PlotSettings {
    fn default() -> Self {
        PlotSettings {
            corner_slowdown_power: 0.2,
            head_pressure: 0.5,
            speed_draw: SpeedRange {
                min: 0.4,
                max: 4.5,
                accelleration_distance: 0.14,
            },
            speed_travel: SpeedRange {
                min: 0.6,
                max: 7.0,
                accelleration_distance: 1.0,
            },
            speed_head_down: SpeedRange {
                min: 0.5,
                max: 4.0,
                accelleration_distance: 0.22,
            },
            speed_head_up: SpeedRange {
                min: 1.5,
                max: 7.0,
                accelleration_distance: 0.06,
            },
        }
    }
}
