use std::{fmt, time::Duration};

use plottery_lib::V2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy, Default)]
pub struct ServerState {
    pub location: V2,
    pub motors_enabled: bool,
    pub head_down: bool,

    pub activity: Activity,
    /// The running plot, once its timeline is prepared.
    pub plot: Option<PlotProgress>,
    /// The last plot that ended since the server started.
    pub last_plot: Option<PlotResult>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Default)]
pub enum Activity {
    #[default]
    Idle,
    /// Estimating a plot before it starts.
    Preparing,
    Traveling,
    Drawing,
    /// Back to the origin after the last shape.
    Returning,
    /// Any task that is not a plot.
    Moving,
    Aborting,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub struct PlotProgress {
    pub id: u32,
    /// Number of the shape being plotted, starting at 1; equal to `shapes` while returning.
    pub shape: u32,
    pub shapes: u32,
    /// Wall-clock time since the plot task started.
    pub elapsed: Duration,
    pub remaining: Duration,
}

impl PlotProgress {
    /// Progress in time, from 0 to 1.
    pub fn fraction(&self) -> f32 {
        let total = self.elapsed + self.remaining;
        if total.is_zero() {
            return 1.0;
        }
        self.elapsed.as_secs_f32() / total.as_secs_f32()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub struct PlotResult {
    pub id: u32,
    pub outcome: PlotOutcome,
    pub duration: Duration,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum PlotOutcome {
    Finished,
    Aborted,
}

impl ServerState {
    pub fn from_binary(data: &[u8]) -> anyhow::Result<Self> {
        Ok(bincode::deserialize(data)?)
    }
    pub fn to_binary(&self) -> anyhow::Result<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateRequestError {
    Unreachable(String),
    /// The server answered with a state this version cannot decode.
    DifferentVersion,
}

impl fmt::Display for StateRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreachable(reason) => write!(f, "plotter server is unreachable: {reason}"),
            Self::DifferentVersion => write!(f, "plotter server is a different version"),
        }
    }
}

impl std::error::Error for StateRequestError {}

pub async fn request_server_state(
    timeout: Option<Duration>,
) -> Result<ServerState, StateRequestError> {
    let unreachable = |err: &dyn fmt::Display| StateRequestError::Unreachable(err.to_string());

    let url = crate::server_url().await.map_err(|err| unreachable(&err))?;
    let mut request = crate::http_client().get(format!("{url}/state"));
    if let Some(timeout_duration) = timeout {
        request = request.timeout(timeout_duration)
    }

    let response = async { request.send().await?.error_for_status()?.bytes().await };
    let bytes = response.await.map_err(|err| {
        crate::forget_server_url();
        unreachable(&err)
    })?;
    ServerState::from_binary(&bytes).map_err(|_| StateRequestError::DifferentVersion)
}
