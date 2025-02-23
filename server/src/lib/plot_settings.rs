use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotSettings {
    pub accelleration_dist: f32,
    pub corner_slowdown_power: f32,
}
