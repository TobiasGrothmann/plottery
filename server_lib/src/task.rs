use base64::prelude::*;
use plottery_lib::*;
use serde::{Deserialize, Serialize};

use crate::plot_setting::PlotSettings;

#[derive(Debug, Serialize, Deserialize)]
pub enum Task {
    PlotShape {
        shape: Shape,
        sample_settings: SampleSettings,
        plot_settings: PlotSettings,
    },
    Plot {
        layer: Layer,
        sample_settings: SampleSettings,
        plot_settings: PlotSettings,
    },
    Abort,
    SetEnabled(bool),
    SetHead(bool),
    MoveTo(V2, PlotSettings),
    Move(V2, PlotSettings),
}

impl Task {
    pub fn from_base64(encoded: &str) -> anyhow::Result<Self> {
        let decoded = BASE64_STANDARD.decode(encoded)?;
        let deserialized: Task = bincode::deserialize(&decoded)?;
        Ok(deserialized)
    }
    pub fn to_base64(&self) -> anyhow::Result<String> {
        let serialized = bincode::serialize(self)?;
        Ok(BASE64_STANDARD.encode(&serialized))
    }
}
