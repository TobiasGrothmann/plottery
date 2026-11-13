use plottery_lib::*;

use crate::{plot_setting::PlotSettings, Axis};

use serde::{Deserialize, Serialize};

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
    SetOrigin(),
    NoteFreq {
        axis: Axis,
        frequency: f32,
        duration_s: f32,
    },
    Note {
        axis: Axis,
        midi: f32,
        duration_s: f32,
    },
}

impl Task {
    pub fn from_binary(data: &[u8]) -> anyhow::Result<Self> {
        Ok(bincode::deserialize(data)?)
    }
    pub fn to_binary(&self) -> anyhow::Result<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }
}

pub async fn send_task(task: Task) -> anyhow::Result<()> {
    let body = task.to_binary()?;
    let url = format!("{}/task", crate::server_url().await?);
    let sent = crate::http_client().post(url).body(body).send().await;
    if sent.is_err() {
        crate::forget_server_url();
    }
    sent?;
    Ok(())
}
