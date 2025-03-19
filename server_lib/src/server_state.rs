use std::time::Duration;

use plottery_lib::V2;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{HOST_NAME, HOST_PORT};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy, Default)]
pub struct ServerState {
    pub location: V2,
    pub motors_enabled: bool,
    pub head_down: bool,

    pub plotting: bool,
}

impl ServerState {
    pub fn from_binary(data: &[u8]) -> anyhow::Result<Self> {
        Ok(bincode::deserialize(data)?)
    }
    pub fn to_binary(&self) -> anyhow::Result<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }
}

pub async fn request_server_state(timeout: Option<Duration>) -> anyhow::Result<ServerState> {
    let client = Client::new();
    let mut request = client.get(format!("http://{}:{}/state", HOST_NAME, HOST_PORT));

    match timeout {
        Some(timeout_duration) => request = request.timeout(timeout_duration),
        None => {}
    }

    let bytes = request.send().await?.bytes().await?.to_vec();
    ServerState::from_binary(&bytes)
}
