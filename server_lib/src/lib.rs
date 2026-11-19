use std::{
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

pub mod acceleration;
pub mod discovery;
pub mod estimation;
pub mod hardware;
mod hardware_test;
pub mod midi;
pub mod pins;
pub mod plot_execution;
pub mod plot_settings;
mod plot_settings_test;
pub mod server_state;
pub mod task;
mod task_test;
pub mod telemetry;

pub use estimation::*;
pub use midi::*;
pub use pins::*;
pub use plot_settings::*;
pub use server_state::*;
pub use task::*;
pub use telemetry::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Axis {
    X,
    Y,
    Head,
}

impl Axis {
    pub const COUNT: usize = 3;

    pub fn index(self) -> usize {
        match self {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Head => 2,
        }
    }
}

/// The HTTP client shared by all requests, so connections to the server are kept alive.
pub(crate) fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(reqwest::Client::new)
}

const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(2);
/// After a failed discovery, requests fail right away for this long instead of browsing again.
const DISCOVERY_RETRY_AFTER: Duration = Duration::from_secs(5);

enum Lookup {
    Found(String),
    Failed(Instant),
}

static LOOKUP: Mutex<Option<Lookup>> = Mutex::new(None);

/// Base URL of the server, found with mDNS. Cached until a request to the server fails.
pub async fn server_url() -> anyhow::Result<String> {
    let not_found = || anyhow::anyhow!("no plotter found on the local network, is plottery_server running?");
    match &*LOOKUP.lock().unwrap() {
        Some(Lookup::Found(url)) => return Ok(url.clone()),
        Some(Lookup::Failed(at)) if at.elapsed() < DISCOVERY_RETRY_AFTER => return Err(not_found()),
        _ => {}
    }
    let url = discovery::discover_plotter(DISCOVERY_TIMEOUT)
        .await?
        .map(|addr| format!("http://{addr}"));
    *LOOKUP.lock().unwrap() = Some(match &url {
        Some(url) => Lookup::Found(url.clone()),
        None => Lookup::Failed(Instant::now()),
    });
    url.ok_or_else(not_found)
}

/// Drops the cached address, e.g. after a failed request: the server may have a new one.
pub(crate) fn forget_server_url() {
    let mut lookup = LOOKUP.lock().unwrap();
    if matches!(*lookup, Some(Lookup::Found(_))) {
        *lookup = None;
    }
}
