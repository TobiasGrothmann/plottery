use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedRange {
    pub min: f32,
    pub max: f32,
    pub accelleration_distance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotSettings {
    pub corner_slowdown_power: f32,
    pub head_pressure: f32,

    pub speed_draw: SpeedRange,
    pub speed_travel: SpeedRange,
    pub speed_head_down: SpeedRange,
    pub speed_head_up: SpeedRange,
}

impl Default for PlotSettings {
    fn default() -> Self {
        PlotSettings {
            corner_slowdown_power: 10.0,
            head_pressure: 0.5,
            speed_draw: SpeedRange {
                min: 0.75,
                max: 5.0,
                accelleration_distance: 0.22,
            },
            speed_travel: SpeedRange {
                min: 4.0,
                max: 28.0,
                accelleration_distance: 0.17,
            },
            speed_head_down: SpeedRange {
                min: 1.8,
                max: 20.0,
                accelleration_distance: 0.2,
            },
            speed_head_up: SpeedRange {
                min: 35.0,
                max: 35.0,
                accelleration_distance: 0.1,
            },
        }
    }
}
