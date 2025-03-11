use plottery_lib::V2;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{HOST_NAME, HOST_PORT};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerState {
    pub current_position: V2,
    pub plotting: bool,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            current_position: V2::new(0.0, 0.0),
            plotting: false,
        }
    }

    pub fn from_binary(data: &[u8]) -> anyhow::Result<Self> {
        Ok(bincode::deserialize(data)?)
    }
    pub fn to_binary(&self) -> anyhow::Result<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }
}

pub async fn request_server_state() -> anyhow::Result<ServerState> {
    let client = Client::new();
    let bytes = client
        .get(format!("http://{}:{}/state", HOST_NAME, HOST_PORT))
        .send()
        .await?
        .bytes()
        .await?
        .to_vec();
    Ok(ServerState::from_binary(&bytes)?)
}
