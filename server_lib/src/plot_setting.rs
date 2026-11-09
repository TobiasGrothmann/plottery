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
    pub head_travel_beyond_paper_cm: f32, // extra pen travel after touching the paper

    pub speed_draw: SpeedRange,
    pub speed_travel: SpeedRange,
    pub speed_head_down: SpeedRange,
    pub speed_head_up: SpeedRange,
}

impl Default for PlotSettings {
    fn default() -> Self {
        // These are the speeds the machine physically ran at before the
        // 2026-10 timing fix: the old settings (draw 0.4-4.5, travel
        // 0.6-7.0, head down 0.5-4.0, head up 1.5-7.0) were tuned against a
        // hidden 62 µs per step, which is now gone. Real limits are found in
        // the retune (docs/step_timing.md, 0.5).
        PlotSettings {
            corner_slowdown_power: 0.2,
            head_travel_beyond_paper_cm: 0.3,
            speed_draw: SpeedRange {
                min: 0.39,
                max: 3.41,
                accelleration_distance: 0.14,
            },
            speed_travel: SpeedRange {
                min: 0.58,
                max: 4.68,
                accelleration_distance: 1.0,
            },
            speed_head_down: SpeedRange {
                min: 0.47,
                max: 2.68,
                accelleration_distance: 0.22,
            },
            speed_head_up: SpeedRange {
                min: 1.27,
                max: 3.75,
                accelleration_distance: 0.06,
            },
        }
    }
}
