//! Project side of the run stream, the caller's side is `RunSocket` in `plottery_project`.

use std::io::Write;

use anyhow::Result;
use plottery_lib::Layer;

/// First byte of a run stream message.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageKind {
    Snapshot = 0,
    Final = 1,
}

/// Project side: sends `layer` as one message.
pub(crate) fn write_message(
    stream: &mut impl Write,
    kind: MessageKind,
    layer: &Layer,
) -> Result<()> {
    let binary = layer.to_binary()?;
    let mut header = [kind as u8; 9];
    header[1..].copy_from_slice(&(binary.len() as u64).to_le_bytes());
    stream.write_all(&header)?;
    stream.write_all(&binary)?;
    Ok(())
}
