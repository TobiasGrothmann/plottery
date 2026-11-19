//! The server advertises itself with mDNS/DNS-SD, the editor finds it by browsing for [`SERVICE_TYPE`].

use std::time::Duration;

use mdns_sd::{IfKind, ServiceDaemon, ServiceInfo};

pub const SERVICE_TYPE: &str = "_plottery._tcp.local.";
pub const PORT: u16 = 29797;

/// IPv4 without loopback: the server listens on IPv4 only.
pub(crate) fn daemon() -> anyhow::Result<ServiceDaemon> {
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
