//! How a running project hands its frames and its final layer to the caller: through one Unix
//! domain socket, as messages of `kind: u8`, `len: u64` (little endian) and `len` bytes of
//! bincode [`Layer`]. Any number of snapshots come first, then exactly one final layer.
//!
//! A socket and not a FIFO, because opening a FIFO waits for the other side: a project that dies
//! early would leave the caller hanging.

use std::{
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
};

use anyhow::{bail, Context, Result};
use async_io::Async;
use async_process::Child;
use futures_lite::{future, AsyncRead, AsyncReadExt};
use plottery_lib::Layer;
use plottery_runtime::MessageKind;
use tempfile::TempDir;

/// The next message, `None` at the end of the stream.
async fn read_message(
    stream: &mut (impl AsyncRead + Unpin),
) -> Result<Option<(MessageKind, Layer)>> {
    let mut kind = [0u8; 1];
    if stream.read(&mut kind).await? == 0 {
        return Ok(None);
    }
    let kind = match kind[0] {
        0 => MessageKind::Snapshot,
        1 => MessageKind::Final,
        other => bail!("unknown message kind {other}"),
    };
    let mut len = [0u8; 8];
    stream.read_exact(&mut len).await?;
    let mut binary = vec![0u8; u64::from_le_bytes(len) as usize];
    stream.read_exact(&mut binary).await?;
    Ok(Some((kind, Layer::new_from_binary(&binary)?)))
}

/// Caller side: the socket a started project connects to.
pub struct RunSocket {
    listener: Async<UnixListener>,
    /// Holds nothing but the socket's name, removed on drop.
    dir: TempDir,
}

impl RunSocket {
    const NAME: &str = "run.sock";

    pub(crate) fn bind() -> Result<Self> {
        let dir = tempfile::tempdir()?;
        let listener = Async::<UnixListener>::bind(dir.path().join(Self::NAME))?;
        Ok(Self { listener, dir })
    }

    pub(crate) fn path(&self) -> PathBuf {
        self.dir.path().join(Self::NAME)
    }

    /// Receives what the project `child` sends until it closes the stream: frames go to
    /// `on_frame` as they arrive, the final layer is returned. While `on_frame` is busy the
    /// project waits to send its next frame.
    pub async fn receive(
        &self,
        child: &mut Child,
        mut on_frame: impl FnMut(Layer),
    ) -> Result<Layer> {
        // a project that dies before it connects would otherwise leave this waiting forever
        let mut stream: Async<UnixStream> =
            future::or(async { Ok(self.listener.accept().await?.0) }, async {
                child.status().await?;
                bail!("project exited before connecting")
            })
            .await?;

        let mut result = None;
        while let Some((kind, layer)) = read_message(&mut stream).await? {
            match kind {
                MessageKind::Snapshot => on_frame(layer),
                MessageKind::Final => result = Some(layer),
            }
        }
        result.context("project exited without a result")
    }
}
