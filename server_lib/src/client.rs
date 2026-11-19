use std::{
    fmt,
    net::{IpAddr, SocketAddr},
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

use anyhow::bail;
use mdns_sd::ServiceEvent;

use crate::{discovery, server_state::ServerState, task::Task};

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
    let not_found =
        || anyhow::anyhow!("no plotter found on the local network, is plottery_server running?");
    match &*LOOKUP.lock().unwrap() {
        Some(Lookup::Found(url)) => return Ok(url.clone()),
        Some(Lookup::Failed(at)) if at.elapsed() < DISCOVERY_RETRY_AFTER => return Err(not_found()),
        _ => {}
    }
    let url = discover_plotter(DISCOVERY_TIMEOUT)
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

/// Address of the first plotter that answers within `timeout`.
async fn discover_plotter(timeout: Duration) -> anyhow::Result<Option<SocketAddr>> {
    let daemon = discovery::daemon()?;
    let events = daemon.browse(discovery::SERVICE_TYPE)?;
    let found = tokio::time::timeout(timeout, async {
        while let Ok(event) = events.recv_async().await {
            if let ServiceEvent::ServiceResolved(service) = event {
                if let Some(ip) = service.get_addresses_v4().into_iter().min() {
                    return Some(SocketAddr::new(IpAddr::V4(ip), service.get_port()));
                }
            }
        }
        None
    })
    .await
    .ok()
    .flatten();
    daemon.stop_browse(discovery::SERVICE_TYPE).ok();
    daemon.shutdown().ok();
    Ok(found)
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

    let url = server_url().await.map_err(|err| unreachable(&err))?;
    let mut request = http_client().get(format!("{url}/state"));
    if let Some(timeout_duration) = timeout {
        request = request.timeout(timeout_duration)
    }

    let response = async { request.send().await?.error_for_status()?.bytes().await };
    let bytes = response.await.map_err(|err| {
        forget_server_url();
        unreachable(&err)
    })?;
    ServerState::from_binary(&bytes).map_err(|_| StateRequestError::DifferentVersion)
}

/// Sends `task` to the server. An answer other than success is an error with the server's message.
pub async fn send_task(task: Task) -> anyhow::Result<()> {
    let body = task.to_binary()?;
    let url = format!("{}/task", server_url().await?);
    let response = match http_client().post(url).body(body).send().await {
        Ok(response) => response,
        Err(err) => {
            forget_server_url();
            return Err(err.into());
        }
    };
    let status = response.status();
    if !status.is_success() {
        let message = response.text().await.unwrap_or_default();
        bail!(
            "The plotter server refused the task ({status}): {}",
            message.trim()
        );
    }
    Ok(())
}
