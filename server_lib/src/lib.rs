use serde::{Deserialize, Serialize};

pub mod accelleration;
pub mod estimation;
pub mod hardware;
mod hardware_test;
pub mod midi;
pub mod pins;
pub mod plot_execution;
pub mod plot_setting;
pub mod server_state;
pub mod task;

pub use estimation::*;
pub use midi::*;
pub use pins::*;
pub use plot_setting::*;
pub use server_state::*;
pub use task::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Axis {
    X,
    Y,
    Head,
}

// TODO: config
pub static HOST_PORT: u16 = 29797;
pub static HOST_NAME: &str = "otter.local";

/// Base URL of the server. `PLOTTERY_HOST` overrides the host (e.g. an IP).
/// Otherwise the name is resolved to an IPv4 address. The server listens
/// on 0.0.0.0 only, and `otter.local` may resolve to an IPv6 address first,
/// which fails with "no route to host" before IPv4 is ever tried.
pub fn server_url() -> String {
    use std::net::ToSocketAddrs;
    if let Ok(host) = std::env::var("PLOTTERY_HOST") {
        return format!("http://{}:{}", host, HOST_PORT);
    }
    let host = (HOST_NAME, HOST_PORT)
        .to_socket_addrs()
        .ok()
        .and_then(|mut addrs| addrs.find(|a| a.is_ipv4()))
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|| HOST_NAME.to_string());
    format!("http://{}:{}", host, HOST_PORT)
}
