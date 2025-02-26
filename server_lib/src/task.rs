use base64::prelude::*;
use plottery_lib::*;

use crate::{plot_setting::PlotSettings, HOST_NAME, HOST_PORT};

use reqwest::Client;
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

pub async fn send_task(task: Task) -> anyhow::Result<()> {
    let client = Client::new();
    let body = task.to_base64().unwrap();
    client
        .post(&format!("http://{}:{}/task", HOST_NAME, HOST_PORT))
        .body(body)
        .send()
        .await?;
    Ok(())
}
