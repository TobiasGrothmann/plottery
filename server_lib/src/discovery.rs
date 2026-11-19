//! The server advertises itself with mDNS/DNS-SD, the editor finds it by browsing for [`SERVICE_TYPE`].

use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use mdns_sd::{IfKind, ServiceDaemon, ServiceEvent, ServiceInfo};

pub const SERVICE_TYPE: &str = "_plottery._tcp.local.";
pub const PORT: u16 = 29797;

/// IPv4 without loopback: the server listens on IPv4 only.
fn daemon() -> anyhow::Result<ServiceDaemon> {
    let daemon = ServiceDaemon::new()?;
    daemon.disable_interface(IfKind::IPv6)?;
    daemon.disable_interface(IfKind::LoopbackV4)?;
    Ok(daemon)
}

/// Keeps the server advertised on the local network until dropped.
pub struct Advertisement {
    daemon: ServiceDaemon,
    fullname: String,
}

impl Advertisement {
    pub fn start() -> anyhow::Result<Self> {
        let daemon = daemon()?;
        // A host name of its own, so the records never clash with those avahi publishes for the machine.
        let info = ServiceInfo::new(
            SERVICE_TYPE,
            "plottery",
            "plottery-server.local.",
            (),
            PORT,
            None,
        )?
        .enable_addr_auto();
        let fullname = info.get_fullname().to_string();
        daemon.register(info)?;
        Ok(Self { daemon, fullname })
    }
}

impl Drop for Advertisement {
    fn drop(&mut self) {
        // Waits for the goodbye packets, so browsers forget the server right away.
        if let Ok(status) = self.daemon.unregister(&self.fullname) {
            status.recv_timeout(Duration::from_secs(1)).ok();
        }
        self.daemon.shutdown().ok();
    }
}

/// Address of the first plotter that answers within `timeout`.
pub async fn discover_plotter(timeout: Duration) -> anyhow::Result<Option<SocketAddr>> {
    let daemon = daemon()?;
    let events = daemon.browse(SERVICE_TYPE)?;
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
    daemon.stop_browse(SERVICE_TYPE).ok();
    daemon.shutdown().ok();
    Ok(found)
}
