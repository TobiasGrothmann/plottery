use std::{
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

pub mod accelleration;
pub mod estimation;
pub mod hardware;
mod hardware_test;
pub mod midi;
pub mod pins;
pub mod plot_execution;
pub mod plot_setting;
mod plot_setting_test;
pub mod server_state;
pub mod task;
mod task_test;
pub mod telemetry;

pub use estimation::*;
pub use midi::*;
pub use pins::*;
pub use plot_setting::*;
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
    pub fn index(self) -> usize {
        match self {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Head => 2,
        }
    }
}

// TODO: config
pub static HOST_PORT: u16 = 29797;
pub static HOST_NAME: &str = "otter.local";

/// The HTTP client shared by all requests, so connections to the server are kept alive.
pub(crate) fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(reqwest::Client::new)
}

const LOOKUP_TIMEOUT: Duration = Duration::from_secs(2);
/// After a failed lookup, requests fail right away for this long instead of looking up again.
const LOOKUP_RETRY_AFTER: Duration = Duration::from_secs(5);

enum Lookup {
    Found(String),
    Failed(Instant),
}

static LOOKUP: Mutex<Option<Lookup>> = Mutex::new(None);

/// Base URL of the server. `PLOTTERY_HOST` overrides the host (e.g. an IP).
/// Otherwise the name is resolved to an IPv4 address. The server listens
/// on 0.0.0.0 only, and `otter.local` may resolve to an IPv6 address first,
/// which fails with "no route to host" before IPv4 is ever tried.
/// The address is cached until a request to the server fails.
pub async fn server_url() -> anyhow::Result<String> {
    if let Ok(host) = std::env::var("PLOTTERY_HOST") {
        return Ok(format!("http://{}:{}", host, HOST_PORT));
    }
    match &*LOOKUP.lock().unwrap() {
        Some(Lookup::Found(url)) => return Ok(url.clone()),
        Some(Lookup::Failed(at)) if at.elapsed() < LOOKUP_RETRY_AFTER => {
            anyhow::bail!("{HOST_NAME} was not found")
        }
        _ => {}
    }

    let lookup = tokio::net::lookup_host((HOST_NAME, HOST_PORT));
    let found = match tokio::time::timeout(LOOKUP_TIMEOUT, lookup).await {
        Ok(Ok(mut addrs)) => Some(
            addrs
                .find(|addr| addr.is_ipv4())
                .map(|addr| addr.ip().to_string())
                .unwrap_or_else(|| HOST_NAME.to_string()),
        ),
        _ => None,
    };
    let url = found.map(|host| format!("http://{}:{}", host, HOST_PORT));
    *LOOKUP.lock().unwrap() = Some(match &url {
        Some(url) => Lookup::Found(url.clone()),
        None => Lookup::Failed(Instant::now()),
    });
    url.ok_or_else(|| anyhow::anyhow!("{HOST_NAME} was not found"))
}

/// Drops the cached address, e.g. after a failed request: the server may have a new one.
pub(crate) fn forget_server_url() {
    let mut lookup = LOOKUP.lock().unwrap();
    if matches!(*lookup, Some(Lookup::Found(_))) {
        *lookup = None;
    }
}
