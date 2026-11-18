use std::path::Path;

use anyhow::{bail, Result};
use async_process::{Command, Stdio};

/// Version of what the CLI and the editor exchange with a project binary: its subcommands, the
/// piped params and the run stream. Changes with any of them, including the bincode layout of
/// params and layers.
pub const PROTOCOL_VERSION: u32 = 1;

/// Checks what a binary answered to `protocol-version`. Binaries older than the subcommand fail on it.
pub(crate) fn check_protocol_answer(success: bool, stdout: &[u8]) -> Result<()> {
    let binary_version = std::str::from_utf8(stdout)
        .ok()
        .filter(|_| success)
        .and_then(|answer| answer.trim().parse::<u32>().ok());
    if binary_version == Some(PROTOCOL_VERSION) {
        return Ok(());
    }
    let binary_version = binary_version.map_or_else(
        || "an unknown protocol version (older than 1)".to_string(),
        |version| format!("protocol version {version}"),
    );
    bail!(
        "The project is built with {binary_version}, this Plottery uses protocol version \
         {PROTOCOL_VERSION}. Rebuild the project against the installed Plottery, or update Plottery."
    )
}

/// Asks the project binary for its protocol version, an error unless it is [`PROTOCOL_VERSION`].
pub(crate) async fn check_binary_protocol(bin_path: &Path) -> Result<()> {
    let output = Command::new(bin_path)
        .arg("protocol-version")
        .stdin(Stdio::null())
        .output()
        .await?;
    check_protocol_answer(output.status.success(), &output.stdout)
}
